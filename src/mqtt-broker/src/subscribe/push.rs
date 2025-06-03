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

use std::sync::Arc;
use std::time::Duration;

use super::common::Subscriber;
use super::common::{get_pkid, min_qos};
use crate::handler::cache::{CacheManager, QosAckPackageData, QosAckPackageType, QosAckPacketInfo};
use crate::handler::error::MqttBrokerError;
use crate::handler::message::is_message_expire;
use crate::handler::sub_option::{get_retain_flag_by_retain_as_published, is_send_msg_by_bo_local};
use crate::observability::slow::sub::{record_slow_sub_data, SlowSubData};
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::ResponsePackage;
use crate::subscribe::common::SubPublishParam;
use axum::extract::ws::Message;
use bytes::{Bytes, BytesMut};
use common_base::tools::{now_mills, now_second};
use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
use protocol::mqtt::common::{MqttPacket, MqttProtocol, PubRel, Publish, PublishProperties, QoS};
use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::{sleep, timeout};
use tracing::debug;

pub async fn build_pub_message(
    record: Record,
    group_id: &str,
    qos: &QoS,
    subscriber: &Subscriber,
    sub_ids: &[usize],
) -> Result<Option<SubPublishParam>, MqttBrokerError> {
    let msg = MqttMessage::decode_record(record.clone())?;

    if is_message_expire(&msg) {
        debug!("Message dropping: message expires, is not pushed to the client, and is discarded");
        return Ok(None);
    }

    if !is_send_msg_by_bo_local(subscriber.nolocal, &subscriber.client_id, &msg.client_id) {
        debug!(
            "Message dropping: message is not pushed to the client, because the client_id is the same as the subscriber, client_id: {}, topic_id: {}",
            subscriber.client_id, subscriber.topic_id
        );
        return Ok(None);
    }

    let retain = get_retain_flag_by_retain_as_published(subscriber.preserve_retain, msg.retain);

    let mut publish = Publish {
        dup: false,
        qos: qos.to_owned(),
        pkid: get_pkid(),
        retain,
        topic: Bytes::from(subscriber.topic_name.clone()),
        payload: msg.payload,
    };

    let properties = PublishProperties {
        payload_format_indicator: msg.format_indicator,
        message_expiry_interval: Some(msg.expiry_interval as u32),
        topic_alias: None,
        response_topic: msg.response_topic,
        correlation_data: msg.correlation_data,
        user_properties: msg.user_properties,
        subscription_identifiers: sub_ids.into(),
        content_type: msg.content_type,
    };

    let pkid = get_pkid();
    publish.pkid = pkid;

    let sub_pub_param = SubPublishParam::new(
        subscriber.clone(),
        publish,
        Some(properties),
        record.timestamp as u128,
        group_id.to_string(),
        pkid,
    );
    Ok(Some(sub_pub_param))
}

pub async fn publish_data(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<CacheManager>,
    sub_pub_param: SubPublishParam,
    stop_sx: &Sender<bool>,
) -> Result<(), MqttBrokerError> {
    match sub_pub_param.publish.qos {
        QoS::AtMostOnce => {
            publish_message_qos(cache_manager, connection_manager, &sub_pub_param, stop_sx).await?;
        }

        QoS::AtLeastOnce => {
            let (wait_puback_sx, _) = broadcast::channel(1);
            let client_id = sub_pub_param.subscribe.client_id.clone();
            let pkid: u16 = sub_pub_param.pkid;
            cache_manager.add_ack_packet(
                &client_id,
                pkid,
                QosAckPacketInfo {
                    sx: wait_puback_sx.clone(),
                    create_time: now_second(),
                },
            );

            exclusive_publish_message_qos1(
                cache_manager,
                connection_manager,
                &sub_pub_param,
                stop_sx,
                &wait_puback_sx,
            )
            .await?;

            cache_manager.remove_ack_packet(&client_id, pkid);
        }

        QoS::ExactlyOnce => {
            let (wait_ack_sx, _) = broadcast::channel(1);
            let client_id = sub_pub_param.subscribe.client_id.clone();
            let pkid = sub_pub_param.pkid;
            cache_manager.add_ack_packet(
                &client_id,
                pkid,
                QosAckPacketInfo {
                    sx: wait_ack_sx.clone(),
                    create_time: now_second(),
                },
            );

            exclusive_publish_message_qos2(
                cache_manager,
                connection_manager,
                &sub_pub_param,
                stop_sx,
                &wait_ack_sx,
            )
            .await?;

            cache_manager.remove_ack_packet(&client_id, pkid);
        }
    }
    Ok(())
}

pub fn build_pub_qos(cache_manager: &Arc<CacheManager>, subscriber: &Subscriber) -> QoS {
    let cluster_qos = cache_manager.get_cluster_info().protocol.max_qos;
    min_qos(cluster_qos, subscriber.qos)
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
#[allow(clippy::question_mark)]
pub async fn publish_message_qos(
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
) -> Result<(), MqttBrokerError> {
    let mut stop_recv = stop_sx.subscribe();
    let mut fail = None;
    let mut times = 0;
    loop {
        let client_id = sub_pub_param.subscribe.client_id.clone();
        let push_to_connect = || async move {
            if metadata_cache.get_session_info(&client_id).is_none() {
                return Err(MqttBrokerError::SessionNullSkipPushMessage(client_id));
            }

            let connect_id_op = metadata_cache.get_connect_id(&client_id);
            if connect_id_op.is_none() {
                return Err(MqttBrokerError::ConnectionNullSkipPushMessage(client_id));
            }

            let connect_id = connect_id_op.unwrap();

            if let Some(conn) = metadata_cache.get_connection(connect_id) {
                if sub_pub_param.publish.payload.len() > (conn.max_packet_size as usize) {
                    return Err(MqttBrokerError::CommonError(format!(
                        "Client {}, the size of the subscription sent packets exceeds the limit. Packet size :{}, Limit size :{}",
                        client_id,sub_pub_param.publish.payload.len(), conn.max_packet_size
                    )));
                }
            }

            let mut contain_properties = false;
            if let Some(protocol) = connection_manager.get_connect_protocol(connect_id) {
                if MqttProtocol::is_mqtt5(&protocol) {
                    contain_properties = true;
                }
            }

            let resp = if contain_properties {
                ResponsePackage {
                    connection_id: connect_id,
                    packet: MqttPacket::Publish(
                        sub_pub_param.publish.clone(),
                        sub_pub_param.properties.clone(),
                    ),
                }
            } else {
                ResponsePackage {
                    connection_id: connect_id,
                    packet: MqttPacket::Publish(sub_pub_param.publish.clone(), None),
                }
            };

            // 2. publish to mqtt client
            if let Err(e) = publish_message_to_client(
                resp.clone(),
                sub_pub_param,
                connection_manager,
                metadata_cache,
            )
            .await
            {
                return Err(e);
            }
            Ok(())
        };

        select! {
            val = stop_recv.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return Ok(());
                    }
                }
            }
            val = push_to_connect() => {
                if let Err(e) = val{
                    if times > 3 {
                        fail = Some(e);
                        break;
                    }
                    times += 1;
                    sleep(Duration::from_secs(1)).await;
                    continue;
                }
                break;
            }
        }
    }
    if let Some(e) = fail {
        return Err(e);
    }
    Ok(())
}

// When the subscribed QOS is 1, we need to keep retrying to send the message to the client.
// To avoid messages that are not successfully pushed to the client. When the client Session expires,
// the push thread will exit automatically and will not attempt to push again.
pub async fn exclusive_publish_message_qos1(
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_puback_sx: &broadcast::Sender<QosAckPackageData>,
) -> Result<(), MqttBrokerError> {
    // 1. send Publish to Client
    publish_message_qos(metadata_cache, connection_manager, sub_pub_param, stop_sx).await?;

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
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_ack_sx: &broadcast::Sender<QosAckPackageData>,
) -> Result<(), MqttBrokerError> {
    // 1. send Publish to Client
    publish_message_qos(metadata_cache, connection_manager, sub_pub_param, stop_sx).await?;

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
            "Publish message to client {}, wait {} timeout, more than {}s",
            client_id, type_name, client_id
        ))),
    }
}

pub async fn publish_message_to_client(
    resp: ResponsePackage,
    sub_pub_param: &SubPublishParam,
    connection_manager: &Arc<ConnectionManager>,
    metadata_cache: &Arc<CacheManager>,
) -> Result<(), MqttBrokerError> {
    if let Some(protocol) = connection_manager.get_connect_protocol(resp.connection_id) {
        let response: MqttPacketWrapper = MqttPacketWrapper {
            protocol_version: protocol.clone().into(),
            packet: resp.packet,
        };

        if connection_manager.is_websocket(resp.connection_id) {
            let mut codec = MqttCodec::new(Some(protocol.into()));
            let mut buff = BytesMut::new();
            if let Err(e) = codec.encode_data(response.clone(), &mut buff) {
                return Err(MqttBrokerError::WebsocketEncodePacketFailed(e.to_string()));
            }
            connection_manager
                .write_websocket_frame(resp.connection_id, response, Message::Binary(buff.to_vec()))
                .await?;
        } else {
            connection_manager
                .write_tcp_frame(resp.connection_id, response)
                .await?
        }

        // record slow sub data
        if metadata_cache.get_slow_sub_config().enable && sub_pub_param.create_time > 0 {
            let slow_data = SlowSubData::build(
                sub_pub_param.subscribe.sub_path.clone(),
                sub_pub_param.subscribe.client_id.clone(),
                sub_pub_param.subscribe.topic_name.clone(),
                (now_mills() - sub_pub_param.create_time) as u64,
            );
            record_slow_sub_data(slow_data, metadata_cache.get_slow_sub_config().whole_ms)?;
        }
    }

    Ok(())
}

pub async fn wait_pub_ack(
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_ack_sx: &broadcast::Sender<QosAckPackageData>,
) -> Result<(), MqttBrokerError> {
    let wait_pub_ack_fn = || async {
        match wait_packet_ack(wait_ack_sx, "PubAck", &sub_pub_param.subscribe.client_id).await {
            Ok(Some(data)) => {
                if data.ack_type == QosAckPackageType::PubAck && data.pkid == sub_pub_param.pkid {
                    return Ok(());
                }
            }
            Ok(None) => {}
            Err(e) => {
                publish_message_qos(metadata_cache, connection_manager, sub_pub_param, stop_sx)
                    .await?;
                return Err(e);
            }
        };

        Err(MqttBrokerError::CommonError(
            "Send message to the client did not receive a correct PubAck return".to_owned(),
        ))
    };

    wait_pub_ack_fn().await
}

pub async fn wait_pub_rec(
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_ack_sx: &broadcast::Sender<QosAckPackageData>,
) -> Result<(), MqttBrokerError> {
    let wait_pub_rec_fn = || async {
        match wait_packet_ack(wait_ack_sx, "PubRec", &sub_pub_param.subscribe.client_id).await {
            Ok(Some(data)) => {
                if data.ack_type == QosAckPackageType::PubRec && data.pkid == sub_pub_param.pkid {
                    return Ok(());
                }
            }
            Ok(None) => {}
            Err(e) => {
                publish_message_qos(metadata_cache, connection_manager, sub_pub_param, stop_sx)
                    .await?;
                return Err(e);
            }
        };

        Err(MqttBrokerError::CommonError(
            "Send message to the client did not receive a correct PubRec return".to_owned(),
        ))
    };

    wait_pub_rec_fn().await
}

pub async fn wait_pub_comp(
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    wait_ack_sx: &broadcast::Sender<QosAckPackageData>,
) -> Result<(), MqttBrokerError> {
    let wait_pub_comp_fn = || async {
        match wait_packet_ack(wait_ack_sx, "PubComp", &sub_pub_param.subscribe.client_id).await {
            Ok(Some(data)) => {
                if data.ack_type == QosAckPackageType::PubComp && data.pkid == sub_pub_param.pkid {
                    return Ok(());
                }
            }
            Ok(None) => {}
            Err(e) => {
                qos2_send_pubrel(metadata_cache, sub_pub_param, connection_manager, stop_sx)
                    .await?;
                return Err(e);
            }
        };

        Err(MqttBrokerError::CommonError(
            "Send message to the client did not receive a correct PubComp return".to_owned(),
        ))
    };

    wait_pub_comp_fn().await
}

pub async fn qos2_send_pubrel(
    metadata_cache: &Arc<CacheManager>,
    sub_pub_param: &SubPublishParam,
    connection_manager: &Arc<ConnectionManager>,
    stop_sx: &broadcast::Sender<bool>,
) -> Result<(), MqttBrokerError> {
    let mut stop_rx = stop_sx.subscribe();

    loop {
        let connect_id =
            if let Some(id) = metadata_cache.get_connect_id(&sub_pub_param.subscribe.client_id) {
                id
            } else {
                sleep(Duration::from_secs(1)).await;
                continue;
            };

        let pubrel = PubRel {
            pkid: sub_pub_param.pkid,
            reason: Some(protocol::mqtt::common::PubRelReason::Success),
        };

        let pubrel_resp = ResponsePackage {
            connection_id: connect_id,
            packet: MqttPacket::PubRel(pubrel, None),
        };

        select! {
            val = stop_rx.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        return Ok(());
                    }
                }
            }

            val = publish_message_to_client(
                pubrel_resp.clone(),
                sub_pub_param,
                connection_manager,
                metadata_cache
            ) =>{
                return val;
            }

        }
    }
}
