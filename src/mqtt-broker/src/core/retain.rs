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
use super::constant::{SUB_RETAIN_MESSAGE_PUSH_FLAG, SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE};
use super::message::build_message_expire;
use crate::core::error::MqttBrokerError;
use crate::core::sub_option::{
    get_retain_flag_by_retain_as_published, is_send_msg_by_bo_local,
    is_send_retain_msg_by_retain_handling,
};
use crate::core::tool::ResultMqttBrokerError;
use crate::storage::topic::TopicStorage;
use crate::subscribe::common::min_qos;
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
    qos, MqttPacket, MqttProtocol, Publish, PublishProperties, QoS, Subscribe, SubscribeProperties,
};
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast;
use tracing::{debug, error, warn};

#[derive(Clone)]
pub struct TrySendRetainMessageContext {
    pub protocol: MqttProtocol,
    pub client_id: String,
    pub subscribe: Subscribe,
    pub subscribe_properties: Option<SubscribeProperties>,
    pub client_pool: Arc<ClientPool>,
    pub cache_manager: Arc<MQTTCacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub is_new_subs: DashMap<String, bool>,
}

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
    send_retain_message_queue: broadcast::Sender<SendRetainData>,
}

impl RetainMessageManager {
    pub fn new(
        cache_manager: Arc<MQTTCacheManager>,
        client_pool: Arc<ClientPool>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        let (send_retain_message_queue, _) = broadcast::channel::<SendRetainData>(100);

        RetainMessageManager {
            cache_manager,
            client_pool,
            connection_manager,
            topic_retain_data: DashMap::new(),
            last_update_time: DashMap::new(),
            send_retain_message_queue,
        }
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

        if self.contain_retain(topic_name).await? && publish.payload.is_empty() {
            topic_storage.delete_retain_message(topic_name).await?;
            record_mqtt_retained_dec();
        }

        if !publish.payload.is_empty() {
            record_retain_recv_metrics(publish.qos);
            record_mqtt_retained_inc();
            let message_expire =
                build_message_expire(&self.cache_manager, publish_properties).await;
            let retain_message =
                MqttMessage::build_message(client_id, publish, publish_properties, message_expire);
            topic_storage
                .set_retain_message(topic_name, &retain_message, message_expire)
                .await?;
        }

        Ok(())
    }

    pub async fn try_send_retain_message(
        &self,
        context: TrySendRetainMessageContext,
    ) -> Result<(), MqttBrokerError> {
        for filter in context.subscribe.filters.iter() {
            if !is_send_retain_msg_by_retain_handling(
                &filter.path,
                &filter.retain_handling,
                &context.is_new_subs,
            ) {
                debug!("retain messages: Determine whether to send retained messages based on the retain handling strategy. Client ID: {}", context.client_id);
                continue;
            }

            let topic_name_list =
                get_sub_topic_name_list(&context.cache_manager, &filter.path).await;
            if topic_name_list.is_empty() {
                continue;
            }

            for topic_name in topic_name_list {
                if !self.contain_retain(&topic_name).await? {
                    continue;
                }

                if !is_send_retain_msg_by_retain_handling(
                    &filter.path,
                    &filter.retain_handling,
                    &context.is_new_subs,
                ) {
                    continue;
                }

                let mut sub_ids = Vec::new();
                if let Some(properties) = context.subscribe_properties.clone() {
                    if let Some(id) = properties.subscription_identifier {
                        sub_ids.push(id);
                    }
                }
                let data = SendRetainData {
                    client_id: context.client_id.clone(),
                    no_local: filter.nolocal,
                    preserve_retain: filter.preserve_retain,
                    qos: filter.qos,
                    sub_ids,
                    topic_name: topic_name.to_string(),
                };
                if let Err(e) = self.send_retain_message_queue.send(data) {
                    return Err(MqttBrokerError::CommonError(e.to_string()));
                }
            }
        }
        Ok(())
    }

    async fn contain_retain(&self, topic: &str) -> Result<bool, MqttBrokerError> {
        let last_update_time = if let Some(update_time) = self.last_update_time.get(topic) {
            *update_time
        } else {
            let topic_storage = TopicStorage::new(self.client_pool.clone());
            let (message, message_at) = topic_storage.get_retain_message(topic).await?;
            self.last_update_time
                .insert(topic.to_string(), now_second());
            if let Some(msg) = message {
                if let Some(msg_at) = message_at {
                    self.topic_retain_data
                        .insert(topic.to_string(), Some((msg, msg_at)));
                }
            }
            now_second()
        };

        if now_second() - last_update_time >= 5 {
            let topic_storage = TopicStorage::new(self.client_pool.clone());
            let (message, message_at) = topic_storage.get_retain_message(topic).await?;
            self.last_update_time
                .insert(topic.to_string(), now_second());
            if let Some(msg) = message {
                if let Some(msg_at) = message_at {
                    self.topic_retain_data
                        .insert(topic.to_string(), Some((msg, msg_at)));
                }
            }
        }

        let contain = if let Some(data) = self.topic_retain_data.get(topic) {
            data.is_some()
        } else {
            false
        };
        Ok(contain)
    }

    async fn send_retain_message(
        &self,
        data: &SendRetainData,
        stop_sx: &broadcast::Sender<bool>,
    ) -> ResultMqttBrokerError {
        let cluster = self.cache_manager.broker_cache.get_cluster_config().await;

        let msg = if let Some(data) = self.topic_retain_data.get(&data.topic_name) {
            if let Some((message, message_at)) = data.clone() {
                if message_at <= now_second() {
                    message
                } else {
                    return Ok(());
                }
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        if !is_send_msg_by_bo_local(data.no_local, &data.client_id, &msg.client_id) {
            return Ok(());
        }

        let retain = get_retain_flag_by_retain_as_published(data.preserve_retain, msg.retain);
        let qos = min_qos(qos(cluster.mqtt_protocol_config.max_qos).unwrap(), data.qos);

        let mut user_properties = msg.user_properties.unwrap_or_default();
        user_properties.push((
            SUB_RETAIN_MESSAGE_PUSH_FLAG.to_string(),
            SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE.to_string(),
        ));

        let properties = PublishProperties {
            payload_format_indicator: msg.format_indicator,
            message_expiry_interval: Some(msg.expiry_interval as u32),
            topic_alias: None,
            response_topic: msg.response_topic,
            correlation_data: msg.correlation_data,
            user_properties,
            subscription_identifiers: data.sub_ids.clone(),
            content_type: msg.content_type,
        };

        let p_kid = self
            .cache_manager
            .pkid_metadata
            .generate_pkid(&data.client_id, &qos)
            .await;

        let publish = Publish {
            dup: false,
            qos,
            p_kid,
            retain,
            topic: Bytes::copy_from_slice(data.topic_name.as_bytes()),
            payload: msg.payload,
        };

        let packet = MqttPacket::Publish(publish.clone(), Some(properties));

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
        debug!(
            "retain the successful message sending: client_id: {}, topi_id: {}",
            data.client_id, data.topic_name
        );

        record_retain_sent_metrics(qos);

        Ok(())
    }
}

pub async fn is_new_sub(
    client_id: &str,
    subscribe: &Subscribe,
    subscribe_manager: &Arc<SubscribeManager>,
) -> DashMap<String, bool> {
    let results = DashMap::with_capacity(2);
    for filter in subscribe.filters.iter() {
        let bool = subscribe_manager
            .get_subscribe(client_id, &filter.path)
            .is_none();
        results.insert(filter.path.to_owned(), bool);
    }
    results
}

pub fn start_send_retain_thread(
    retain_message_manager: Arc<RetainMessageManager>,
    stop_sx: broadcast::Sender<bool>,
) {
    tokio::spawn(async move {
        let mut stop_recv = stop_sx.subscribe();
        let mut retain_message_rx = retain_message_manager.send_retain_message_queue.subscribe();
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
                    match res{
                        Ok(data) => {
                            let raw_retain_message_manager = retain_message_manager.clone();
                            let raw_stop_sx = stop_sx.clone();
                            tokio::spawn(async move{
                                if let Err(e) = raw_retain_message_manager.send_retain_message(&data, &raw_stop_sx).await{
                                    if !client_unavailable_error(&e) {
                                        warn!( "Sending retain message failed with error message :{},client_id:{}", e, data.client_id);
                                    }
                                }
                            });
                        }
                        Err(e) =>{
                            error!("{:?}",e);
                        }
                    }
                }
            }
        }
    });
}
