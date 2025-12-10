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

use super::error::MqttBrokerError;
use crate::handler::cache::MQTTCacheManager;
use crate::handler::tool::ResultMqttBrokerError;
use crate::storage::topic::TopicStorage;
use crate::subscribe::manager::SubscribeManager;
use bytes::Bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::MQTTTopic;
use protocol::mqtt::common::{Publish, PublishProperties};
use regex::Regex;
use rocksdb_engine::metrics::mqtt::MQTTMetricsCache;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::storage::{ArcStorageAdapter, ShardInfo};
use tokio::time::sleep;

pub fn payload_format_validator(
    payload: &Bytes,
    payload_format_indicator: u8,
    max_packet_size: usize,
) -> bool {
    if payload.is_empty() || payload.len() > max_packet_size {
        return false;
    }

    if payload_format_indicator == 1 {
        return std::str::from_utf8(payload.to_vec().as_slice()).is_ok();
    }

    false
}

pub fn topic_name_validator(topic_name: &str) -> ResultMqttBrokerError {
    if topic_name.is_empty() {
        return Err(MqttBrokerError::TopicNameIsEmpty);
    }

    let topic_slice: Vec<&str> = topic_name.split("/").collect();
    if topic_slice.first().unwrap() == &"/" {
        return Err(MqttBrokerError::TopicNameIncorrectlyFormatted(
            topic_name.to_owned(),
        ));
    }

    if topic_slice.last().unwrap() == &"/" {
        return Err(MqttBrokerError::TopicNameIncorrectlyFormatted(
            topic_name.to_owned(),
        ));
    }

    let format_str = "^[A-Za-z0-9_+#$/\\-]+$";
    let re = Regex::new(format_str).unwrap();
    if !re.is_match(topic_name) {
        return Err(MqttBrokerError::TopicNameIncorrectlyFormatted(
            topic_name.to_owned(),
        ));
    }
    Ok(())
}

pub async fn get_topic_name(
    cache_manager: &Arc<MQTTCacheManager>,
    connect_id: u64,
    publish: &Publish,
    publish_properties: &Option<PublishProperties>,
) -> Result<String, MqttBrokerError> {
    let topic = String::from_utf8(publish.topic.to_vec())?;

    let topic_alias = if let Some(pub_properties) = publish_properties {
        pub_properties.topic_alias
    } else {
        None
    };

    if topic.is_empty() && topic_alias.is_none() {
        return Err(MqttBrokerError::TopicNameIsEmpty);
    }

    let mut topic_name = if topic.is_empty() {
        get_topic_alias(cache_manager, connect_id, topic_alias).await?
    } else {
        topic
    };

    if !cache_manager.topic_is_validator.contains_key(&topic_name) {
        topic_name_validator(&topic_name)?;
        cache_manager.add_topic_is_validator(&topic_name);
    }

    if let Some(new_topic_name) = cache_manager.get_new_rewrite_name(&topic_name) {
        topic_name = new_topic_name;
    }

    Ok(topic_name)
}

pub async fn get_topic_alias(
    cache_manager: &Arc<MQTTCacheManager>,
    connect_id: u64,
    topic_alias: Option<u16>,
) -> Result<String, MqttBrokerError> {
    let mut times = 0;
    let topic_name;
    loop {
        if times > 10 {
            return Err(MqttBrokerError::TopicAliasInvalid(topic_alias));
        }
        if let Some(tn) = cache_manager.get_topic_alias(connect_id, topic_alias.unwrap()) {
            topic_name = Some(tn);
            break;
        } else {
            times += 1;
            sleep(Duration::from_millis(50)).await;
            continue;
        }
    }
    if let Some(tn) = topic_name {
        return Ok(tn);
    }
    Err(MqttBrokerError::TopicAliasInvalid(topic_alias))
}

pub async fn try_init_topic(
    topic_name: &str,
    metadata_cache: &Arc<MQTTCacheManager>,
    message_storage_adapter: &ArcStorageAdapter,
    client_pool: &Arc<ClientPool>,
) -> Result<MQTTTopic, MqttBrokerError> {
    let topic = if let Some(tp) = metadata_cache.get_topic_by_name(topic_name) {
        tp
    } else {
        // create Topic
        let topic_storage = TopicStorage::new(client_pool.clone());
        let topic = if let Some(topic) = topic_storage.get_topic(topic_name).await? {
            topic
        } else {
            let topic = MQTTTopic::new(topic_name.to_owned());
            topic_storage.save_topic(topic.clone()).await?;
            topic
        };

        metadata_cache.add_topic(topic_name, &topic);

        // Create the resource object of the storage layer
        let list = message_storage_adapter.list_shard(topic_name).await?;
        if list.is_empty() {
            let shard = ShardInfo {
                shard_name: topic_name.to_owned(),
                replica_num: 1,
                ..Default::default()
            };
            message_storage_adapter.create_shard(&shard).await?;
        }
        metadata_cache.set_re_calc_topic_rewrite(true).await;
        return Ok(topic);
    };
    Ok(topic)
}

pub async fn delete_topic(
    cache_manager: &Arc<MQTTCacheManager>,
    topic_name: &str,
    message_storage_adapter: &ArcStorageAdapter,
    subscribe_manager: &Arc<SubscribeManager>,
    metrics_manager: &Arc<MQTTMetricsCache>,
) -> Result<(), MqttBrokerError> {
    // delete shard
    let list = message_storage_adapter.list_shard(topic_name).await?;
    if !list.is_empty() {
        message_storage_adapter.delete_shard(topic_name).await?;
    }

    cache_manager.delete_topic(topic_name);
    metrics_manager.remove_topic(topic_name)?;
    subscribe_manager.remove_by_topic(topic_name);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::topic_name_validator;
    use crate::handler::error::MqttBrokerError;

    #[test]
    pub fn topic_name_validator_test() {
        let topic_name = "".to_string();
        if let Err(e) = topic_name_validator(&topic_name) {
            // assert!(e.to_string() == MqttBrokerError::TopicNameIsEmpty.to_string())
            assert_eq!(e.to_string(), MqttBrokerError::TopicNameIsEmpty.to_string());
        }

        let topic_name = "/test/test".to_string();
        topic_name_validator(&topic_name).unwrap();

        let topic_name = "test/test/".to_string();
        topic_name_validator(&topic_name).unwrap();

        let topic_name = "test/$1".to_string();
        if let Err(err) = topic_name_validator(&topic_name) {
            println!("{err:?}");
        }

        let topic_name = "test/1".to_string();
        topic_name_validator(&topic_name).unwrap();

        let topic_name =
            "/sys/request_response/response/1eb1f833e0de4169908acedec8eb62f7".to_string();
        topic_name_validator(&topic_name).unwrap();
    }

    #[test]
    pub fn gen_rewrite_topic_test() {}
}
