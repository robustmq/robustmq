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
use crate::core::cache::{
    MQTTCacheManager, QosAckPackageData, QosAckPackageType, QosAckPacketInfo,
};
use crate::core::error::MqttBrokerError;
use crate::core::metrics::record_publish_send_metrics;
use crate::core::metrics::record_send_metrics;
use crate::core::sub_slow::record_slow_subscribe_data;
use crate::core::tool::ResultMqttBrokerError;
use crate::subscribe::common::{client_unavailable_error, SubPublishParam};
use axum::extract::ws::Message;
use bytes::{Bytes, BytesMut};
use common_base::network::broker_not_available;
use common_base::tools::now_millis;
use common_base::tools::now_second;
use metadata_struct::storage::record::StorageRecord;
use network_server::common::connection_manager::ConnectionManager;
use network_server::common::packet::build_mqtt_packet_wrapper;
use network_server::common::packet::ResponsePackage;
use protocol::mqtt::codec::MqttCodec;
use protocol::mqtt::codec::MqttPacketWrapper;
use protocol::mqtt::common::{MqttPacket, PubRel, Publish, PublishProperties, QoS};
use protocol::robust::RobustMQPacket;
use protocol::robust::RobustMQProtocol;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::select;
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout};
use tracing::warn;

// Timeout constants
const ACK_WAIT_TIMEOUT_SECS: u64 = 5;
const RETRY_OPERATION_TIMEOUT_SECS: u64 = 3;
const RETRY_SLEEP_INTERVAL_MS: u64 = 200;
const RETRY_SLEEP_ITERATIONS: usize = 5; // 5 * 200ms = 1000ms
const QOS_ACK_RESEND_MAX_RETRIES: usize = 3;

// Push Config
pub const BATCH_SIZE: u64 = 500;
pub const IDLE_SLEEP_MS: u64 = 100;
pub const LOW_LOAD_SLEEP_MS: u64 = 50;
pub const HIGH_LOAD_SLEEP_MS: u64 = 10;
pub const LOW_LOAD_THRESHOLD: u64 = 10;

pub async fn push_data(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<MQTTCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    subscriber: &Subscriber,
    record: &StorageRecord,
    stop_sx: &Sender<bool>,
) -> Result<bool, MqttBrokerError> {
    let sub_pub_param = if let Some(params) =
        build_publish_message(cache_manager, connection_manager, record, subscriber).await?
    {
        params
    } else {
        return Ok(false);
    };

    send_publish_packet_to_client(connection_manager, cache_manager, &sub_pub_param, stop_sx)
        .await?;

    record_slow_subscribe_data(
        cache_manager,
        rocksdb_engine_handler,
        subscriber,
        now_second(),
        record.metadata.create_t,
    )
    .await?;

    Ok(true)
}

pub async fn build_publish_message(
    cache_manager: &Arc<MQTTCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    msg: &StorageRecord,
    subscriber: &Subscriber,
) -> Result<Option<SubPublishParam>, MqttBrokerError> {
    let connect_id = cache_manager
        .get_connect_id(&subscriber.client_id)
        .ok_or_else(|| {
            MqttBrokerError::ConnectionNullSkipPushMessage(subscriber.client_id.to_owned())
        })?;

    let qos = build_pub_qos(subscriber).await;
    let p_kid = cache_manager
        .pkid_manager
        .generate_publish_to_client_pkid(&subscriber.client_id, &qos)
        .await;

    let retain = build_retain_flag(msg, subscriber.preserve_retain);
    let publish = Publish {
        dup: false,
        qos,
        p_kid,
        retain,
        topic: Bytes::copy_from_slice(subscriber.topic_name.as_bytes()),
        payload: msg.data.clone(),
    };

    let properties = build_publish_properties(connection_manager, msg, connect_id, subscriber);
    let packet = MqttPacket::Publish(publish, properties);
    Ok(Some(SubPublishParam {
        packet,
        create_time: now_second(),
        client_id: subscriber.client_id.to_string(),
        p_kid,
        qos,
    }))
}

fn build_retain_flag(msg: &StorageRecord, preserve_retain: bool) -> bool {
    if !preserve_retain {
        return false;
    }

    if let Some(protocol_data) = msg.protocol_data.clone() {
        if let Some(mqtt_data) = protocol_data.mqtt {
            return mqtt_data.retain;
        }
    }

    false
}

fn build_publish_properties(
    connection_manager: &Arc<ConnectionManager>,
    msg: &StorageRecord,
    connect_id: u64,
    subscriber: &Subscriber,
) -> Option<PublishProperties> {
    let contain_properties = connection_manager
        .get_connect_protocol(connect_id)
        .map(|p| p.is_mqtt5())
        .unwrap_or(false);
    if !contain_properties {
        return None;
    }

    let mut user_properties = Vec::new();
    if let Some(header) = msg.metadata.header.clone() {
        for row in header {
            user_properties.push((row.name, row.value));
        }
    }

    let mut properties = PublishProperties::default();
    if let Some(protocol_data) = msg.protocol_data.clone() {
        if let Some(mqtt_data) = protocol_data.mqtt {
            user_properties.extend(mqtt_data.user_properties);
            properties = PublishProperties {
                payload_format_indicator: mqtt_data.format_indicator,
                topic_alias: None,
                response_topic: mqtt_data.response_topic.clone(),
                correlation_data: mqtt_data.correlation_data.clone(),
                content_type: mqtt_data.content_type.clone(),
                user_properties: user_properties.clone(),
                ..Default::default()
            };
        }
    }
    properties.user_properties = user_properties;
    properties.subscription_identifiers = subscriber
        .subscription_identifier
        .into_iter()
        .collect();

    Some(properties)
}

pub async fn send_publish_packet_to_client(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<MQTTCacheManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
) -> ResultMqttBrokerError {
    match sub_pub_param.qos {
        QoS::AtMostOnce => {
            push_packet_to_client(cache_manager, connection_manager, sub_pub_param, stop_sx).await
        }

        QoS::AtLeastOnce => {
            let (wait_puback_sx, wait_ack_rx) = mpsc::channel(1);
            let pkid = sub_pub_param.p_kid;
            cache_manager
                .pkid_manager
                .add_publish_to_client_qos_ack_data(
                    &sub_pub_param.client_id,
                    pkid,
                    QosAckPacketInfo {
                        sx: wait_puback_sx.clone(),
                        create_time: now_millis(),
                    },
                );

            let result = exclusive_publish_message_qos1(
                cache_manager,
                connection_manager,
                sub_pub_param,
                stop_sx,
                wait_ack_rx,
            )
            .await;

            cache_manager
                .pkid_manager
                .remove_publish_to_client_pkid(&sub_pub_param.client_id, pkid);

            result
        }

        QoS::ExactlyOnce => {
            let (wait_ack_sx, wait_ack_rx) = mpsc::channel(1);
            let pkid = sub_pub_param.p_kid;
            cache_manager
                .pkid_manager
                .add_publish_to_client_qos_ack_data(
                    &sub_pub_param.client_id,
                    pkid,
                    QosAckPacketInfo {
                        sx: wait_ack_sx.clone(),
                        create_time: now_millis(),
                    },
                );

            let result = exclusive_publish_message_qos2(
                cache_manager,
                connection_manager,
                sub_pub_param,
                stop_sx,
                wait_ack_rx,
            )
            .await;

            cache_manager
                .pkid_manager
                .remove_publish_to_client_pkid(&sub_pub_param.client_id, pkid);
            result
        }
    }
}

async fn build_pub_qos(subscriber: &Subscriber) -> QoS {
    min_qos(QoS::ExactlyOnce, subscriber.qos)
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
        let resp = ResponsePackage::new(connect_id, packet);

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
    wait_puback_rx: mpsc::Receiver<QosAckPackageData>,
) -> ResultMqttBrokerError {
    // 1. send Publish to Client
    push_packet_to_client(metadata_cache, connection_manager, sub_pub_param, stop_sx).await?;

    // 2. wait PubAck ack
    wait_pub_ack(
        metadata_cache,
        connection_manager,
        sub_pub_param,
        stop_sx,
        wait_puback_rx,
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
    wait_ack_rx: mpsc::Receiver<QosAckPackageData>,
) -> ResultMqttBrokerError {
    // 1. send Publish to Client
    push_packet_to_client(metadata_cache, connection_manager, sub_pub_param, stop_sx).await?;

    // 2. wait PubRec ack
    let new_wait_ack_rx = wait_pub_rec(
        metadata_cache,
        connection_manager,
        sub_pub_param,
        stop_sx,
        wait_ack_rx,
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
        new_wait_ack_rx,
    )
    .await?;
    Ok(())
}

pub async fn send_message_to_client(
    resp: ResponsePackage,
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<MQTTCacheManager>,
) -> ResultMqttBrokerError {
    let start = now_millis();
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
                .write_websocket_frame(
                    resp.connection_id,
                    response,
                    Message::Binary(buff.to_vec().into()),
                )
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
                    &connection.tenant,
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
    mut wait_ack_rx: mpsc::Receiver<QosAckPackageData>,
) -> ResultMqttBrokerError {
    let mut stop_recv = stop_sx.subscribe();
    let mut resend_times = 0usize;
    loop {
        select! {
            val = stop_recv.recv() => {
                match val {
                    Ok(true) | Err(broadcast::error::RecvError::Closed) => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
            recv_res = timeout(Duration::from_secs(ACK_WAIT_TIMEOUT_SECS), wait_ack_rx.recv()) => {
                match recv_res {
                    Ok(Some(package)) => {
                        if package.ack_type == QosAckPackageType::PubAck
                            && package.pkid == sub_pub_param.p_kid
                        {
                            return Ok(());
                        }
                    }
                    Ok(None) => {
                        return Err(MqttBrokerError::CommonError(
                            "wait_pub_ack channel closed before receiving PubAck".to_string(),
                        ));
                    }
                    Err(_) => {
                        if resend_times >= QOS_ACK_RESEND_MAX_RETRIES {
                            return Err(MqttBrokerError::OperationTimeout(
                                ACK_WAIT_TIMEOUT_SECS,
                                "wait_pub_ack".to_string(),
                            ));
                        }
                        resend_times += 1;
                        let resend_param = build_dup_publish_param(sub_pub_param)?;
                        push_packet_to_client(
                            metadata_cache,
                            connection_manager,
                            &resend_param,
                            stop_sx
                        ).await?;
                    }
                }
            }
        }
    }
}

fn build_dup_publish_param(
    sub_pub_param: &SubPublishParam,
) -> Result<SubPublishParam, MqttBrokerError> {
    match &sub_pub_param.packet {
        MqttPacket::Publish(publish, properties) => {
            let mut resend_publish = publish.clone();
            resend_publish.dup = true;
            Ok(SubPublishParam {
                packet: MqttPacket::Publish(resend_publish, properties.clone()),
                create_time: sub_pub_param.create_time,
                client_id: sub_pub_param.client_id.clone(),
                p_kid: sub_pub_param.p_kid,
                qos: sub_pub_param.qos,
            })
        }
        _ => Err(MqttBrokerError::CommonError(
            "wait_pub_ack expects Publish packet for QoS1 resend".to_string(),
        )),
    }
}

pub async fn wait_pub_rec(
    metadata_cache: &Arc<MQTTCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    mut wait_ack_rx: mpsc::Receiver<QosAckPackageData>,
) -> Result<mpsc::Receiver<QosAckPackageData>, MqttBrokerError> {
    let mut stop_recv = stop_sx.subscribe();
    let mut resend_times = 0usize;
    loop {
        select! {
            val = stop_recv.recv() => {
                match val {
                    Ok(true) | Err(broadcast::error::RecvError::Closed) => {
                        break;
                    }
                    _ => {}
                }
            }
            recv_res = timeout(Duration::from_secs(ACK_WAIT_TIMEOUT_SECS), wait_ack_rx.recv()) => {
                match recv_res {
                    Ok(Some(package)) => {
                        if package.ack_type == QosAckPackageType::PubRec
                            && package.pkid == sub_pub_param.p_kid
                        {
                            break;
                        }
                    }
                    Ok(None) => {
                        return Err(MqttBrokerError::CommonError(
                            "wait_pub_rec channel closed before receiving PubRec".to_string(),
                        ));
                    }
                    Err(_) => {
                        if resend_times >= QOS_ACK_RESEND_MAX_RETRIES {
                            return Err(MqttBrokerError::OperationTimeout(
                                ACK_WAIT_TIMEOUT_SECS,
                                "wait_pub_rec".to_string(),
                            ));
                        }
                        resend_times += 1;
                        let resend_param = build_dup_publish_param(sub_pub_param)?;
                        push_packet_to_client(metadata_cache, connection_manager, &resend_param, stop_sx)
                            .await?;
                    }
                }
            }
        }
    }
    Ok(wait_ack_rx)
}

pub async fn wait_pub_comp(
    metadata_cache: &Arc<MQTTCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    stop_sx: &broadcast::Sender<bool>,
    mut wait_ack_rx: mpsc::Receiver<QosAckPackageData>,
) -> ResultMqttBrokerError {
    let mut stop_recv = stop_sx.subscribe();
    let mut resend_times = 0usize;
    loop {
        select! {
            val = stop_recv.recv() => {
                match val {
                    Ok(true) | Err(broadcast::error::RecvError::Closed) => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
            recv_res = timeout(Duration::from_secs(ACK_WAIT_TIMEOUT_SECS), wait_ack_rx.recv()) => {
                match recv_res {
                    Ok(Some(package)) => {
                        if package.ack_type == QosAckPackageType::PubComp
                            && package.pkid == sub_pub_param.p_kid
                        {
                            return Ok(());
                        }
                    }
                    Ok(None) => {
                        return Err(MqttBrokerError::CommonError(
                            "wait_pub_comp channel closed before receiving PubComp".to_string(),
                        ));
                    }
                    Err(_) => {
                        if resend_times >= QOS_ACK_RESEND_MAX_RETRIES {
                            return Err(MqttBrokerError::OperationTimeout(
                                ACK_WAIT_TIMEOUT_SECS,
                                "wait_pub_comp".to_string(),
                            ));
                        }
                        resend_times += 1;
                        qos2_send_pubrel(metadata_cache, sub_pub_param, connection_manager, stop_sx)
                            .await?;
                    }
                }
            }
        }
    }
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
                match val {
                    Ok(true) | Err(broadcast::error::RecvError::Closed) => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
            val = ac_fn() => {
                if let Err(e) = val {
                    if broker_not_available(&e.to_string()){
                        return Err(e);
                    }

                    if !client_unavailable_error(&e){
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
                match val {
                    Ok(true) | Err(broadcast::error::RecvError::Closed) => {
                        return Err(());
                    }
                    _ => {}
                }
            }
            _ = sleep(Duration::from_millis(RETRY_SLEEP_INTERVAL_MS)) => {}
        }
    }
    Ok(())
}
