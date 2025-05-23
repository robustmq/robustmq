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

use crate::handler::cache::{CacheManager, QosAckPacketInfo};
use crate::handler::error::MqttBrokerError;
use crate::handler::message::is_message_expire;
use crate::server::connection_manager::ConnectionManager;
use crate::storage::message::MessageStorage;
use crate::subscribe::common::{
    exclusive_publish_message_qos1, exclusive_publish_message_qos2, get_pkid, loop_commit_offset,
    min_qos, publish_message_qos,
};
use crate::subscribe::manager::{ShareLeaderSubscribeData, SubscribeManager};
use crate::subscribe::meta::SubPublishParam;
use crate::subscribe::meta::Subscriber;
use bytes::Bytes;
use common_base::tools::now_second;
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::common::{Publish, PublishProperties, QoS};
use storage_adapter::storage::StorageAdapter;
use tokio::select;
use tokio::sync::broadcast::{self, Sender};
use tokio::time::sleep;
use tracing::{error, info};
#[derive(Clone)]
pub struct ShareLeaderPush<S> {
    pub subscribe_manager: Arc<SubscribeManager>,
    message_storage: Arc<S>,
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<CacheManager>,
}

impl<S> ShareLeaderPush<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        subscribe_manager: Arc<SubscribeManager>,
        message_storage: Arc<S>,
        connection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        ShareLeaderPush {
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
                .share_leader_push
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
        for (share_leader_key, sub_data) in self.subscribe_manager.share_leader_push.clone() {
            if sub_data.sub_list.is_empty() {
                if let Some(sx) = self
                    .subscribe_manager
                    .share_leader_push_thread
                    .get(&share_leader_key)
                {
                    if sx.send(true).is_ok() {
                        self.subscribe_manager
                            .share_leader_push
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
                if let Err(e) = self.push_by_round_robin(share_leader_key, sub_data).await {
                    error!("{:?}", e);
                }
            }
        }
    }

    async fn push_by_round_robin(
        &self,
        share_leader_key: String,
        sub_data: ShareLeaderSubscribeData,
    ) -> Result<(), MqttBrokerError> {
        let (sub_thread_stop_sx, mut sub_thread_stop_rx) = broadcast::channel(1);
        let group_id = format!(
            "system_sub_{}_{}_{}",
            sub_data.group_name, sub_data.sub_name, sub_data.topic_id
        );

        // get current offset by group
        let message_storage = MessageStorage::new(self.message_storage.clone());
        let mut offset = message_storage.get_group_offset(&group_id).await?;

        // save push thread
        self.subscribe_manager
            .share_leader_push_thread
            .insert(share_leader_key.clone(), sub_thread_stop_sx.clone());

        let connection_manager = self.connection_manager.clone();
        let cache_manager = self.cache_manager.clone();
        let subscribe_manager = self.subscribe_manager.clone();

        tokio::spawn(async move {
            info!(
                "Share leader push data thread for GroupName {}/{},Topic [{}] was started successfully",
                sub_data.group_name, sub_data.sub_name, sub_data.topic_name
            );

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
                        &subscribe_manager,
                        &share_leader_key,
                        &sub_data,
                        &group_id,
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
                    }
                }
            }

            subscribe_manager
                .share_leader_push_thread
                .remove(&share_leader_key);
        });
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
async fn read_message_process<S>(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<CacheManager>,
    message_storage: &MessageStorage<S>,
    subscribe_manager: &Arc<SubscribeManager>,
    share_leader_key: &str,
    sub_data: &ShareLeaderSubscribeData,
    group_id: &str,
    offset: u64,
    stop_sx: &Sender<bool>,
) -> Result<Option<u64>, MqttBrokerError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let results = message_storage
        .read_topic_message(&sub_data.topic_id, offset, 100)
        .await?;

    if results.is_empty() {
        return Ok(None);
    }

    for record in results.iter() {
        let msg = MqttMessage::decode_record(record.clone())?;

        if is_message_expire(&msg) {
            continue;
        }

        loop {
            let subscribe = if let Some(subscrbie) =
                get_subscribe_by_random(subscribe_manager, share_leader_key, record.offset.unwrap())
            {
                subscrbie
            } else {
                info!(
                    "Not subscribed to the topic {}, and the message will not be sent.",
                    &sub_data.topic_name
                );
                continue;
            };

            if let Some((mut publish, properties)) =
                build_publish(cache_manager, &subscribe, &sub_data.topic_name, &msg)
            {
                let pkid = get_pkid();
                publish.pkid = pkid;

                let sub_pub_param = SubPublishParam::new(
                    subscribe.clone(),
                    publish,
                    Some(properties),
                    record.timestamp as u128,
                    group_id.to_owned(),
                    pkid,
                );

                publish_data(connection_manager, cache_manager, sub_pub_param, stop_sx).await?;
                break;
            }
        }

        // commit offset
        loop_commit_offset(
            message_storage,
            &sub_data.topic_id,
            group_id,
            record.offset.unwrap(),
        )
        .await?;
    }
    Ok(results.last().unwrap().offset)
}

async fn publish_data(
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
            let pkid = sub_pub_param.pkid;
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

fn get_subscribe_by_random(
    subscribe_manager: &Arc<SubscribeManager>,
    share_leader_key: &str,
    offset: u64,
) -> Option<Subscriber> {
    if let Some(sub_list) = subscribe_manager.share_leader_push.get(share_leader_key) {
        let index = offset % (sub_list.sub_list.len() as u64);
        if let Some(subscribe) = sub_list.sub_list.get(index as usize) {
            return Some(subscribe.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {}
