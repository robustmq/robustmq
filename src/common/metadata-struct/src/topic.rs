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

use common_base::{
    error::common::CommonError, tools::now_second, utils::serialize, uuid::unique_id,
};
use common_config::storage::StorageType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::storage::shard::{DEFAULT_MAX_SEGMENT_SIZE, DEFAULT_RETENTION_SEC};

/// Identifies which protocol or subsystem created the topic.
#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub enum TopicSource {
    /// Created by the broker's internal system logic.
    #[default]
    SystemInner,
    MQTT,
    NATS,
    Kafka,
    AMQP,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct Topic {
    pub topic_id: String,
    pub tenant: String,
    pub topic_name: String,
    pub storage_type: StorageType,
    /// Which protocol or subsystem created this topic.
    pub source: TopicSource,
    /// Retention duration in seconds. 0 means no expiry.
    pub ttl: u64,
    pub partition: u32,
    pub replication: u32,
    /// Maps partition index to its storage shard name. Populated via `create_partition_name` or set manually.
    pub storage_name_list: HashMap<u32, String>,
    pub config: TopicConfig,
    pub create_time: u64,
}

impl Topic {
    pub fn new(tenant: &str, topic_name: &str, storage_type: StorageType) -> Self {
        let unique_id = unique_id();
        Topic {
            topic_id: unique_id.clone(),
            tenant: tenant.to_string(),
            topic_name: topic_name.to_string(),
            storage_type,
            source: TopicSource::SystemInner,
            partition: 1,
            replication: 1,
            ttl: 0,
            storage_name_list: Topic::create_partition_name(&unique_id, 1),
            config: TopicConfig::default(),
            create_time: now_second(),
        }
    }

    pub fn with_source(mut self, source: TopicSource) -> Self {
        self.source = source;
        self
    }

    pub fn with_ttl(mut self, ttl: u64) -> Self {
        self.ttl = ttl;
        self
    }

    pub fn with_partition(mut self, partition: u32) -> Self {
        self.partition = partition;
        self.storage_name_list = Topic::create_partition_name(&self.topic_id, partition);
        self
    }

    pub fn with_replication(mut self, replication: u32) -> Self {
        self.replication = replication;
        self
    }

    pub fn with_config(mut self, config: TopicConfig) -> Self {
        self.config = config;
        self
    }

    /// Overrides the storage name list directly, bypassing the auto-generated names.
    pub fn with_storage_name_list(mut self, storage_name_list: HashMap<u32, String>) -> Self {
        self.partition = storage_name_list.len() as u32;
        self.storage_name_list = storage_name_list;
        self
    }

    pub fn create_partition_name(topic_id: &str, partition: u32) -> HashMap<u32, String> {
        let mut results = HashMap::new();
        for i in 0..partition {
            results.insert(i, Topic::build_storage_name(topic_id, i));
        }
        results
    }

    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }

    pub fn build_storage_name(topic_id: &str, partition: u32) -> String {
        format!("{}-{}", topic_id, partition)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct TopicConfig {
    /// Max size per segment in bytes. Default: 1 GiB.
    pub max_segment_size: u64,
    /// Retention duration in seconds. Default: 24 hours.
    pub retention_sec: u64,
}

impl Default for TopicConfig {
    fn default() -> Self {
        TopicConfig {
            max_segment_size: DEFAULT_MAX_SEGMENT_SIZE,
            retention_sec: DEFAULT_RETENTION_SEC,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let topic = Topic {
            topic_id: "test-id".to_string(),
            tenant: "test-tenant".to_string(),
            topic_name: "test-topic".to_string(),
            storage_type: StorageType::EngineMemory,
            source: TopicSource::SystemInner,
            ttl: 0,
            partition: 3,
            replication: 2,
            storage_name_list: HashMap::new(),
            config: TopicConfig::default(),
            create_time: 1234567890,
        };

        let encoded = topic.encode().unwrap();
        let decoded = Topic::decode(&encoded).unwrap();

        assert_eq!(topic, decoded);
    }

    #[test]
    fn test_with_builders() {
        let storage_name_list =
            HashMap::from([(0, "shard-0".to_string()), (1, "shard-1".to_string())]);
        let topic = Topic::new("tenant", "my-topic", StorageType::EngineMemory)
            .with_ttl(3600)
            .with_partition(2)
            .with_replication(3)
            .with_storage_name_list(storage_name_list.clone());

        assert_eq!(topic.ttl, 3600);
        assert_eq!(topic.partition, 2);
        assert_eq!(topic.replication, 3);
        assert_eq!(topic.storage_name_list, storage_name_list);
    }
}
