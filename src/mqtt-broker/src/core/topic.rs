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
use crate::core::cache::MQTTCacheManager;
use crate::core::tool::ResultMqttBrokerError;
use crate::subscribe::manager::SubscribeManager;
use common_base::tools::now_second;
use common_base::uuid::unique_id;
use common_config::storage::StorageType;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::topic::Topic;

use metadata_struct::storage::shard::EngineShardConfig;
use protocol::mqtt::common::{Publish, PublishProperties};
use rocksdb_engine::metrics::mqtt::MQTTMetricsCache;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::{driver::StorageDriverManager, topic::create_topic_full};
use tokio::time::sleep;

/// Validates MQTT topic name according to MQTT 5.0 specification
///
/// Rules:
/// - Must not be empty
/// - Must be valid UTF-8
/// - Must not contain null character (U+0000)
/// - Must not contain wildcard characters (+ or #) - only allowed in subscriptions
/// - Length must not exceed 65535 bytes
/// - Can contain empty levels (e.g., "a//b" or "/a/b" are valid)
/// - Topics starting with "$" are system topics
pub fn topic_name_validator(topic_name: &str) -> ResultMqttBrokerError {
    if topic_name.is_empty() {
        return Err(MqttBrokerError::TopicNameIsEmpty);
    }

    if topic_name.len() > 65535 {
        return Err(MqttBrokerError::TopicNameIncorrectlyFormatted(format!(
            "Topic name exceeds maximum length of 65535 bytes: {}",
            topic_name
        )));
    }

    if topic_name.contains('\0') {
        return Err(MqttBrokerError::TopicNameIncorrectlyFormatted(format!(
            "Topic name contains null character: {}",
            topic_name
        )));
    }

    if topic_name.contains('+') {
        return Err(MqttBrokerError::TopicNameIncorrectlyFormatted(format!(
            "Topic name contains wildcard '+': {}",
            topic_name
        )));
    }

    if topic_name.contains('#') {
        return Err(MqttBrokerError::TopicNameIncorrectlyFormatted(format!(
            "Topic name contains wildcard '#': {}",
            topic_name
        )));
    }

    for c in topic_name.chars() {
        if c.is_control() && c != '\t' && c != '\n' && c != '\r' {
            return Err(MqttBrokerError::TopicNameIncorrectlyFormatted(format!(
                "Topic name contains invalid control character: {}",
                topic_name
            )));
        }
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
    storage_driver_manager: &Arc<StorageDriverManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<Topic, MqttBrokerError> {
    let topic = if let Some(tp) = metadata_cache.broker_cache.get_topic_by_name(topic_name) {
        tp
    } else {
        let uid = unique_id();
        let topic = Topic {
            topic_id: uid.clone(),
            topic_name: topic_name.to_string(),
            storage_type: StorageType::EngineRocksDB,
            partition: 1,
            replication: 1,
            storage_name_list: Topic::create_partition_name(&uid, 1),
            create_time: now_second(),
        };

        let shard_config = EngineShardConfig {
            replica_num: 1,
            storage_type: StorageType::EngineRocksDB,

            ..Default::default()
        };
        create_topic_full(
            &metadata_cache.broker_cache,
            storage_driver_manager,
            client_pool,
            &topic,
            &shard_config,
        )
        .await?;
        topic
    };
    Ok(topic)
}

pub async fn delete_topic(
    cache_manager: &Arc<MQTTCacheManager>,
    topic_name: &str,
    storage_driver_manager: &Arc<StorageDriverManager>,
    subscribe_manager: &Arc<SubscribeManager>,
    metrics_manager: &Arc<MQTTMetricsCache>,
) -> Result<(), MqttBrokerError> {
    storage_driver_manager
        .delete_storage_resource(topic_name)
        .await?;
    cache_manager.broker_cache.delete_topic(topic_name);
    metrics_manager.remove_topic(topic_name)?;
    subscribe_manager.remove_by_topic(topic_name);
    Ok(())
}

#[cfg(test)]
mod test {
    use super::topic_name_validator;

    #[test]
    pub fn topic_name_validator_test() {
        assert!(topic_name_validator("").is_err());
        assert!(topic_name_validator("/test/test").is_ok());
        assert!(topic_name_validator("test/test/").is_ok());
        assert!(topic_name_validator("$SYS/broker/clients").is_ok());
        assert!(topic_name_validator("test/1").is_ok());
        assert!(topic_name_validator(
            "/sys/request_response/response/1eb1f833e0de4169908acedec8eb62f7"
        )
        .is_ok());
        assert!(topic_name_validator("a//b").is_ok());
        assert!(topic_name_validator("sensor/+/temperature").is_err());
        assert!(topic_name_validator("sensor/#").is_err());
        assert!(topic_name_validator("test\0topic").is_err());
        assert!(topic_name_validator("device/123/sensor-data_2024").is_ok());
        assert!(topic_name_validator("传感器/温度").is_ok());
        assert!(topic_name_validator(&"a/".repeat(40000)).is_err());
    }
}
