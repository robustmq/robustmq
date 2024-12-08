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
use log::{error, info};
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::common::{MqttPacket, MqttProtocol, Publish, PublishProperties, QoS};
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::sleep;

use super::sub_common::{
    loop_commit_offset, min_qos, publish_message_qos0, publish_message_to_client,
    qos2_send_publish, qos2_send_pubrel, wait_packet_ack,
};
use super::subscribe_manager::{ShareLeaderSubscribeData, SubscribeManager};
use crate::handler::cache::{CacheManager, QosAckPackageData, QosAckPackageType, QosAckPacketInfo};
use crate::handler::error::MqttBrokerError;
use crate::handler::message::is_message_expire;
use crate::server::connection_manager::ConnectionManager;
use crate::server::packet::ResponsePackage;
use crate::storage::message::MessageStorage;
use crate::subscribe::subscriber::Subscriber;
use crate::subscribe::SubPublishParam;

#[derive(Clone)]
pub struct SubscribeShareLeader<S> {
    pub subscribe_manager: Arc<SubscribeManager>,
    message_storage: Arc<S>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
}

impl<S> SubscribeShareLeader<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        message_storage: Arc<S>,
        connection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        SubscribeShareLeader {
            subscribe_manager,
            message_storage,
            connection_manager,
            cache_manager,
        }
    }

    pub async fn start(&self) {
        loop {
            self.start_push_thread().await;
            self.try_thread_gc();
            sleep(Duration::from_secs(1)).await;
        }
    }

    pub fn try_thread_gc(&self) {
        // Periodically verify that a push task is running, but the subscribe task has stopped
        // If so, stop the process and clean up the data
        for (share_leader_key, sx) in self.subscribe_manager.share_leader_push_thread.clone() {
            if !self
                .subscribe_manager
                .share_leader_subscribe
                .contains_key(&share_leader_key)
            {
                match sx.send(true) {
                    Ok(_) => {
                        self.subscribe_manager
                            .share_leader_push_thread
                            .remove(&share_leader_key);
                    }
                    Err(err) => {
                        error!("stop sub share thread error, error message:{}", err);
                    }
                }
            }
        }
    }

    pub async fn start_push_thread(&self) {
        // Periodically verify if any push tasks are not started. If so, the thread is started
        for (share_leader_key, sub_data) in self.subscribe_manager.share_leader_subscribe.clone() {
            if sub_data.sub_list.is_empty() {
                if let Some(sx) = self
                    .subscribe_manager
                    .share_leader_push_thread
                    .get(&share_leader_key)
                {
                    if sx.send(true).is_ok() {
                        self.subscribe_manager
                            .share_leader_subscribe
                            .remove(&share_leader_key);
                    }
                }
            }

            // start push data thread
            if !self
                .subscribe_manager
                .share_leader_push_thread
                .contains_key(&share_leader_key)
            {
                self.push_by_round_robin(
                    share_leader_key,
                    sub_data,
                    self.subscribe_manager.clone(),
                )
                .await;
            }
        }
    }

    async fn push_by_round_robin(
        &self,
        share_leader_key: String,
        sub_data: ShareLeaderSubscribeData,
        subscribe_manager: Arc<SubscribeManager>,
    ) {
        let (sub_thread_stop_sx, mut sub_thread_stop_rx) = broadcast::channel(1);

        let group_id = format!(
            "system_sub_{}_{}_{}",
            sub_data.group_name, sub_data.sub_name, sub_data.topic_id
        );
        let cursor_point = 0;

        let message_storage = MessageStorage::new(self.message_storage.clone());

        // get current offset by group
        let mut offset = match message_storage.get_group_offset(&group_id).await {
            Ok(offset) => offset,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };

        // save push thread
        self.subscribe_manager
            .share_leader_push_thread
            .insert(share_leader_key.clone(), sub_thread_stop_sx.clone());

        let connection_manager = self.connection_manager.clone();
        let cache_manager = self.cache_manager.clone();

        tokio::spawn(async move {
            info!(
                "Share leader push data thread for GroupName {}/{},Topic [{}] was started successfully",
                sub_data.group_name, sub_data.sub_name, sub_data.topic_name
            );

            let mut sub_list: Vec<Subscriber> =
                build_share_leader_sub_list(&subscribe_manager, &share_leader_key);
            let mut pre_times = now_second();
            loop {
                select! {
                    val = sub_thread_stop_rx.recv() =>{
                        if let Ok(flag) = val {
                            if flag {
                                info!(
                                    "Share sub push data thread for GroupName {},Topic [{}] was stopped successfully",
                                    sub_data.group_name, sub_data.topic_name
                                );
                                break;
                            }
                        }
                    }
                    res = read_message_process(
                        &connection_manager,
                        &cache_manager,
                        &message_storage,
                        &sub_data,
                        &sub_list,
                        &group_id,
                        cursor_point,
                        offset,
                        &sub_thread_stop_sx
                    ) =>{
                        match res {
                            Ok(data) => {
                                if let Some(offset_cur) = data{
                                    offset = offset_cur + 1;
                                }
                            },

                            Err(e) => {
                                error!(
                                    "Failed to read message from storage, failure message: {},topic:{},group{}",
                                    e.to_string(),
                                    &sub_data.topic_id,
                                    group_id
                                );
                                sleep(Duration::from_millis(100)).await;
                            }
                        }

                        // Refresh the subscriber list of shared subscriptions every second to ensure that new subscribers can get messages in time.
                        if now_second() - pre_times >= 1{
                            sub_list = build_share_leader_sub_list(&subscribe_manager, &share_leader_key);
                            pre_times = now_second();
                        }
                    }
                }
            }

            subscribe_manager
                .share_leader_push_thread
                .remove(&share_leader_key);
        });
    }
}

#[allow(clippy::too_many_arguments)]
async fn read_message_process<S>(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<CacheManager>,
    message_storage: &MessageStorage<S>,
    sub_data: &ShareLeaderSubscribeData,
    sub_list: &[Subscriber],
    group_id: &str,
    mut cursor_point: usize,
    offset: u64,
    stop_sx: &Sender<bool>,
) -> Result<Option<u64>, MqttBrokerError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let record_num = calc_record_num(sub_list.len());

    let results = message_storage
        .read_topic_message(&sub_data.topic_id, offset, record_num as u64)
        .await?;

    if results.is_empty() {
        return Ok(None);
    }

    for record in results.iter() {
        let msg = MqttMessage::decode_record(record.clone())?;

        if is_message_expire(&msg) {
            continue;
        }

        let mut loop_times = 0;
        loop {
            if loop_times > try_loop_times(sub_list.len()) {
                error!("Share subscription push message fails, dropping the message, possibly because no subscriber is available");
                break;
            }

            cursor_point = choose_available_sub(cursor_point, sub_list);
            let subscribe = sub_list.get(cursor_point).unwrap();

            if let Some((mut publish, properties)) =
                build_publish(cache_manager, subscribe, &sub_data.topic_name, &msg)
            {
                let pkid = if publish.qos != QoS::AtMostOnce {
                    cache_manager.get_pkid(&subscribe.client_id).await
                } else {
                    0
                };

                publish.pkid = pkid;

                let sub_pub_param = SubPublishParam::new(
                    subscribe.clone(),
                    publish,
                    Some(properties),
                    record.timestamp,
                    group_id.to_owned(),
                    pkid,
                );

                if qos_publish(
                    connection_manager,
                    cache_manager,
                    message_storage,
                    sub_pub_param,
                    record.offset.unwrap(),
                    stop_sx,
                )
                .await
                {
                    break;
                }
            }
            loop_times += 1;
        }

        // commit offset
        loop_commit_offset(
            message_storage,
            &sub_data.topic_id,
            group_id,
            record.offset.unwrap(),
        )
        .await;
    }
    Ok(results.last().unwrap().offset)
}

fn try_loop_times(sub_len: usize) -> usize {
    sub_len * 2
}

fn choose_available_sub(cursor_point: usize, sub_list: &[Subscriber]) -> usize {
    let current_point = cursor_point + 1;
    if current_point < sub_list.len() {
        current_point
    } else {
        0
    }
}

async fn qos_publish<S>(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<CacheManager>,
    message_storage: &MessageStorage<S>,
    sub_pub_param: SubPublishParam,
    offset: u64,
    stop_sx: &Sender<bool>,
) -> bool
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    match sub_pub_param.publish.qos {
        QoS::AtMostOnce => {
            publish_message_qos0(cache_manager, connection_manager, &sub_pub_param, stop_sx).await;
            true
        }

        QoS::AtLeastOnce => {
            let (wait_puback_sx, _) = broadcast::channel(1);
            cache_manager.add_ack_packet(
                &sub_pub_param.subscribe.client_id,
                sub_pub_param.pkid,
                QosAckPacketInfo {
                    sx: wait_puback_sx.clone(),
                    create_time: now_second(),
                },
            );

            match share_leader_publish_message_qos1(
                cache_manager,
                connection_manager,
                &sub_pub_param,
                &wait_puback_sx,
            )
            .await
            {
                Ok(()) => {
                    // remove data
                    cache_manager
                        .remove_pkid_info(&sub_pub_param.subscribe.client_id, sub_pub_param.pkid);
                    cache_manager
                        .remove_ack_packet(&sub_pub_param.subscribe.client_id, sub_pub_param.pkid);
                    true
                }
                Err(e) => {
                    error!(
                        "SharSub Leader failed to send QOS1 message to {}, error message :{},
                     trying to deliver the message to another client.",
                        sub_pub_param.subscribe.client_id.clone(),
                        e.to_string()
                    );
                    false
                }
            }
        }

        QoS::ExactlyOnce => {
            let (wait_ack_sx, _) = broadcast::channel(1);
            cache_manager.add_ack_packet(
                &sub_pub_param.subscribe.client_id,
                sub_pub_param.pkid,
                QosAckPacketInfo {
                    sx: wait_ack_sx.clone(),
                    create_time: now_second(),
                },
            );

            match share_leader_publish_message_qos2(
                cache_manager,
                connection_manager,
                message_storage,
                &sub_pub_param,
                offset,
                stop_sx,
                &wait_ack_sx,
            )
            .await
            {
                Ok(()) => true,
                Err(e) => {
                    error!("{}", e);
                    false
                }
            }
        }
    }
}

fn build_publish(
    metadata_cache: &Arc<CacheManager>,
    subscribe: &Subscriber,
    topic_name: &str,
    msg: &MqttMessage,
) -> Option<(Publish, PublishProperties)> {
    let cluster_qos = metadata_cache.get_cluster_info().protocol.max_qos;
    let qos = min_qos(cluster_qos, subscribe.qos);

    let retain = if subscribe.preserve_retain {
        msg.retain
    } else {
        false
    };

    if subscribe.nolocal && (subscribe.client_id == msg.client_id) {
        return None;
    }

    let publish = Publish {
        dup: false,
        qos,
        pkid: 0,
        retain,
        topic: Bytes::from(topic_name.to_owned()),
        payload: msg.payload.clone(),
    };

    let mut sub_ids = Vec::new();
    if let Some(id) = subscribe.subscription_identifier {
        sub_ids.push(id);
    }

    let properties = PublishProperties {
        payload_format_indicator: msg.format_indicator,
        message_expiry_interval: Some(msg.expiry_interval as u32),
        topic_alias: None,
        response_topic: msg.response_topic.clone(),
        correlation_data: msg.correlation_data.clone(),
        user_properties: msg.user_properties.clone(),
        subscription_identifiers: sub_ids,
        content_type: msg.content_type.clone(),
    };
    Some((publish, properties))
}

// To avoid messages that are not successfully pushed to the client. When the client Session expires,
// the push thread will exit automatically and will not attempt to push again.
async fn share_leader_publish_message_qos1(
    metadata_cache: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    sub_pub_param: &SubPublishParam,
    wait_puback_sx: &broadcast::Sender<QosAckPackageData>,
) -> Result<(), MqttBrokerError> {
    let connect_id =
        if let Some(id) = metadata_cache.get_connect_id(&sub_pub_param.subscribe.client_id) {
            id
        } else {
            return Err(MqttBrokerError::CommonError(format!(
                "Client [{}] failed to get connect id, no connection available.",
                sub_pub_param.subscribe.client_id
            )));
        };

    if let Some(conn) = metadata_cache.get_connection(connect_id) {
        if sub_pub_param.publish.payload.len() > (conn.max_packet_size as usize) {
            return Err(MqttBrokerError::PacketLengthError(
                sub_pub_param.publish.payload.len(),
            ));
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

    match publish_message_to_client(resp.clone(), sub_pub_param, connection_manager).await {
        Ok(_) => {
            if let Some(data) = wait_packet_ack(wait_puback_sx).await {
                if data.ack_type == QosAckPackageType::PubAck && data.pkid == sub_pub_param.pkid {
                    return Ok(());
                }
            }
            Err(MqttBrokerError::CommonError(
                "QOS1 publishes a message and waits for the PubAck packet to fail to be received"
                    .to_string(),
            ))
        }
        Err(e) => Err(MqttBrokerError::CommonError(format!(
            "Failed to write QOS1 Publish message to response queue, failure message: {}",
            e
        ))),
    }
}

// send publish message
// wait pubrec message
// send pubrel message
// wait pubcomp message

async fn share_leader_publish_message_qos2<S>(
    cache_manager: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    message_storage: &MessageStorage<S>,
    sub_pub_param: &SubPublishParam,
    offset: u64,
    stop_sx: &broadcast::Sender<bool>,
    wait_ack_sx: &broadcast::Sender<QosAckPackageData>,
) -> Result<(), MqttBrokerError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    // 1. send Publish to Client
    qos2_send_publish(connection_manager, cache_manager, sub_pub_param, stop_sx).await?;

    // 2. wait pub rec
    loop {
        if let Ok(flag) = stop_sx.subscribe().try_recv() {
            if flag {
                return Ok(());
            }
        }
        if let Some(data) = wait_packet_ack(wait_ack_sx).await {
            if data.ack_type == QosAckPackageType::PubRec && data.pkid == sub_pub_param.pkid {
                // When sending a QOS2 message, as long as the pubrec is received, the offset can be submitted,
                // the pubrel is sent asynchronously, and the pubcomp is waited for. Push the next message at the same time.
                loop_commit_offset(
                    message_storage,
                    &sub_pub_param.subscribe.topic_id,
                    &sub_pub_param.group_id,
                    offset,
                )
                .await;
                break;
            }
        } else {
            return Err(MqttBrokerError::SubPublishWaitPubRecTimeout(
                sub_pub_param.subscribe.client_id.to_owned(),
            ));
        }
    }

    // async wait
    // 3. send pub rel
    qos2_send_pubrel(cache_manager, sub_pub_param, connection_manager, stop_sx).await;

    // 4. wait pub comp
    loop {
        if let Ok(flag) = stop_sx.subscribe().try_recv() {
            if flag {
                break;
            }
        }
        if let Some(data) = wait_packet_ack(wait_ack_sx).await {
            if data.ack_type == QosAckPackageType::PubComp && data.pkid == sub_pub_param.pkid {
                cache_manager
                    .remove_pkid_info(&sub_pub_param.subscribe.client_id, sub_pub_param.pkid);
                cache_manager
                    .remove_ack_packet(&sub_pub_param.subscribe.client_id, sub_pub_param.pkid);
                break;
            }
        } else {
            qos2_send_pubrel(cache_manager, sub_pub_param, connection_manager, stop_sx).await;
        }
    }

    Ok(())
}

fn build_share_leader_sub_list(
    subscribe_manager: &Arc<SubscribeManager>,
    key: &str,
) -> Vec<Subscriber> {
    let sub_list = if let Some(sub) = subscribe_manager.share_leader_subscribe.get(key) {
        sub.sub_list.clone()
    } else {
        return Vec::new();
    };

    let mut result = Vec::new();
    for (_, sub) in sub_list {
        result.push(sub);
    }
    result
}

fn calc_record_num(sub_len: usize) -> usize {
    if sub_len == 0 {
        return 100;
    }

    let num = sub_len * 5;
    if num > 1000 {
        return 1000;
    }
    num
}

#[cfg(test)]
mod tests {}
