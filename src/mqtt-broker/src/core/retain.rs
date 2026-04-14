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
use crate::core::sub_option::is_send_retain_msg_by_retain_handling;
use crate::core::subscribe::is_new_sub;
use crate::core::tool::ResultMqttBrokerError;
use crate::storage::retain::RetainStorage;
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
use metadata_struct::mqtt::retain_message::MQTTRetainMessage;
use network_server::common::connection_manager::ConnectionManager;
use protocol::mqtt::common::{MqttPacket, Publish, PublishProperties, QoS, Subscribe};
use std::sync::Arc;
use tokio::select;
use tokio::sync::{broadcast, mpsc, Semaphore};
use tracing::{debug, error, warn};

#[derive(Clone)]
struct SendRetainData {
    pub tenant: String,
    pub topic_name: String,
    pub client_id: String,
}

pub struct RetainMessageManager {
    cache_manager: Arc<MQTTCacheManager>,
    client_pool: Arc<ClientPool>,
    connection_manager: Arc<ConnectionManager>,
    topic_retain_data: DashMap<String, DashMap<String, MQTTRetainMessage>>,
    last_update_time: DashMap<String, DashMap<String, u64>>,
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
        tenant: &str,
        topic_name: &str,
        publish: &Publish,
        publish_properties: &Option<PublishProperties>,
    ) -> ResultMqttBrokerError {
        if !publish.retain {
            return Ok(());
        }

        let topic_storage = RetainStorage::new(self.client_pool.clone());
        let had_retain = self.contain_retain(tenant, topic_name).await?;

        if had_retain && publish.payload.is_empty() {
            topic_storage
                .delete_retain_message(tenant, topic_name)
                .await?;
            record_mqtt_retained_dec();

            self.topic_retain_data
                .entry(tenant.to_string())
                .or_default()
                .remove(topic_name);

            self.last_update_time
                .entry(tenant.to_string())
                .or_default()
                .remove(topic_name);
        }

        if !publish.payload.is_empty() {
            record_retain_recv_metrics(publish.qos);
            if !had_retain {
                record_mqtt_retained_inc();
            }

            let expired_at = build_message_expire(&self.cache_manager, publish_properties).await;
            let retain_message = MQTTRetainMessage {
                tenant: tenant.to_string(),
                topic_name: topic_name.to_string(),
                payload: publish.payload.clone(),
                expired_at,
                create_time: now_second(),
            };

            topic_storage
                .set_retain_message(tenant, topic_name, &retain_message)
                .await?;

            self.topic_retain_data
                .entry(tenant.to_string())
                .or_default()
                .insert(topic_name.to_string(), retain_message.clone());

            self.last_update_time
                .entry(tenant.to_string())
                .or_default()
                .insert(topic_name.to_string(), now_second());
        }

        Ok(())
    }

    pub fn get_retain_message(&self, tenant: &str, topic_name: &str) -> Option<MQTTRetainMessage> {
        self.topic_retain_data
            .get(tenant)
            .and_then(|m| m.get(topic_name).map(|v| v.clone()))
    }

    pub async fn try_send_retain_message(
        &self,
        tenant: &str,
        client_id: &str,
        subscribe: &Subscribe,
        cache_manager: &Arc<MQTTCacheManager>,
        subscribe_manager: &Arc<SubscribeManager>,
    ) -> Result<(), MqttBrokerError> {
        let is_new_subs = is_new_sub(tenant, client_id, subscribe, subscribe_manager);
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
                if !self.contain_retain(tenant, &topic_name).await? {
                    continue;
                }

                let data = SendRetainData {
                    tenant: tenant.to_string(),
                    client_id: client_id.to_string(),
                    topic_name,
                };
                if let Err(e) = self.send_retain_message_queue.send(data).await {
                    return Err(MqttBrokerError::CommonError(e.to_string()));
                }
            }
        }
        Ok(())
    }

    async fn contain_retain(&self, tenant: &str, topic: &str) -> Result<bool, MqttBrokerError> {
        let current_time = now_second();

        let cache_age = self
            .last_update_time
            .get(tenant)
            .and_then(|m| m.get(topic).map(|v| current_time.saturating_sub(*v)));

        // Load from storage if never cached or cache is stale (>= 5s)
        if cache_age.is_none_or(|age| age >= 5) {
            self.load_retain_from_storage(tenant, topic).await?;
        }

        let Some(msg) = self
            .topic_retain_data
            .get(tenant)
            .and_then(|m| m.get(topic).map(|v| v.clone()))
        else {
            return Ok(false);
        };

        if msg.expired_at > 0 && current_time >= msg.expired_at {
            self.clear_local_retain_cache(tenant, topic);
            return Ok(false);
        }

        Ok(true)
    }

    async fn load_retain_from_storage(
        &self,
        tenant: &str,
        topic: &str,
    ) -> Result<(), MqttBrokerError> {
        let topic_storage = RetainStorage::new(self.client_pool.clone());
        let retain_message = topic_storage.get_retain_message(tenant, topic).await?;

        if let Some(data) = retain_message {
            self.topic_retain_data
                .entry(tenant.to_string())
                .or_default()
                .insert(topic.to_string(), data);
        } else {
            self.clear_local_retain_cache(tenant, topic);
        }

        let current_time = now_second();
        self.last_update_time
            .entry(tenant.to_string())
            .or_default()
            .insert(topic.to_string(), current_time);

        Ok(())
    }

    fn clear_local_retain_cache(&self, tenant: &str, topic: &str) {
        self.topic_retain_data
            .entry(tenant.to_string())
            .or_default()
            .remove(topic);

        self.last_update_time
            .entry(tenant.to_string())
            .or_default()
            .remove(topic);
    }

    async fn send_retain_message(
        &self,
        data: &SendRetainData,
        stop_sx: &broadcast::Sender<bool>,
    ) -> ResultMqttBrokerError {
        let retain_message =
            if let Some(msg) = self.get_retain_message(&data.tenant, &data.topic_name) {
                msg
            } else {
                return Ok(());
            };

        if now_second() > retain_message.expired_at {
            self.clear_local_retain_cache(&retain_message.tenant, &retain_message.topic_name);
            return Ok(());
        }

        let qos = QoS::AtLeastOnce;
        let p_kid = self
            .cache_manager
            .pkid_manager
            .generate_publish_to_client_pkid(&data.client_id, &qos)
            .await;

        let publish = Publish {
            dup: false,
            qos,
            p_kid,
            retain: false,
            topic: Bytes::copy_from_slice(retain_message.topic_name.as_bytes()),
            payload: retain_message.payload.clone(),
        };

        let publish_properties = PublishProperties {
            user_properties: vec![(
                SUB_RETAIN_MESSAGE_PUSH_FLAG.to_string(),
                SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE.to_string(),
            )],
            ..Default::default()
        };

        let packet = MqttPacket::Publish(publish, Some(publish_properties));

        let sub_pub_param = SubPublishParam {
            packet,
            create_time: now_second(),
            client_id: data.client_id.clone(),
            p_kid,
            qos,
        };

        send_publish_packet_to_client(
            &self.connection_manager,
            &self.cache_manager,
            &sub_pub_param,
            stop_sx,
        )
        .await?;

        record_retain_sent_metrics(qos);

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
                        Ok(true) => {
                            debug!("Retain message send thread stopped by stop signal.");
                            break;
                        }
                        Ok(false) => {}
                        Err(broadcast::error::RecvError::Closed) => {
                            debug!("Retain message send thread stop channel closed, exiting.");
                            break;
                        }
                        Err(broadcast::error::RecvError::Lagged(skipped)) => {
                            debug!(
                                "Retain message send thread stop channel lagged, skipped {} messages.",
                                skipped
                            );
                        }
                    }
                }
                res = retain_message_rx.recv()  => {
                    match res {
                        Some(data) => {
                            let raw_retain_message_manager = retain_message_manager.clone();
                            let raw_stop_sx = stop_sx.clone();
                            let semaphore_clone = semaphore.clone();
                            let permit = match semaphore_clone.acquire_owned().await {
                                Ok(permit) => permit,
                                Err(e) => {
                                    error!(
                                        "Failed to acquire semaphore permit for sending retain message, \
                                        semaphore may be closed. client_id={}, topic={}, error={}",
                                        data.client_id, data.topic_name, e
                                    );
                                    continue;
                                }
                            };
                            tokio::spawn(async move{
                                let _permit = permit;
                                debug!(
                                    "Processing retain message: client_id={}, topic={}",
                                    data.client_id, data.topic_name
                                );

                                if let Err(e) = raw_retain_message_manager.send_retain_message(&data, &raw_stop_sx).await{
                                    if !client_unavailable_error(&e) {
                                        warn!(
                                            "Sending retain message failed: client_id={}, topic={}, error={}",
                                            data.client_id, data.topic_name, e
                                        );
                                    }
                                }
                            });
                        }
                        None => {
                            debug!("Retain message send thread channel closed, exiting.");
                            break;
                        }
                    }
                }
            }
        }
    });
}
