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

use super::cache::CacheManager;
use super::constant::{SUB_RETAIN_MESSAGE_PUSH_FLAG, SUB_RETAIN_MESSAGE_PUSH_FLAG_VALUE};
use super::error::MqttBrokerError;
use super::message::build_message_expire;
use crate::handler::sub_option::{
    get_retain_flag_by_retain_as_published, is_send_msg_by_bo_local,
    is_send_retain_msg_by_retain_handling,
};
use crate::observability::metrics::packets::{
    record_retain_recv_metrics, record_retain_sent_metrics,
};
use crate::server::connection_manager::ConnectionManager;
use crate::storage::topic::TopicStorage;
use crate::subscribe::common::SubPublishParam;
use crate::subscribe::common::Subscriber;
use crate::subscribe::common::{get_pkid, get_sub_topic_id_list, min_qos};
use crate::subscribe::manager::SubscribeManager;
use crate::subscribe::push::publish_data;
use bytes::Bytes;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::message::MqttMessage;
use protocol::mqtt::common::{
    MqttProtocol, Publish, PublishProperties, Subscribe, SubscribeProperties,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::error;

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
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    topic_name: String,
    client_id: &str,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> Result<(), MqttBrokerError> {
    if !publish.retain {
        return Ok(());
    }

    let topic_storage = TopicStorage::new(client_pool.clone());

    if publish.payload.is_empty() {
        topic_storage
            .delete_retain_message(topic_name.clone())
            .await?;
        cache_manager.update_topic_retain_message(&topic_name, Some(Vec::new()));
    } else {
        record_retain_recv_metrics(publish.qos);
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

#[allow(clippy::too_many_arguments)]
pub async fn try_send_retain_message(
    protocol: MqttProtocol,
    client_id: String,
    subscribe: Subscribe,
    subscribe_properties: Option<SubscribeProperties>,
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    is_new_subs: DashMap<String, bool>,
) {
    tokio::spawn(async move {
        let (stop_sx, _) = broadcast::channel(1);
        if let Err(e) = send_retain_message(
            &protocol,
            &client_id,
            &subscribe,
            &subscribe_properties,
            &client_pool,
            &cache_manager,
            &connection_manager,
            &stop_sx,
            &is_new_subs,
        )
        .await
        {
            error!("Sending retain message failed with error message :{}", e);
        }
    });
}

#[allow(clippy::too_many_arguments)]
async fn send_retain_message(
    protocol: &MqttProtocol,
    client_id: &str,
    subscribe: &Subscribe,
    subscribe_properties: &Option<SubscribeProperties>,
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<CacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    stop_sx: &broadcast::Sender<bool>,
    is_new_subs: &DashMap<String, bool>,
) -> Result<(), MqttBrokerError> {
    let mut sub_ids = Vec::new();
    if let Some(properties) = subscribe_properties {
        if let Some(id) = properties.subscription_identifier {
            sub_ids.push(id);
        }
    }

    for filter in subscribe.filters.iter() {
        if !is_send_retain_msg_by_retain_handling(
            &filter.path,
            &filter.retain_handling,
            is_new_subs,
        ) {
            continue;
        }

        let topic_id_list = get_sub_topic_id_list(cache_manager, &filter.path).await;
        let topic_storage = TopicStorage::new(client_pool.clone());
        let cluster = cache_manager.get_cluster_info();

        for topic_id in topic_id_list.iter() {
            let topic_name = if let Some(topic_name) = cache_manager.topic_name_by_id(topic_id) {
                topic_name
            } else {
                continue;
            };

            let msg = if let Some(message) = topic_storage.get_retain_message(&topic_name).await? {
                message
            } else {
                continue;
            };

            if !is_send_msg_by_bo_local(filter.nolocal, client_id, &msg.client_id) {
                continue;
            }

            let retain = get_retain_flag_by_retain_as_published(filter.preserve_retain, msg.retain);
            let qos = min_qos(cluster.protocol.max_qos, filter.qos);

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

            let pkid = get_pkid();
            let publish = Publish {
                dup: false,
                qos,
                pkid,
                retain,
                topic: Bytes::from(topic_name.clone()),
                payload: msg.payload,
            };

            let sub_pub_param = SubPublishParam::new(
                Subscriber {
                    protocol: protocol.to_owned(),
                    client_id: client_id.to_string(),
                    ..Default::default()
                },
                publish,
                Some(properties),
                msg.create_time as u128,
                "".to_string(),
                pkid,
            );

            publish_data(connection_manager, cache_manager, sub_pub_param, stop_sx).await?;
            record_retain_sent_metrics(qos);
        }
    }
    Ok(())
}
