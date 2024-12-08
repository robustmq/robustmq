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

use bytes::Bytes;
use common_base::tools::now_second;
use log::{debug, error, info};
use metadata_struct::adapter::record::Record;
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::common::{MqttPacket, MqttProtocol, Publish, PublishProperties, QoS};
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast::{self};
use tokio::time::sleep;

use super::sub_common::{
    loop_commit_offset, min_qos, publish_message_qos0, publish_message_to_client,
    qos2_send_publish, qos2_send_pubrel, wait_packet_ack,
};
use super::subscribe_manager::SubscribeManager;
use super::subscriber::Subscriber;
use crate::handler::cache::{CacheManager, QosAckPackageData, QosAckPackageType, QosAckPacketInfo};
use crate::handler::error::MqttBrokerError;
use crate::handler::message::is_message_expire;
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::ResponsePackage;
use crate::storage::message::MessageStorage;
use crate::subscribe::SubPublishParam;

pub struct SubscribeExclusive<S> {
    cache_manager: Arc<CacheManager>,
    subscribe_manager: Arc<SubscribeManager>,
    connection_manager: Arc<ConnectionManager>,
    message_storage: Arc<S>,
}

impl<S> SubscribeExclusive<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        message_storage: Arc<S>,
        cache_manager: Arc<CacheManager>,
        subscribe_manager: Arc<SubscribeManager>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        SubscribeExclusive {
            message_storage,
            cache_manager,
            subscribe_manager,
            connection_manager,
        }
    }

    pub async fn start(&self) {
        loop {
            self.start_push_thread().await;
            self.try_thread_gc().await;
            sleep(Duration::from_secs(1)).await;
        }
    }

    async fn try_thread_gc(&self) {
        // Periodically verify that a push task is running, but the subscribe task has stopped
        // If so, stop the process and clean up the data
        for (exclusive_key, sx) in self.subscribe_manager.exclusive_push_thread.clone() {
            if !self
                .subscribe_manager
                .exclusive_subscribe
                .contains_key(&exclusive_key)
                && sx.send(true).is_ok()
            {
                self.subscribe_manager
                    .exclusive_push_thread
                    .remove(&exclusive_key);
            }
        }
    }

    // Handles exclusive subscription push tasks
    // Exclusively subscribed messages are pushed directly to the consuming client
    async fn start_push_thread(&self) {
        for (exclusive_key, subscriber) in self.subscribe_manager.exclusive_subscribe.clone() {
            if self
                .subscribe_manager
                .exclusive_push_thread
                .contains_key(&exclusive_key)
            {
                continue;
            }

            let (sub_thread_stop_sx, mut sub_thread_stop_rx) = broadcast::channel(1);

            let message_storage = MessageStorage::new(self.message_storage.clone());
            let cache_manager = self.cache_manager.clone();
            let connection_manager = self.connection_manager.clone();
            let subscribe_manager = self.subscribe_manager.clone();

            // Subscribe to the data push thread
            self.subscribe_manager
                .exclusive_push_thread
                .insert(exclusive_key.clone(), sub_thread_stop_sx.clone());

            tokio::spawn(async move {
                info!("Exclusive push thread for client_id [{}], sub_path: [{}], topic_id [{}] was started successfully",
                        subscriber.client_id, subscriber.sub_path, subscriber.topic_id);

                let group_id = build_group_name(&subscriber);
                let qos = build_pub_qos(&cache_manager, &subscriber);
                let sub_ids = build_sub_ids(&subscriber);

                let mut offset = match message_storage.get_group_offset(&group_id).await {
                    Ok(offset) => offset,
                    Err(e) => {
                        error!("{}", e);
                        subscribe_manager
                            .exclusive_push_thread
                            .remove(&exclusive_key);
                        return;
                    }
                };

                loop {
                    select! {
                        val = sub_thread_stop_rx.recv() =>{
                            if let Ok(flag) = val {
                                if flag {
                                    info!(
                                        "Exclusive Push thread for client_id [{}], sub_path: [{}], topic_id [{}] was stopped successfully",
                                        subscriber.client_id,
                                        subscriber.sub_path,
                                        subscriber.topic_id
                                    );

                                    subscribe_manager.exclusive_push_thread.remove(&exclusive_key);
                                    break;
                                }
                            }
                        },
                        val = pub_message(
                                &connection_manager,
                                &message_storage,
                                &cache_manager,
                                &subscriber,
                                &group_id,
                                &qos,
                                &sub_ids,
                                offset,
                                &sub_thread_stop_sx
                            ) => {
                                match val{
                                    Ok(offset_op) => {
                                        if let Some(off) = offset_op{
                                            offset = off + 1;
                                        }else{
                                            sleep(Duration::from_millis(100)).await;
                                        }
                                    }
                                    Err(e) => {
                                        error!(
                                            "Push message to client failed, failure message: {},topic:{},group{}",
                                            e.to_string(),
                                            subscriber.topic_id.clone(),
                                            group_id.clone()
                                        );
                                        sleep(Duration::from_millis(100)).await;
                                    }
                                }
                            }
                    }
                }
            });
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn pub_message<S>(
    connection_manager: &Arc<ConnectionManager>,
    message_storage: &MessageStorage<S>,
    cache_manager: &Arc<CacheManager>,
    subscriber: &Subscriber,
    group_id: &str,
    qos: &QoS,
    sub_ids: &[usize],
    offset: u64,
    sub_thread_stop_sx: &broadcast::Sender<bool>,
) -> Result<Option<u64>, MqttBrokerError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let record_num = 5;
    let client_id = subscriber.client_id.clone();

    let results = message_storage
        .read_topic_message(&subscriber.topic_id, offset, record_num)
        .await?;

    if results.is_empty() {
        return Ok(None);
    }

    for record in results.iter() {
        let record_offset = record.offset.unwrap();

        // build publish params
        let sub_pub_param = if let Some(params) = buile_pub_message(
            record.to_owned(),
            group_id,
            qos,
            subscriber,
            cache_manager,
            sub_ids,
        )
        .await?
        {
            params
        } else {
            continue;
        };

        let pkid = sub_pub_param.pkid;
        match qos {
            QoS::AtMostOnce => {
                publish_message_qos0(
                    cache_manager,
                    connection_manager,
                    &sub_pub_param,
                    sub_thread_stop_sx,
                )
                .await;
            }

            QoS::AtLeastOnce => {
                let (wait_puback_sx, _) = broadcast::channel(1);
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
                    sub_thread_stop_sx,
                    &wait_puback_sx,
                )
                .await?;

                cache_manager.remove_pkid_info(&client_id, pkid);
                cache_manager.remove_ack_packet(&client_id, pkid);
            }

            QoS::ExactlyOnce => {
                let (wait_ack_sx, _) = broadcast::channel(1);
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
                    sub_thread_stop_sx,
                    &wait_ack_sx,
                )
                .await?;

                cache_manager.remove_pkid_info(&client_id, pkid);
                cache_manager.remove_ack_packet(&client_id, pkid);
            }
        }

        // commit offset
        loop_commit_offset(
            message_storage,
            &subscriber.topic_id,
            group_id,
            record_offset,
        )
        .await;
    }

    Ok(Some(results.last().unwrap().offset.unwrap()))
}

async fn buile_pub_message(
    record: Record,
    group_id: &str,
    qos: &QoS,
    subscriber: &Subscriber,
    cache_manager: &Arc<CacheManager>,
    sub_ids: &[usize],
) -> Result<Option<SubPublishParam>, MqttBrokerError> {
    let msg = MqttMessage::decode_record(record.clone())?;

    if is_message_expire(&msg) {
        debug!("message expires, is not pushed to the client, and is discarded");
        return Ok(None);
    }

    if subscriber.nolocal && (subscriber.client_id == msg.client_id) {
        return Ok(None);
    }

    let retain = if subscriber.preserve_retain {
        msg.retain
    } else {
        false
    };

    let mut publish = Publish {
        dup: false,
        qos: qos.to_owned(),
        pkid: 0,
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

    let pkid = if *qos != QoS::AtMostOnce {
        cache_manager.get_pkid(&subscriber.client_id).await
    } else {
        0
    };
    publish.pkid = pkid;

    let sub_pub_param = SubPublishParam::new(
        subscriber.clone(),
        publish,
        Some(properties),
        record.timestamp,
        group_id.to_string(),
        pkid,
    );
    Ok(Some(sub_pub_param))
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
    let mut retry_times = 0;
    let mut publish = sub_pub_param.publish.clone();
    loop {
        if let Ok(flag) = stop_sx.subscribe().try_recv() {
            if flag {
                return Ok(());
            }
        }

        let connect_id =
            if let Some(id) = metadata_cache.get_connect_id(&sub_pub_param.subscribe.client_id) {
                id
            } else {
                sleep(Duration::from_secs(1)).await;
                continue;
            };

        if let Some(conn) = metadata_cache.get_connection(connect_id) {
            if publish.payload.len() > (conn.max_packet_size as usize) {
                return Ok(());
            }
        }

        retry_times += 1;
        publish.dup = retry_times >= 2;

        let mut contain_properties = false;
        if let Some(protocol) = connection_manager.get_connect_protocol(connect_id) {
            if MqttProtocol::is_mqtt5(&protocol) {
                contain_properties = true;
            }
        }

        let resp = if contain_properties {
            ResponsePackage {
                connection_id: connect_id,
                packet: MqttPacket::Publish(publish.clone(), sub_pub_param.properties.clone()),
            }
        } else {
            ResponsePackage {
                connection_id: connect_id,
                packet: MqttPacket::Publish(publish.clone(), None),
            }
        };

        match publish_message_to_client(resp, sub_pub_param, connection_manager).await {
            Ok(_) => {
                if let Some(data) = wait_packet_ack(wait_puback_sx).await {
                    if data.ack_type == QosAckPackageType::PubAck && data.pkid == sub_pub_param.pkid
                    {
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                error!(
                    "Failed to write QOS1 Publish message to response queue, failure message: {}",
                    e
                );
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
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
    qos2_send_publish(connection_manager, metadata_cache, sub_pub_param, stop_sx).await?;

    // 2. wait PubRec ack
    loop {
        if let Ok(flag) = stop_sx.subscribe().try_recv() {
            if flag {
                return Ok(());
            }
        }
        if let Some(data) = wait_packet_ack(wait_ack_sx).await {
            if data.ack_type == QosAckPackageType::PubRec && data.pkid == sub_pub_param.pkid {
                break;
            }
        } else {
            qos2_send_publish(connection_manager, metadata_cache, sub_pub_param, stop_sx).await?;
        }
        sleep(Duration::from_millis(1)).await;
    }

    // 3. send PubRel to Client
    qos2_send_pubrel(metadata_cache, sub_pub_param, connection_manager, stop_sx).await;

    // 4. wait pub comp
    loop {
        if let Ok(flag) = stop_sx.subscribe().try_recv() {
            if flag {
                break;
            }
        }
        if let Some(data) = wait_packet_ack(wait_ack_sx).await {
            if data.ack_type == QosAckPackageType::PubComp && data.pkid == sub_pub_param.pkid {
                break;
            }
        } else {
            qos2_send_pubrel(metadata_cache, sub_pub_param, connection_manager, stop_sx).await;
        }
        sleep(Duration::from_millis(1)).await;
    }
    Ok(())
}

fn build_group_name(subscriber: &Subscriber) -> String {
    format!(
        "system_sub_{}_{}_{}",
        subscriber.client_id, subscriber.sub_path, subscriber.topic_id
    )
}

fn build_pub_qos(cache_manager: &Arc<CacheManager>, subscriber: &Subscriber) -> QoS {
    let cluster_qos = cache_manager.get_cluster_info().protocol.max_qos;
    min_qos(cluster_qos, subscriber.qos)
}

fn build_sub_ids(subscriber: &Subscriber) -> Vec<usize> {
    let mut sub_ids = Vec::new();
    if let Some(id) = subscriber.subscription_identifier {
        sub_ids.push(id);
    }
    sub_ids
}

#[cfg(test)]
mod test {}
