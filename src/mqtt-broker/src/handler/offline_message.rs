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

use super::{cache::CacheManager, error::MqttBrokerError, message::build_message_expire};
use crate::{
    observability::metrics::packets::record_messages_dropped_no_subscribers_metrics,
    storage::message::MessageStorage, subscribe::subscribe_manager::SubscribeManager,
};
use metadata_struct::mqtt::{message::MqttMessage, topic::MqttTopic};
use protocol::mqtt::common::{Publish, PublishProperties};
use storage_adapter::storage::StorageAdapter;

pub fn is_exist_subscribe(subscribe_manager: &Arc<SubscribeManager>, topic: &str) -> bool {
    subscribe_manager.contain_topic_subscribe(topic)
}

pub async fn save_message<S>(
    message_storage_adapter: &Arc<S>,
    cache_manager: &Arc<CacheManager>,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
    subscribe_manager: &Arc<SubscribeManager>,
    client_id: &str,
    topic: &MqttTopic,
) -> Result<Option<String>, MqttBrokerError>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    // If Topic is not subscribed and offline messaging is not enabled. The message is not saved.
    if !is_exist_subscribe(subscribe_manager, &topic.topic_name)
        && !cache_manager.get_cluster_info().offline_message.enable
    {
        record_messages_dropped_no_subscribers_metrics(publish.qos);
        return Ok(None);
    }

    let message_storage = MessageStorage::new(message_storage_adapter.clone());
    let message_expire = build_message_expire(cache_manager, publish_properties);
    let offset = if let Some(record) =
        MqttMessage::build_record(client_id, publish, publish_properties, message_expire)
    {
        let offsets = message_storage
            .append_topic_message(&topic.topic_id, vec![record])
            .await?;
        Some(format!("{:?}", offsets))
    } else {
        None
    };
    Ok(offset)
}
