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

use super::cache::MQTTCacheManager;
use super::constant::{
    MAX_RETAIN_MESSAGE_SEND_CONCURRENCY, SUB_RETAIN_MESSAGE_PUSH_FLAG,
    SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE,
};
use super::message::build_message_expire;
use crate::core::error::MqttBrokerError;
use crate::core::sub_option::{
    get_retain_flag_by_retain_as_published, is_send_msg_by_bo_local,
    is_send_retain_msg_by_retain_handling,
};
use crate::core::subscribe::is_new_sub;
use crate::core::tool::ResultMqttBrokerError;
use crate::storage::topic::TopicStorage;
use crate::subscribe::common::SubPublishParam;
use crate::subscribe::common::{client_unavailable_error, get_sub_topic_name_list};
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::push::send_publish_packet_to_client;
use bytes::Bytes;
use common_base::tools::now_second;
use common_metrics::mqtt::packets::{record_retain_recv_metrics, record_retain_sent_metrics};
use common_metrics::mqtt::statistics::{record_mqtt_retained_dec, record_mqtt_retained_inc};
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::message::MqttMessage;
use network_server::common::connection_manager::ConnectionManager;
use protocol::mqtt::common::{
    MqttPacket, Publish, PublishProperties, QoS, Subscribe, SubscribeProperties,
};
use std::sync::Arc;
use tokio::select;
use tokio::sync::{broadcast, mpsc, Semaphore};
use tracing::{debug, error, warn};

#[derive(Clone)]
struct SendRetainData {
    pub topic_name: String,
    pub client_id: String,
    pub no_local: bool,
    pub preserve_retain: bool,
    pub qos: QoS,
    pub sub_ids: Vec<usize>,
}

pub struct RetainMessageManager {
    cache_manager: Arc<MQTTCacheManager>,
    client_pool: Arc<ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    topic_retain_data: DashMap<String, Option<(MqttMessage, u64)>>,
    last_update_time: DashMap<String, u64>,
    send_retain_message_queue: mpsc::Sender<SendRetainData>,
}

impl RetainMessageManager {
    pub fn new(
        cache_manager: Arc<MQTTCacheManager>,
        client_pool: Arc<ClientPool>,
        connection_manager: Arc<ConnectionManager>,
        stop_sx: broadcast::Sender<bool>,
    ) -> Arc<RetainMessageManager> {
        let (send_retain_message_queue, rx) = mpsc::channel::<SendRetainData>(3000);
        let manager = Arc::new(RetainMessageManager {
            cache_manager,
            client_pool,
            connection_manager,
            topic_retain_data: DashMap::new(),
            last_update_time: DashMap::new(),
            send_retain_message_queue,
        });
        start_send_retain_thread(manager.clone(), rx, stop_sx);
        manager
    }

    pub async fn save_retain_message(
        &self,
        topic_name: &str,
        client_id: &str,
        publish: &Publish,
        publish_properties: &Option<PublishProperties>,
    ) -> ResultMqttBrokerError {
        if !publish.retain {
            return Ok(());
        }

        let topic_storage = TopicStorage::new(self.client_pool.clone());
        let had_retain = self.contain_retain(topic_name).await?;

        if had_retain && publish.payload.is_empty() {
            topic_storage.delete_retain_message(topic_name).await?;
            record_mqtt_retained_dec();
            self.topic_retain_data.insert(topic_name.to_string(), None);
            self.last_update_time
                .insert(topic_name.to_string(), now_second());
        }

        if !publish.payload.is_empty() {
            record_retain_recv_metrics(publish.qos);
            if !had_retain {
                record_mqtt_retained_inc();
            }
            let message_expire =
                build_message_expire(&self.cache_manager, publish_properties).await;
            let retain_message =
                MqttMessage::build_message(client_id, publish, publish_properties, message_expire);
            topic_storage
                .set_retain_message(topic_name, &retain_message, message_expire)
                .await?;

            self.topic_retain_data.insert(
                topic_name.to_string(),
                Some((retain_message, message_expire)),
            );
            self.last_update_time
                .insert(topic_name.to_string(), now_second());
        }

        Ok(())
    }

    pub async fn try_send_retain_message(
        &self,
        client_id: &str,
        subscribe: &Subscribe,
        subscribe_properties: &Option<SubscribeProperties>,
        cache_manager: &Arc<MQTTCacheManager>,
        subscribe_manager: &Arc<SubscribeManager>,
    ) -> Result<(), MqttBrokerError> {
        let sub_ids = subscribe_properties
            .as_ref()
            .and_then(|props| props.subscription_identifier)
            .map(|id| vec![id])
            .unwrap_or_default();

        let is_new_subs = is_new_sub(client_id, subscribe, subscribe_manager);
        for filter in subscribe.filters.iter() {
            if !is_send_retain_msg_by_retain_handling(
                &filter.path,
                &filter.retain_handling,
                &is_new_subs,
            ) {
                debug!("retain messages: Determine whether to send retained messages based on the retain handling strategy. Client ID: {}", client_id);
                continue;
            }

            let topic_name_list = get_sub_topic_name_list(cache_manager, &filter.path).await;
            if topic_name_list.is_empty() {
                continue;
            }

            for topic_name in topic_name_list {
                if !self.contain_retain(&topic_name).await? {
                    continue;
                }

                let data = SendRetainData {
                    client_id: client_id.to_string(),
                    no_local: filter.no_local,
                    preserve_retain: filter.preserve_retain,
                    qos: filter.qos,
                    sub_ids: sub_ids.clone(),
                    topic_name,
                };
                if let Err(e) = self.send_retain_message_queue.send(data).await {
                    return Err(MqttBrokerError::CommonError(e.to_string()));
                }
            }
        }
        Ok(())
    }

    async fn contain_retain(&self, topic: &str) -> Result<bool, MqttBrokerError> {
        let current_time = now_second();

        let last_update_time = if let Some(update_time) = self.last_update_time.get(topic) {
            *update_time
        } else {
            self.load_retain_from_storage(topic).await?;
            current_time
        };

        if current_time - last_update_time >= 5 {
            self.load_retain_from_storage(topic).await?;
        }

        let contain = if let Some(data) = self.topic_retain_data.get(topic) {
            if let Some((_, expire_at)) = data.as_ref() {
                if current_time >= *expire_at {
                    drop(data);
                    self.clear_local_retain_cache(topic, current_time);
                    false
                } else {
                    true
                }
            } else {
                false
            }
        } else {
            false
        };
        Ok(contain)
    }

    async fn load_retain_from_storage(&self, topic: &str) -> Result<(), MqttBrokerError> {
        let topic_storage = TopicStorage::new(self.client_pool.clone());
        let (message, message_at) = topic_storage.get_retain_message(topic).await?;

        let current_time = now_second();
        self.last_update_time
            .insert(topic.to_string(), current_time);

        let retain_data = match (message, message_at) {
            (Some(msg), Some(msg_at)) => Some((msg, msg_at)),
            _ => None,
        };
        self.topic_retain_data
            .insert(topic.to_string(), retain_data);

        Ok(())
    }

    fn clear_local_retain_cache(&self, topic: &str, current_time: u64) {
        self.topic_retain_data.insert(topic.to_string(), None);
        self.last_update_time
            .insert(topic.to_string(), current_time);
    }

    async fn send_retain_message(
        &self,
        data: &SendRetainData,
        stop_sx: &broadcast::Sender<bool>,
    ) -> ResultMqttBrokerError {
        let mut is_expired = false;
        let msg_data = {
            let cache_entry = self.topic_retain_data.get(&data.topic_name);
            if let Some(entry) = cache_entry {
                if let Some((message, message_at)) = entry.as_ref() {
                    if now_second() >= *message_at {
                        is_expired = true;
                        None
                    } else {
                        Some((
                            message.client_id.clone(),
                            message.retain,
                            message.payload.clone(),
                            message.format_indicator,
                            message.expiry_interval,
                            message.response_topic.clone(),
                            message.correlation_data.clone(),
                            message.user_properties.clone(),
                            message.content_type.clone(),
                        ))
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        if is_expired {
            self.clear_local_retain_cache(&data.topic_name, now_second());
            return Ok(());
        }

        let Some((
            msg_client_id,
            retain_flag,
            payload,
            format_indicator,
            expiry_interval,
            response_topic,
            correlation_data,
            user_properties_opt,
            content_type,
        )) = msg_data
        else {
            return Ok(());
        };

        if !is_send_msg_by_bo_local(data.no_local, &data.client_id, &msg_client_id) {
            return Ok(());
        }

        let retain = get_retain_flag_by_retain_as_published(data.preserve_retain, retain_flag);

        let mut user_properties = user_properties_opt.unwrap_or_default();
        user_properties.push((
            SUB_RETAIN_MESSAGE_PUSH_FLAG.to_string(),
            SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE.to_string(),
        ));

        let properties = PublishProperties {
            payload_format_indicator: format_indicator,
            message_expiry_interval: Some(expiry_interval as u32),
            topic_alias: None,
            response_topic,
            correlation_data,
            user_properties,
            subscription_identifiers: data.sub_ids.clone(),
            content_type,
        };

        let p_kid = self
            .cache_manager
            .pkid_data
            .generate_publish_to_client_pkid(&data.client_id, &data.qos)
            .await;

        let publish = Publish {
            dup: false,
            qos: data.qos,
            p_kid,
            retain,
            topic: Bytes::copy_from_slice(data.topic_name.as_bytes()),
            payload,
        };

        let packet = MqttPacket::Publish(publish, Some(properties));

        let sub_pub_param = SubPublishParam {
            packet,
            create_time: now_second(),
            client_id: data.client_id.clone(),
            p_kid,
            qos: data.qos,
        };

        send_publish_packet_to_client(
            &self.connection_manager,
            &self.cache_manager,
            &sub_pub_param,
            stop_sx,
        )
        .await?;
        debug!(
            "retain message sent successfully: client_id: {}, topic_id: {}",
            data.client_id, data.topic_name
        );

        record_retain_sent_metrics(data.qos);

        Ok(())
    }
}

fn start_send_retain_thread(
    retain_message_manager: Arc<RetainMessageManager>,
    mut retain_message_rx: mpsc::Receiver<SendRetainData>,
    stop_sx: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let mut stop_recv = stop_sx.subscribe();

        let semaphore = Arc::new(Semaphore::new(MAX_RETAIN_MESSAGE_SEND_CONCURRENCY));

        loop {
            select! {
                val = stop_recv.recv() =>{
                    match val {
                        Ok(flag) if flag => {
                            break;
                        }
                        Err(_) => {
                            break;
                        }
                        _ => {}
                    }
                }
                res = retain_message_rx.recv()  => {
                    if let Some(data) = res{
                        let raw_retain_message_manager = retain_message_manager.clone();
                        let raw_stop_sx = stop_sx.clone();
                        let semaphore_clone = semaphore.clone();
                        let permit = match semaphore_clone.acquire_owned().await {
                            Ok(permit) => permit,
                            Err(e) => {
                                error!(
                                    "Failed to acquire semaphore permit for sending retain message, \
                                    semaphore may be closed. client_id={}, topic={}, qos={:?}, error={}",
                                    data.client_id, data.topic_name, data.qos, e
                                );
                                continue;
                            }
                        };
                        tokio::spawn(async move{
                            let _permit = permit;
                            debug!(
                                "Processing retain message: client_id={}, topic={}, qos={:?}",
                                data.client_id, data.topic_name, data.qos
                            );

                            if let Err(e) = raw_retain_message_manager.send_retain_message(&data, &raw_stop_sx).await{
                                if !client_unavailable_error(&e) {
                                    warn!(
                                        "Sending retain message failed: client_id={}, topic={}, qos={:?}, error={}",
                                        data.client_id, data.topic_name, data.qos, e
                                    );
                                }
                            }
                        });
                    }

                }
            }
        }
    });
}
