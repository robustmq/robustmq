// Copyright 2023 RobustMQ Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::common::min_qos;
use super::common::Subscriber;
use crate::common::types::ResultMqttBrokerError;
use crate::handler::cache::{
    MQTTCacheManager, QosAckPackageData, QosAckPackageType, QosAckPacketInfo,
};
use crate::handler::error::MqttBrokerError;
use crate::handler::message::is_message_expire;
use crate::handler::sub_option::{get_retain_flag_by_retain_as_published, is_send_msg_by_bo_local};
use crate::subscribe::common::{is_ignore_push_error, SubPublishParam};
use axum::extract::ws::Message;
use bytes::{Bytes, BytesMut};
use common_base::network::broker_not_available;
use common_base::tools::now_mills;
use common_metrics::mqtt::packets::record_sent_metrics;
use common_metrics::mqtt::publish::record_mqtt_message_bytes_sent;
use common_metrics::mqtt::publish::record_mqtt_messages_sent_inc;
use common_metrics::mqtt::time::record_mqtt_packet_send_duration;
use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::message::MqttMessage;
use network_server::common::connection_manager::ConnectionManager;
use network_server::common::packet::build_mqtt_packet_wrapper;
use network_server::common::packet::ResponsePackage;
use protocol::mqtt::codec::MqttCodec;
use protocol::mqtt::codec::MqttPacketWrapper;
use protocol::mqtt::common::mqtt_packet_to_string;
use protocol::mqtt::common::qos;
use protocol::mqtt::common::{MqttPacket, PubRel, Publish, PublishProperties, QoS};
use protocol::robust::RobustMQPacket;
use protocol::robust::RobustMQProtocol;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::{sleep, timeout};
use tracing::{debug, warn};

#[derive(Clone)]
pub struct BuildPublishMessageContext {
    pub cache_manager: Arc<MQTTCacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub client_id: String,
    pub record: Record,
    pub group_id: String,
    pub qos: QoS,
    pub subscriber: Subscriber,
    pub sub_ids: Vec<usize>,
}

pub async fn build_publish_message(
    context: BuildPublishMessageContext,
) -> Result<Option<SubPublishParam>, MqttBrokerError> {
    let msg = MqttMessage::decode_record(context.record.clone())?;

    if is_message_expire(&msg) {
        debug!("Message dropping: message expires, is not pushed to the client, and is discarded");
        return Ok(None);
    }

    if !is_send_msg_by_bo_local(
        context.subscriber.nolocal,
        &context.subscriber.client_id,
        &msg.client_id,
    ) {
        debug!(
            "Message dropping: message is not pushed to the client, because the client_id is the same as the subscriber, client_id: {}, topic_id: {}",
            context.subscriber.client_id, context.subscriber.topic_id
        );
        return Ok(None);
    }

    let connect_id = if let Some(id) = context.cache_manager.get_connect_id(&context.client_id) {
        id
    } else {
        return Err(MqttBrokerError::ConnectionNullSkipPushMessage(
            context.client_id.to_owned(),
        ));
    };

    if let Some(conn) = context.cache_manager.get_connection(connect_id) {
        if msg.payload.len() > (conn.max_packet_size as usize) {
            debug!(
                "{:?}",
                MqttBrokerError::PacketsExceedsLimitBySubPublish(
                    context.client_id.to_owned(),
                    msg.payload.len(),
                    conn.max_packet_size
                )
            );
            return Ok(None);
        }
    }

    let mut contain_properties = false;
    if let Some(protocol) = context.connection_manager.get_connect_protocol(connect_id) {
        if protocol.is_mqtt5() {
            contain_properties = true;
        }
    }

    let pkid = context
        .cache_manager
        .pkid_metadata
        .generate_pkid(&context.client_id, &context.qos)
        .await;

    let retain =
        get_retain_flag_by_retain_as_published(context.subscriber.preserve_retain, msg.retain);

    let publish = Publish {
        dup: false,
        qos: context.qos,
        p_kid: pkid,
        retain,
        topic: Bytes::from(context.subscriber.topic_name.clone()),
        payload: msg.payload,
    };

    let properties = if contain_properties {
        Some(PublishProperties {
            payload_format_indicator: msg.format_indicator,
            message_expiry_interval: Some(msg.expiry_interval as u32),
            topic_alias: None,
            response_topic: msg.response_topic,
            correlation_data: msg.correlation_data,
            user_properties: msg.user_properties,
            subscription_identifiers: context.sub_ids,
            content_type: msg.content_type,
        })
    } else {
        None
    };

    let packet = MqttPacket::Publish(publish, properties);
    let sub_pub_param = SubPublishParam::new(
        context.subscriber.clone(),
        packet,
        context.record.timestamp as u128,
        context.group_id.to_string(),
        pkid,
    );
    Ok(Some(sub_pub_param))
}

pub async fn send_publish_packet_to_client(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<MQTTCacheManager>,
    sub_pub_param: &SubPublishParam,
    qos: &QoS,
    stop_sx: &Sender<bool>,
) -> ResultMqttBrokerError {
    match qos {
        QoS::AtMostOnce => {
            push_packet_to_client(cache_manager, connection_manager, sub_pub_param, stop_sx)
                .await?;
        }

        QoS::AtLeastOnce => {
            let (wait_puback_sx, _) = broadcast::channel(1);
            let client_id = sub_pub_param.subscribe.client_id.clone();
            let pkid: u16 = sub_pub_param.pkid;
            cache_manager.pkid_metadata.add_ack_packet(
                &client_id,
                pkid,
                QosAckPacketInfo {
                    sx: wait_puback_sx.clone(),
                    create_time: now_mills(),
                },
            );

            exclusive_publish_message_qos1(
                cache_manager,
                connection_manager,
                sub_pub_param,
                stop_sx,
                &wait_puback_sx,
            )
            .await?;

            cache_manager
                .pkid_metadata
                .remove_ack_packet(&client_id, pkid);
        }

        QoS::ExactlyOnce => {
            let (wait_ack_sx, _) = broadcast::channel(1);
            let client_id = sub_pub_param.subscribe.client_id.clone();
            let pkid = sub_pub_param.pkid;
            cache_manager.pkid_metadata.add_ack_packet(
                &client_id,
                pkid,
                QosAckPacketInfo {
                    sx: wait_ack_sx.clone(),
                    create_time: now_mills(),
                },
            );

            exclusive_publish_message_qos2(
                cache_manager,
                connection_manager,
                sub_pub_param,
                stop_sx,
                &wait_ack_sx,
            )
            .await?;

            cache_manager
                .pkid_metadata
                .remove_ack_packet(&client_id, pkid);
        }
    }
    Ok(())
}

pub fn build_pub_qos(cache_manager: &Arc<MQTTCacheManager>, subscriber: &Subscriber) -> QoS {
    let cluster_qos = cache_manager
        .broker_cache
        .get_cluster_config()
        .mqtt_protocol_config
        .max_qos;
    min_qos(qos(cluster_qos).unwrap(), subscriber.qos)
}

pub fn build_sub_ids(subscriber: &Subscriber) -> Vec<usize> {
    let mut sub_ids = Vec::new();
    if let Some(id) = subscriber.subscription_identifier {
        sub_ids.push(id);
    }
    sub_ids
}

// When the subscription QOS is 0,
// the message can be pushed directly to the request return queue without the need for a retry mechanism.
pub async fn push_packet_to_client(
    cache_manager: &Arc<MQTTCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
) -> ResultMqttBrokerError {
    let action_fn = || async {
        let client_id = sub_pub_param.subscribe.client_id.clone();

        if cache_manager.get_session_info(&client_id).is_none() {
            return Err(MqttBrokerError::SessionNullSkipPushMessage(client_id));
        }

        let connect_id = if let Some(id) = cache_manager.get_connect_id(&client_id) {
            id
        } else {
            return Err(MqttBrokerError::ConnectionNullSkipPushMessage(client_id));
        };

        let packet = RobustMQPacket::MQTT(sub_pub_param.packet.clone());
        let resp = ResponsePackage::new(connect_id, packet, 0, 0, 0, "Subsceibe".to_string());

        send_message_to_client(resp, connection_manager).await
    };

    retry_tool_fn_timeout(action_fn, stop_sx, "push_packet_to_client").await
}

// When the subscribed QOS is 1, we need to keep retrying to send the message to the client.
// To avoid messages that are not successfully pushed to the client. When the client Session expires,
// the push thread will exit automatically and will not attempt to push again.
pub async fn exclusive_publish_message_qos1(
    metadata_cache: &Arc<MQTTCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_puback_sx: &broadcast::Sender<QosAckPackageData>,
) -> ResultMqttBrokerError {
    // 1. send Publish to Client
    push_packet_to_client(metadata_cache, connection_manager, sub_pub_param, stop_sx).await?;

    // 2. wait PubAck ack
    wait_pub_ack(
        metadata_cache,
        connection_manager,
        sub_pub_param,
        stop_sx,
        wait_puback_sx,
    )
    .await?;

    Ok(())
}

// send publish message
// wait pubrec message
// send pubrel message
// wait pubcomp message
pub async fn exclusive_publish_message_qos2(
    metadata_cache: &Arc<MQTTCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_ack_sx: &broadcast::Sender<QosAckPackageData>,
) -> ResultMqttBrokerError {
    // 1. send Publish to Client
    push_packet_to_client(metadata_cache, connection_manager, sub_pub_param, stop_sx).await?;

    // 2. wait PubRec ack
    wait_pub_rec(
        metadata_cache,
        connection_manager,
        sub_pub_param,
        stop_sx,
        wait_ack_sx,
    )
    .await?;

    // 3. send PubRel to Client
    qos2_send_pubrel(metadata_cache, sub_pub_param, connection_manager, stop_sx).await?;

    // 4. wait PubComp ack
    wait_pub_comp(
        metadata_cache,
        connection_manager,
        sub_pub_param,
        stop_sx,
        wait_ack_sx,
    )
    .await?;
    Ok(())
}

pub async fn wait_packet_ack(
    sx: &Sender<QosAckPackageData>,
    type_name: &str,
    client_id: &str,
) -> Result<Option<QosAckPackageData>, MqttBrokerError> {
    let timeout_ms = 30;
    match timeout(Duration::from_secs(timeout_ms), async {
        (sx.subscribe().recv().await).ok()
    })
    .await
    {
        Ok(Some(data)) => Ok(Some(data)),
        Ok(None) => Ok(None),
        Err(_) => Err(MqttBrokerError::CommonError(format!(
            "Publish message to client {client_id}, wait {type_name} timeout, more than {client_id}s"
        ))),
    }
}

pub async fn send_message_to_client(
    resp: ResponsePackage,
    connection_manager: &Arc<ConnectionManager>,
) -> ResultMqttBrokerError {
    let start = now_mills();
    let protocol =
        if let Some(protocol) = connection_manager.get_connect_protocol(resp.connection_id) {
            protocol
        } else {
            RobustMQProtocol::MQTT3
        };

    let packet = resp.packet.get_mqtt_packet().unwrap();
    let response = build_mqtt_packet_wrapper(protocol.clone(), packet.clone());

    if connection_manager.is_websocket(resp.connection_id) {
        let mut codec = MqttCodec::new(Some(protocol.to_u8()));
        let mut buff = BytesMut::new();
        if let Err(e) = codec.encode_data(response.to_mqtt(), &mut buff) {
            return Err(MqttBrokerError::WebsocketEncodePacketFailed(e.to_string()));
        }
        connection_manager
            .write_websocket_frame(resp.connection_id, response, Message::Binary(buff.to_vec()))
            .await?;
    } else if connection_manager.is_quic(resp.connection_id) {
        connection_manager
            .write_quic_frame(resp.connection_id, response)
            .await?;
    } else {
        connection_manager
            .write_tcp_frame(resp.connection_id, response)
            .await?
    }

    let network_type = connection_manager.get_network_type(resp.connection_id);
    let wrapper = MqttPacketWrapper {
        protocol_version: protocol.to_u8(),
        packet: packet.clone(),
    };
    if let MqttPacket::Publish(publish, _) = packet.clone() {
        let topic_name = String::from_utf8(publish.topic.to_vec()).unwrap();
        record_mqtt_messages_sent_inc(topic_name.clone());
        record_mqtt_message_bytes_sent(topic_name.clone(), publish.payload.len() as u64);
    }
    if let Some(network) = network_type.clone() {
        record_mqtt_packet_send_duration(
            network,
            mqtt_packet_to_string(&packet),
            (now_mills() - start) as f64,
        );
    }
    record_sent_metrics(&wrapper, format!("{:?}", network_type));
    Ok(())
}

pub async fn wait_pub_ack(
    metadata_cache: &Arc<MQTTCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_ack_sx: &broadcast::Sender<QosAckPackageData>,
) -> ResultMqttBrokerError {
    let wait_pub_ack_fn = async || -> ResultMqttBrokerError {
        let mut wait_ack_rx = wait_ack_sx.subscribe();
        loop {
            let package = wait_ack_rx.recv().await?;
            if package.ack_type == QosAckPackageType::PubAck && package.pkid == sub_pub_param.pkid {
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;
        }
    };

    let ac_fn = async || -> ResultMqttBrokerError {
        loop {
            if timeout(Duration::from_secs(5), wait_pub_ack_fn())
                .await
                .is_err()
            {
                push_packet_to_client(metadata_cache, connection_manager, sub_pub_param, stop_sx)
                    .await?;
                continue;
            }
            break;
        }
        Ok(())
    };

    retry_tool_fn_timeout(ac_fn, stop_sx, "wait_pub_ack").await
}

pub async fn wait_pub_rec(
    metadata_cache: &Arc<MQTTCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_rec_sx: &broadcast::Sender<QosAckPackageData>,
) -> ResultMqttBrokerError {
    let wait_pub_rec_fn = async || -> ResultMqttBrokerError {
        let mut wait_ack_rx = wait_rec_sx.subscribe();
        loop {
            let package = wait_ack_rx.recv().await?;
            if package.ack_type == QosAckPackageType::PubRec && package.pkid == sub_pub_param.pkid {
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;
        }
    };

    let ac_fn = async || -> ResultMqttBrokerError {
        loop {
            if timeout(Duration::from_secs(5), wait_pub_rec_fn())
                .await
                .is_err()
            {
                push_packet_to_client(metadata_cache, connection_manager, sub_pub_param, stop_sx)
                    .await?;
                continue;
            }
            break;
        }
        Ok(())
    };

    retry_tool_fn_timeout(ac_fn, stop_sx, "wait_pub_rec").await
}

pub async fn wait_pub_comp(
    metadata_cache: &Arc<MQTTCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_comp_sx: &broadcast::Sender<QosAckPackageData>,
) -> ResultMqttBrokerError {
    let wait_pub_rec_fn = async || -> ResultMqttBrokerError {
        let mut wait_ack_rx = wait_comp_sx.subscribe();
        loop {
            let package = wait_ack_rx.recv().await?;
            if package.ack_type == QosAckPackageType::PubComp && package.pkid == sub_pub_param.pkid
            {
                return Ok(());
            }
            sleep(Duration::from_secs(1)).await;
        }
    };

    let ac_fn = async || -> ResultMqttBrokerError {
        loop {
            if timeout(Duration::from_secs(5), wait_pub_rec_fn())
                .await
                .is_err()
            {
                qos2_send_pubrel(metadata_cache, sub_pub_param, connection_manager, stop_sx)
                    .await?;
                continue;
            }
            break;
        }
        Ok(())
    };

    retry_tool_fn_timeout(ac_fn, stop_sx, "wait_pub_comp").await
}

pub async fn qos2_send_pubrel(
    metadata_cache: &Arc<MQTTCacheManager>,
    sub_pub_param: &SubPublishParam,
    connection_manager: &Arc<ConnectionManager>,
    stop_sx: &broadcast::Sender<bool>,
) -> ResultMqttBrokerError {
    let mut new_sub_pub_param = sub_pub_param.to_owned();

    let pubrel = PubRel {
        pkid: sub_pub_param.pkid,
        reason: Some(protocol::mqtt::common::PubRelReason::Success),
    };
    new_sub_pub_param.packet = MqttPacket::PubRel(pubrel, None);

    push_packet_to_client(
        metadata_cache,
        connection_manager,
        &new_sub_pub_param,
        stop_sx,
    )
    .await
}

async fn retry_tool_fn_timeout<F, Fut>(
    ac_fn: F,
    stop_sx: &broadcast::Sender<bool>,
    action: &str,
) -> ResultMqttBrokerError
where
    F: FnOnce() -> Fut + Copy,
    Fut: Future<Output = ResultMqttBrokerError>,
{
    let to = 3;
    match timeout(Duration::from_secs(to), retry_tool_fn(ac_fn, stop_sx)).await {
        Ok(res) => res?,
        Err(_) => return Err(MqttBrokerError::OperationTimeout(to, action.to_string())),
    }
    Ok(())
}

async fn retry_tool_fn<F, Fut>(ac_fn: F, stop_sx: &broadcast::Sender<bool>) -> ResultMqttBrokerError
where
    F: FnOnce() -> Fut + Copy,
    Fut: Future<Output = ResultMqttBrokerError>,
{
    let mut stop_recv = stop_sx.subscribe();
    loop {
        select! {
            val = stop_recv.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return Ok(());
                    }
                }
            }
            val = ac_fn() => {
                if let Err(e) = val {
                    if broker_not_available(&e.to_string()){
                        return Err(e);
                    }

                    if !is_ignore_push_error(&e){
                        warn!("retry tool fn fail, error message:{}",e);
                        sleep(Duration::from_secs(1)).await;
                        continue;
                    }
                }
                break;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn topic_subscribe_test() {}
}
