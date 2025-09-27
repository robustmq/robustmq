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

use super::{
    cache::MQTTCacheManager,
    delay_message::{
        DelayPublishTopic, DELAY_MESSAGE_FLAG, DELAY_MESSAGE_RECV_MS, DELAY_MESSAGE_TARGET_MS,
    },
    error::MqttBrokerError,
    message::build_message_expire,
    retain::save_retain_message,
};
use crate::{storage::message::MessageStorage, subscribe::manager::SubscribeManager};
use common_base::tools::now_second;
use common_metrics::mqtt::packets::record_messages_dropped_no_subscribers_metrics;
use delay_message::DelayMessageManager;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::{message::MqttMessage, topic::MQTTTopic};
use protocol::mqtt::common::{Publish, PublishProperties};
use storage_adapter::storage::ArcStorageAdapter;

pub fn is_exist_subscribe(subscribe_manager: &Arc<SubscribeManager>, topic: &str) -> bool {
    subscribe_manager.contain_topic_subscribe(topic)
}

#[derive(Clone)]
pub struct SaveMessageContext {
    pub message_storage_adapter: ArcStorageAdapter,
    pub delay_message_manager: Arc<DelayMessageManager>,
    pub cache_manager: Arc<MQTTCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub publish: Publish,
    pub publish_properties: Option<PublishProperties>,
    pub subscribe_manager: Arc<SubscribeManager>,
    pub client_id: String,
    pub topic: MQTTTopic,
    pub delay_info: Option<DelayPublishTopic>,
}

pub async fn save_message(context: SaveMessageContext) -> Result<Option<String>, MqttBrokerError> {
    let offline_message_disabled = !context
        .cache_manager
        .broker_cache
        .get_cluster_config()
        .mqtt_offline_message
        .enable;
    let not_exist_subscribe =
        !is_exist_subscribe(&context.subscribe_manager, &context.topic.topic_name);
    if offline_message_disabled && not_exist_subscribe {
        record_messages_dropped_no_subscribers_metrics(context.publish.qos);
        return Ok(None);
    }

    let message_expire = build_message_expire(&context.cache_manager, &context.publish_properties);

    if context.delay_info.is_some() {
        return save_delay_message(
            &context.delay_message_manager,
            &context.publish,
            &context.publish_properties,
            &context.client_id,
            message_expire,
            context.delay_info.as_ref().unwrap(),
        )
        .await;
    }

    // Persisting retain message data
    save_retain_message(
        &context.cache_manager,
        &context.client_pool,
        context.topic.topic_name.clone(),
        &context.client_id,
        &context.publish,
        &context.publish_properties,
    )
    .await?;

    return save_simple_message(
        &context.message_storage_adapter,
        &context.publish,
        &context.publish_properties,
        &context.client_id,
        &context.topic,
        message_expire,
    )
    .await;
}

async fn save_delay_message(
    delay_message_manager: &Arc<DelayMessageManager>,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
    client_id: &str,
    message_expire: u64,
    delay_info: &DelayPublishTopic,
) -> Result<Option<String>, MqttBrokerError> {
    let new_publish_properties = if let Some(mut properties) = publish_properties.clone() {
        properties.user_properties = vec![
            (DELAY_MESSAGE_FLAG.to_string(), "true".to_string()),
            (DELAY_MESSAGE_RECV_MS.to_string(), now_second().to_string()),
            (
                DELAY_MESSAGE_TARGET_MS.to_string(),
                (now_second() + delay_info.delay_timestamp).to_string(),
            ),
        ];
        properties
    } else {
        PublishProperties {
            user_properties: vec![
                (DELAY_MESSAGE_FLAG.to_string(), "true".to_string()),
                (DELAY_MESSAGE_RECV_MS.to_string(), now_second().to_string()),
                (
                    DELAY_MESSAGE_TARGET_MS.to_string(),
                    (now_second() + delay_info.delay_timestamp).to_string(),
                ),
            ],
            ..Default::default()
        }
    };

    if let Some(record) = MqttMessage::build_record(
        client_id,
        publish,
        &Some(new_publish_properties),
        message_expire,
    ) {
        let target_shard_name = delay_info.tagget_shard_name.as_ref().unwrap();
        delay_message_manager
            .send(target_shard_name, delay_info.delay_timestamp, record)
            .await?;
        return Ok(None);
    }

    Err(MqttBrokerError::FailedToBuildMessage)
}

async fn save_simple_message(
    message_storage_adapter: &ArcStorageAdapter,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
    client_id: &str,
    topic: &MQTTTopic,
    message_expire: u64,
) -> Result<Option<String>, MqttBrokerError> {
    if let Some(record) =
        MqttMessage::build_record(client_id, publish, publish_properties, message_expire)
    {
        let message_storage = MessageStorage::new(message_storage_adapter.clone());
        let offsets = message_storage
            .append_topic_message(&topic.topic_id, vec![record])
            .await?;
        return Ok(Some(format!("{offsets:?}")));
    }

    Err(MqttBrokerError::FailedToBuildMessage)
}
