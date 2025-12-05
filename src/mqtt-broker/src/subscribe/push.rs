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
use crate::handler::metrics::record_publish_send_metrics;
use crate::handler::metrics::record_send_metrics;
use crate::handler::sub_option::{get_retain_flag_by_retain_as_published, is_send_msg_by_bo_local};
use crate::subscribe::common::{is_ignore_push_error, SubPublishParam};
use axum::extract::ws::Message;
use bytes::{Bytes, BytesMut};
use common_base::network::broker_not_available;
use common_base::tools::now_mills;
use common_base::tools::now_second;
use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::message::MqttMessage;
use network_server::common::connection_manager::ConnectionManager;
use network_server::common::packet::build_mqtt_packet_wrapper;
use network_server::common::packet::ResponsePackage;
use protocol::mqtt::codec::MqttCodec;
use protocol::mqtt::codec::MqttPacketWrapper;
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

// Timeout constants
const ACK_WAIT_TIMEOUT_SECS: u64 = 5;
const RETRY_OPERATION_TIMEOUT_SECS: u64 = 3;
const RETRY_SLEEP_INTERVAL_MS: u64 = 200;
const RETRY_SLEEP_ITERATIONS: usize = 5; // 5 * 200ms = 1000ms

pub async fn build_publish_message(
    cache_manager: &Arc<MQTTCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    record: &Record,
    subscriber: &Subscriber,
) -> Result<Option<SubPublishParam>, MqttBrokerError> {
    let msg = MqttMessage::decode_record(record.clone())?;

    // Early exit for expired messages
    if is_message_expire(&msg) {
        debug!(
            "Message dropping: message expired, client_id: {}, topic: {}",
            subscriber.client_id, subscriber.topic_name
        );
        return Ok(None);
    }

    // Early exit for no_local constraint
    if !is_send_msg_by_bo_local(subscriber.no_local, &subscriber.client_id, &msg.client_id) {
        debug!(
            "Message dropping: no_local constraint, client_id: {}, topic: {}",
            subscriber.client_id, subscriber.topic_name
        );
        return Ok(None);
    }

    let connect_id = cache_manager
        .get_connect_id(&subscriber.client_id)
        .ok_or_else(|| {
            MqttBrokerError::ConnectionNullSkipPushMessage(subscriber.client_id.to_owned())
        })?;

    // Check max packet size constraint
    if let Some(conn) = cache_manager.get_connection(connect_id) {
        if msg.payload.len() > (conn.max_packet_size as usize) {
            debug!(
                "Message dropping: payload size {} exceeds max packet size {}, client_id: {}",
                msg.payload.len(),
                conn.max_packet_size,
                subscriber.client_id
            );
            return Ok(None);
        }
    }

    // Determine if MQTT5 properties should be included
    let contain_properties = connection_manager
        .get_connect_protocol(connect_id)
        .map(|p| p.is_mqtt5())
        .unwrap_or(false);

    let qos = build_pub_qos(cache_manager, subscriber).await;
    let p_kid = cache_manager
        .pkid_metadata
        .generate_pkid(&subscriber.client_id, &qos)
        .await;

    let retain = get_retain_flag_by_retain_as_published(subscriber.preserve_retain, msg.retain);
    let sub_ids = build_sub_ids(subscriber);

    let publish = Publish {
        dup: false,
        qos,
        p_kid,
        retain,
        topic: Bytes::copy_from_slice(subscriber.topic_name.as_bytes()),
        payload: msg.payload,
    };

    let properties = if contain_properties {
        Some(PublishProperties {
            payload_format_indicator: msg.format_indicator,
            message_expiry_interval: Some(msg.expiry_interval as u32),
            topic_alias: None,
            response_topic: msg.response_topic,
            correlation_data: msg.correlation_data,
            user_properties: msg.user_properties.unwrap_or_default(),
            subscription_identifiers: sub_ids,
            content_type: msg.content_type,
        })
    } else {
        None
    };

    let packet = MqttPacket::Publish(publish, properties);
    Ok(Some(SubPublishParam {
        packet,
        create_time: now_second(),
        client_id: subscriber.client_id.to_string(),
        p_kid,
        qos,
    }))
}

pub async fn send_publish_packet_to_client(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<MQTTCacheManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &Sender<bool>,
) -> ResultMqttBrokerError {
    match sub_pub_param.qos {
        QoS::AtMostOnce => {
            push_packet_to_client(cache_manager, connection_manager, sub_pub_param, stop_sx).await
        }

        QoS::AtLeastOnce => {
            let (wait_puback_sx, _) = broadcast::channel(1);
            let pkid = sub_pub_param.p_kid;
            cache_manager.pkid_metadata.add_ack_packet(
                &sub_pub_param.client_id,
                pkid,
                QosAckPacketInfo {
                    sx: wait_puback_sx.clone(),
                    create_time: now_mills(),
                },
            );

            let result = exclusive_publish_message_qos1(
                cache_manager,
                connection_manager,
                sub_pub_param,
                stop_sx,
                &wait_puback_sx,
            )
            .await;

            cache_manager
                .pkid_metadata
                .remove_ack_packet(&sub_pub_param.client_id, pkid);

            result
        }

        QoS::ExactlyOnce => {
            let (wait_ack_sx, _) = broadcast::channel(1);
            let pkid = sub_pub_param.p_kid;
            cache_manager.pkid_metadata.add_ack_packet(
                &sub_pub_param.client_id,
                pkid,
                QosAckPacketInfo {
                    sx: wait_ack_sx.clone(),
                    create_time: now_mills(),
                },
            );

            let result = exclusive_publish_message_qos2(
                cache_manager,
                connection_manager,
                sub_pub_param,
                stop_sx,
                &wait_ack_sx,
            )
            .await;

            cache_manager
                .pkid_metadata
                .remove_ack_packet(&sub_pub_param.client_id, pkid);

            result
        }
    }
}

pub async fn build_pub_qos(cache_manager: &Arc<MQTTCacheManager>, subscriber: &Subscriber) -> QoS {
    let cluster_qos = cache_manager
        .broker_cache
        .get_cluster_config()
        .await
        .mqtt_protocol_config
        .max_qos;

    let cluster_qos_level = qos(cluster_qos).unwrap_or(QoS::ExactlyOnce);
    min_qos(cluster_qos_level, subscriber.qos)
}

pub fn build_sub_ids(subscriber: &Subscriber) -> Vec<usize> {
    subscriber.subscription_identifier.into_iter().collect()
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
        if cache_manager
            .get_session_info(&sub_pub_param.client_id)
            .is_none()
        {
            return Err(MqttBrokerError::SessionNullSkipPushMessage(
                sub_pub_param.client_id.clone(),
            ));
        }

        let connect_id = cache_manager
            .get_connect_id(&sub_pub_param.client_id)
            .ok_or_else(|| {
                MqttBrokerError::ConnectionNullSkipPushMessage(sub_pub_param.client_id.clone())
            })?;

        let packet = RobustMQPacket::MQTT(sub_pub_param.packet.clone());
        let resp = ResponsePackage::new(connect_id, packet, 0, 0, 0, "Subscribe".to_string());

        send_message_to_client(resp, connection_manager, cache_manager).await
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

pub async fn send_message_to_client(
    resp: ResponsePackage,
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<MQTTCacheManager>,
) -> ResultMqttBrokerError {
    let start = now_mills();
    let protocol =
        if let Some(protocol) = connection_manager.get_connect_protocol(resp.connection_id) {
            protocol
        } else {
            RobustMQProtocol::MQTT3
        };

    let packet = resp
        .packet
        .get_mqtt_packet()
        .ok_or_else(|| MqttBrokerError::CommonError("Failed to get MQTT packet".to_string()))?;

    let response = build_mqtt_packet_wrapper(protocol.clone(), packet.clone());

    // Send based on connection type
    match (
        connection_manager.is_websocket(resp.connection_id),
        connection_manager.is_quic(resp.connection_id),
    ) {
        (true, _) => {
            let mut codec = MqttCodec::new(Some(protocol.to_u8()));
            let mut buff = BytesMut::new();
            if let Err(e) = codec.encode_data(response.to_mqtt(), &mut buff) {
                return Err(MqttBrokerError::WebsocketEncodePacketFailed(e.to_string()));
            }
            connection_manager
                .write_websocket_frame(resp.connection_id, response, Message::Binary(buff.to_vec()))
                .await?;
        }
        (false, true) => {
            connection_manager
                .write_quic_frame(resp.connection_id, response)
                .await?;
        }
        (false, false) => {
            connection_manager
                .write_tcp_frame(resp.connection_id, response)
                .await?;
        }
    }

    // Record metrics for Publish packets
    if let MqttPacket::Publish(publish, _) = &packet {
        if let Ok(topic_name) = String::from_utf8(publish.topic.to_vec()) {
            if let Some(connection) = cache_manager.get_connection(resp.connection_id) {
                record_publish_send_metrics(
                    &connection.client_id,
                    resp.connection_id,
                    &topic_name,
                    publish.payload.len() as u64,
                );
            }
        }
    }

    let network_type = connection_manager.get_network_type(resp.connection_id);
    let wrapper = MqttPacketWrapper {
        protocol_version: protocol.to_u8(),
        packet,
    };

    record_send_metrics(&wrapper, &wrapper.packet, network_type, start);
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
            if package.ack_type == QosAckPackageType::PubAck && package.pkid == sub_pub_param.p_kid
            {
                return Ok(());
            }
        }
    };

    let ac_fn = async || -> ResultMqttBrokerError {
        loop {
            if timeout(
                Duration::from_secs(ACK_WAIT_TIMEOUT_SECS),
                wait_pub_ack_fn(),
            )
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
            if package.ack_type == QosAckPackageType::PubRec && package.pkid == sub_pub_param.p_kid
            {
                return Ok(());
            }
        }
    };

    let ac_fn = async || -> ResultMqttBrokerError {
        loop {
            if timeout(
                Duration::from_secs(ACK_WAIT_TIMEOUT_SECS),
                wait_pub_rec_fn(),
            )
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
    let wait_pub_comp_fn = async || -> ResultMqttBrokerError {
        let mut wait_ack_rx = wait_comp_sx.subscribe();
        loop {
            let package = wait_ack_rx.recv().await?;
            if package.ack_type == QosAckPackageType::PubComp && package.pkid == sub_pub_param.p_kid
            {
                return Ok(());
            }
        }
    };

    let ac_fn = async || -> ResultMqttBrokerError {
        loop {
            if timeout(
                Duration::from_secs(ACK_WAIT_TIMEOUT_SECS),
                wait_pub_comp_fn(),
            )
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
    let pubrel = PubRel {
        pkid: sub_pub_param.p_kid,
        reason: Some(protocol::mqtt::common::PubRelReason::Success),
    };

    let pubrel_param = SubPublishParam {
        packet: MqttPacket::PubRel(pubrel, None),
        create_time: sub_pub_param.create_time,
        client_id: sub_pub_param.client_id.clone(),
        p_kid: sub_pub_param.p_kid,
        qos: sub_pub_param.qos,
    };

    push_packet_to_client(metadata_cache, connection_manager, &pubrel_param, stop_sx).await
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
    timeout(
        Duration::from_secs(RETRY_OPERATION_TIMEOUT_SECS),
        retry_tool_fn(ac_fn, stop_sx),
    )
    .await
    .map_err(|_| {
        MqttBrokerError::OperationTimeout(RETRY_OPERATION_TIMEOUT_SECS, action.to_string())
    })?
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

                        // Sleep with interruptible intervals for better responsiveness
                        if interruptible_sleep(&mut stop_recv, RETRY_SLEEP_ITERATIONS).await.is_err() {
                            return Ok(());
                        }
                        continue;
                    }
                }
                break;
            }
        }
    }
    Ok(())
}

/// Sleep in small intervals while checking for stop signal
async fn interruptible_sleep(
    stop_recv: &mut broadcast::Receiver<bool>,
    iterations: usize,
) -> Result<(), ()> {
    for _ in 0..iterations {
        select! {
            val = stop_recv.recv() => {
                if let Ok(true) = val {
                    return Err(());
                }
            }
            _ = sleep(Duration::from_millis(RETRY_SLEEP_INTERVAL_MS)) => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::tool::test_build_mqtt_cache_manager;
    use crate::subscribe::common::Subscriber;
    use common_base::tools::now_second;
    use protocol::mqtt::common::{MqttProtocol, QoS, RetainHandling};
    use std::time::Instant;

    fn create_test_subscriber(
        client_id: &str,
        subscription_identifier: Option<usize>,
        qos: QoS,
    ) -> Subscriber {
        Subscriber {
            client_id: client_id.to_string(),
            sub_path: "/test/topic".to_string(),
            rewrite_sub_path: None,
            topic_name: "/test/topic".to_string(),
            group_name: "test_group".to_string(),
            protocol: MqttProtocol::Mqtt5,
            qos,
            no_local: false,
            preserve_retain: false,
            retain_forward_rule: RetainHandling::OnEverySubscribe,
            subscription_identifier,
            create_time: now_second(),
        }
    }

    #[test]
    fn test_build_sub_ids_with_identifier() {
        let subscriber = create_test_subscriber("client1", Some(123), QoS::AtLeastOnce);
        assert_eq!(build_sub_ids(&subscriber), vec![123]);
    }

    #[test]
    fn test_build_sub_ids_without_identifier() {
        let subscriber = create_test_subscriber("client1", None, QoS::AtLeastOnce);
        assert_eq!(build_sub_ids(&subscriber), Vec::<usize>::new());
    }

    #[test]
    fn test_build_sub_ids_with_zero_identifier() {
        let subscriber = create_test_subscriber("client1", Some(0), QoS::AtLeastOnce);
        assert_eq!(build_sub_ids(&subscriber), vec![0]);
    }

    #[tokio::test]
    async fn test_build_pub_qos_min_selection() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let subscriber = create_test_subscriber("client1", None, QoS::ExactlyOnce);
        let result = build_pub_qos(&cache_manager, &subscriber).await;
        assert!(matches!(result, QoS::AtLeastOnce | QoS::ExactlyOnce));
    }

    #[tokio::test]
    async fn test_build_pub_qos_subscriber_lower() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let subscriber = create_test_subscriber("client1", None, QoS::AtMostOnce);
        assert_eq!(
            build_pub_qos(&cache_manager, &subscriber).await,
            QoS::AtMostOnce
        );
    }

    #[tokio::test]
    async fn test_build_pub_qos_same_level() {
        let cache_manager = test_build_mqtt_cache_manager().await;
        let subscriber = create_test_subscriber("client1", None, QoS::AtLeastOnce);
        assert_eq!(
            build_pub_qos(&cache_manager, &subscriber).await,
            QoS::AtLeastOnce
        );
    }

    #[tokio::test]
    async fn test_interruptible_sleep_completes_normally() {
        let (_stop_sx, mut stop_rx) = broadcast::channel(1);
        let start = Instant::now();
        let result = interruptible_sleep(&mut stop_rx, RETRY_SLEEP_ITERATIONS).await;

        assert!(result.is_ok());
        assert!(start.elapsed().as_millis() >= 1000);
    }

    #[tokio::test]
    async fn test_interruptible_sleep_interrupted_early() {
        let (stop_sx, mut stop_rx) = broadcast::channel(1);
        let start = Instant::now();

        tokio::spawn(async move {
            sleep(Duration::from_millis(400)).await;
            let _ = stop_sx.send(true);
        });

        let result = interruptible_sleep(&mut stop_rx, RETRY_SLEEP_ITERATIONS).await;
        assert!(result.is_err());
        assert!(start.elapsed().as_millis() < 1000);
    }

    #[tokio::test]
    async fn test_interruptible_sleep_immediate_stop() {
        let (stop_sx, mut stop_rx) = broadcast::channel(1);
        stop_sx.send(true).unwrap();
        assert!(interruptible_sleep(&mut stop_rx, RETRY_SLEEP_ITERATIONS)
            .await
            .is_err());
    }

    #[tokio::test]
    async fn test_interruptible_sleep_false_signal_ignored() {
        let (stop_sx, mut stop_rx) = broadcast::channel(1);

        tokio::spawn(async move {
            sleep(Duration::from_millis(200)).await;
            let _ = stop_sx.send(false);
        });

        assert!(interruptible_sleep(&mut stop_rx, 2).await.is_ok());
    }

    #[tokio::test]
    async fn test_interruptible_sleep_zero_iterations() {
        let (_stop_sx, mut stop_rx) = broadcast::channel(1);
        assert!(interruptible_sleep(&mut stop_rx, 0).await.is_ok());
    }

    #[tokio::test]
    async fn test_interruptible_sleep_responsiveness() {
        let (stop_sx, mut stop_rx) = broadcast::channel(1);
        let start = Instant::now();

        tokio::spawn(async move {
            sleep(Duration::from_millis(50)).await;
            let _ = stop_sx.send(true);
        });

        assert!(interruptible_sleep(&mut stop_rx, RETRY_SLEEP_ITERATIONS)
            .await
            .is_err());
        assert!(start.elapsed().as_millis() <= 250);
    }

    #[test]
    fn test_retry_sleep_iterations_constant() {
        let total_ms = (RETRY_SLEEP_ITERATIONS as u64) * RETRY_SLEEP_INTERVAL_MS;
        assert_eq!(total_ms, 1000);
    }
}
