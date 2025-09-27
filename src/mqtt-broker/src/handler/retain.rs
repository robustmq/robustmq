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
use crate::common::types::ResultMqttBrokerError;
use crate::handler::sub_option::{
    get_retain_flag_by_retain_as_published, is_send_msg_by_bo_local,
    is_send_retain_msg_by_retain_handling,
};
use crate::storage::topic::TopicStorage;
use crate::subscribe::common::Subscriber;
use crate::subscribe::common::{get_sub_topic_id_list, min_qos};
use crate::subscribe::common::{is_ignore_push_error, SubPublishParam};
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::push::send_publish_packet_to_client;
use bytes::Bytes;
use common_metrics::mqtt::packets::{record_retain_recv_metrics, record_retain_sent_metrics};
use common_metrics::mqtt::statistics::{record_mqtt_retained_dec, record_mqtt_retained_inc};
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::message::MqttMessage;
use network_server::common::connection_manager::ConnectionManager;
use protocol::mqtt::common::{
    qos, MqttPacket, MqttProtocol, Publish, PublishProperties, Subscribe, SubscribeProperties,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::time::sleep;
use tracing::{info, warn};

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

pub async fn save_retain_message(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    topic_name: String,
    client_id: &str,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> ResultMqttBrokerError {
    if !publish.retain {
        return Ok(());
    }

    let topic_storage = TopicStorage::new(client_pool.clone());

    if publish.payload.is_empty() {
        topic_storage
            .delete_retain_message(topic_name.clone())
            .await?;
        cache_manager.update_topic_retain_message(&topic_name, Some(Vec::new()));
        record_mqtt_retained_dec();
    } else {
        record_retain_recv_metrics(publish.qos);
        record_mqtt_retained_inc();
        let message_expire = build_message_expire(cache_manager, publish_properties);
        let retain_message =
            MqttMessage::build_message(client_id, publish, publish_properties, message_expire);
        topic_storage
            .set_retain_message(topic_name.clone(), &retain_message, message_expire)
            .await?;

        cache_manager.update_topic_retain_message(&topic_name, Some(retain_message.encode()));
    }

    Ok(())
}

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

pub async fn try_send_retain_message(context: TrySendRetainMessageContext) {
    tokio::spawn(async move {
        // Do not send messages immediately. Avoid publishing and subing in the same connection successively.
        // At this point, the network model is processed in parallel.
        // There may be a situation where the subscription starts before the reserved message is successfully retained,
        // resulting in the subscription end not receiving the reserved message.
        sleep(Duration::from_secs(3)).await;
        let (stop_sx, _) = broadcast::channel(1);
        if let Err(e) = send_retain_message(SendRetainMessageContext {
            protocol: context.protocol.clone(),
            client_id: context.client_id.clone(),
            subscribe: context.subscribe.clone(),
            subscribe_properties: context.subscribe_properties.clone(),
            client_pool: context.client_pool.clone(),
            cache_manager: context.cache_manager.clone(),
            connection_manager: context.connection_manager.clone(),
            stop_sx,
            is_new_subs: context.is_new_subs.clone(),
        })
        .await
        {
            if !is_ignore_push_error(&e) {
                warn!(
                    "Sending retain message failed with error message :{},client_id:{}",
                    e, context.client_id
                );
            }
        }
    });
}

#[derive(Clone)]
pub struct SendRetainMessageContext {
    pub protocol: MqttProtocol,
    pub client_id: String,
    pub subscribe: Subscribe,
    pub subscribe_properties: Option<SubscribeProperties>,
    pub client_pool: Arc<ClientPool>,
    pub cache_manager: Arc<MQTTCacheManager>,
    pub connection_manager: Arc<ConnectionManager>,
    pub stop_sx: broadcast::Sender<bool>,
    pub is_new_subs: DashMap<String, bool>,
}

async fn send_retain_message(context: SendRetainMessageContext) -> ResultMqttBrokerError {
    let mut sub_ids = Vec::new();
    if let Some(properties) = context.subscribe_properties {
        if let Some(id) = properties.subscription_identifier {
            sub_ids.push(id);
        }
    }

    for filter in context.subscribe.filters.iter() {
        if !is_send_retain_msg_by_retain_handling(
            &filter.path,
            &filter.retain_handling,
            &context.is_new_subs,
        ) {
            info!("retain messages: Determine whether to send retained messages based on the retain handling strategy. Client ID: {}", context.client_id);
            continue;
        }

        let topic_id_list = get_sub_topic_id_list(&context.cache_manager, &filter.path).await;
        let topic_storage = TopicStorage::new(context.client_pool.clone());
        let cluster = context.cache_manager.broker_cache.get_cluster_config();

        for topic_id in topic_id_list.iter() {
            let topic_name =
                if let Some(topic_name) = context.cache_manager.topic_name_by_id(topic_id) {
                    topic_name
                } else {
                    continue;
                };

            let msg = if let Some(message) = topic_storage.get_retain_message(&topic_name).await? {
                message
            } else {
                continue;
            };

            if !is_send_msg_by_bo_local(filter.nolocal, &context.client_id, &msg.client_id) {
                info!("retain messages: Determine whether to send retained messages based on the no local strategy. Client ID: {}", context.client_id);
                continue;
            }

            let retain = get_retain_flag_by_retain_as_published(filter.preserve_retain, msg.retain);
            let qos = min_qos(
                qos(cluster.mqtt_protocol_config.max_qos).unwrap(),
                filter.qos,
            );

            let mut user_properties = msg.user_properties;
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
                subscription_identifiers: sub_ids.clone(),
                content_type: msg.content_type,
            };

            let pkid = context
                .cache_manager
                .pkid_metadata
                .generate_pkid(&context.client_id, &qos)
                .await;

            let publish = Publish {
                dup: false,
                qos,
                p_kid: pkid,
                retain,
                topic: Bytes::from(topic_name.clone()),
                payload: msg.payload,
            };

            let packet = MqttPacket::Publish(publish.clone(), Some(properties));

            let sub_pub_param = SubPublishParam::new(
                Subscriber {
                    protocol: context.protocol.to_owned(),
                    client_id: context.client_id.to_string(),
                    ..Default::default()
                },
                packet,
                msg.create_time as u128,
                "".to_string(),
                pkid,
            );

            send_publish_packet_to_client(
                &context.connection_manager,
                &context.cache_manager,
                &sub_pub_param,
                &qos,
                &context.stop_sx,
            )
            .await?;
            info!(
                "retain the successful message sending: client_id: {}, topi_id: {}",
                context.client_id, topic_id
            );

            record_retain_sent_metrics(qos);
        }
    }
    Ok(())
}
