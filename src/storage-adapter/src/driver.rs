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

use crate::{engine::EngineStorageAdapter, storage::StorageAdapter};
use broker_core::cache::NodeCacheManager;
use common_base::error::common::CommonError;
use common_config::storage::StorageType;
use common_group::manager::OffsetManager;
use dashmap::DashMap;
use metadata_struct::{
    adapter::adapter_shard::AdapterShardDetail,
    mqtt::topic::Topic,
    storage::{
        adapter_offset::{AdapterConsumerGroupOffset, AdapterOffsetStrategy, AdapterShardInfo},
        adapter_read_config::{AdapterReadConfig, AdapterWriteRespRow},
        adapter_record::AdapterWriteRecord,
        record::StorageRecord,
        shard::EngineShardConfig,
    },
};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc},
};
use storage_engine::handler::adapter::StorageEngineHandler;

pub type ArcStorageAdapter = Arc<dyn StorageAdapter + Send + Sync>;

#[derive(Clone)]
pub struct StorageDriverManager {
    pub driver_list: DashMap<String, ArcStorageAdapter>,
    pub engine_storage_handler: Arc<StorageEngineHandler>,
    pub broker_cache: Arc<NodeCacheManager>,
    pub offset_manager: Arc<OffsetManager>,
    pub message_seq: Arc<AtomicU64>,
}

impl StorageDriverManager {
    pub async fn new(
        offset_manager: Arc<OffsetManager>,
        engine_storage_handler: Arc<StorageEngineHandler>,
    ) -> Result<Self, CommonError> {
        Ok(StorageDriverManager {
            driver_list: DashMap::with_capacity(2),
            engine_storage_handler: engine_storage_handler.clone(),
            broker_cache: engine_storage_handler.cache_manager.broker_cache.clone(),
            offset_manager,
            message_seq: Arc::new(AtomicU64::new(0)),
        })
    }

    pub async fn create_storage_resource(
        &self,
        tenant: &str,
        topic_name: &str,
        config: &EngineShardConfig,
    ) -> Result<(), CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        for (_, shard_name) in topic.storage_name_list.iter() {
            driver
                .create_shard(&AdapterShardInfo {
                    shard_name: shard_name.to_string(),
                    config: config.clone(),
                    desc: format!("topic name:{}", topic_name),
                })
                .await?;
        }
        Ok(())
    }

    pub async fn list_storage_resource(
        &self,
        tenant: &str,
        topic_name: &str,
    ) -> Result<HashMap<u32, AdapterShardDetail>, CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        let mut results = HashMap::new();
        for (partition, shard_name) in topic.storage_name_list {
            let resp = driver.list_shard(Some(shard_name)).await?;
            for raw in resp {
                results.insert(partition, raw);
            }
        }
        Ok(results)
    }

    pub async fn delete_storage_resource(
        &self,
        tenant: &str,
        topic_name: &str,
    ) -> Result<(), CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        for (_, shard_name) in topic.storage_name_list {
            driver.delete_shard(&shard_name).await?;
        }
        Ok(())
    }

    pub async fn write(
        &self,
        tenant: &str,
        topic_name: &str,
        data: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;

        // Pick a partition via round-robin and use the shard name from storage_name_list.
        let partition_count = topic.partition as u64;
        let partition = self
            .message_seq
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % partition_count;
        let partition_name = topic
            .storage_name_list
            .get(&(partition as u32))
            .cloned()
            .unwrap_or_else(|| Topic::build_storage_name(&topic.topic_id, partition as u32));
        driver.write(&partition_name, data).await
    }

    pub async fn read_by_offset(
        &self,
        tenant: &str,
        topic_name: &str,
        offsets: &HashMap<String, u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        let mut results = Vec::new();
        for (_, shard_name) in topic.storage_name_list {
            let offset = if let Some(offset) = offsets.get(&shard_name) {
                *offset
            } else {
                0
            };
            let resp = driver
                .read_by_offset(&shard_name, offset, read_config)
                .await?;
            results.extend(resp);
        }
        Ok(results)
    }

    pub async fn read_by_tag(
        &self,
        tenant: &str,
        topic_name: &str,
        tag: &str,
        offsets: &HashMap<String, u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        let mut results = Vec::new();
        for (_, shard_name) in topic.storage_name_list {
            let offset = offsets.get(&shard_name).copied();
            let resp = driver
                .read_by_tag(&shard_name, tag, offset, read_config)
                .await?;
            results.extend(resp);
        }
        Ok(results)
    }

    pub async fn read_by_keys(
        &self,
        tenant: &str,
        topic_name: &str,
        keys: &[&str],
    ) -> Result<HashMap<String, Vec<StorageRecord>>, CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        let mut results: HashMap<String, Vec<StorageRecord>> = HashMap::new();
        for (_, shard_name) in topic.storage_name_list {
            let shard_result = driver.read_by_keys(&shard_name, keys).await?;
            for (key, records) in shard_result {
                results.entry(key).or_default().extend(records);
            }
        }
        Ok(results)
    }

    pub async fn delete_by_keys(
        &self,
        tenant: &str,
        topic_name: &str,
        keys: &[&str],
    ) -> Result<(), CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        for (_, shard_name) in topic.storage_name_list {
            driver.delete_by_keys(&shard_name, keys).await?
        }
        Ok(())
    }

    pub async fn delete_by_offsets(
        &self,
        tenant: &str,
        topic_name: &str,
        offsets: &[u64],
    ) -> Result<(), CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        for (_, shard_name) in topic.storage_name_list {
            driver.delete_by_offsets(&shard_name, offsets).await?
        }
        Ok(())
    }

    pub async fn get_offset_by_timestamp(
        &self,
        tenant: &str,
        topic_name: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        let mut results = Vec::new();
        for (_, shard_name) in topic.storage_name_list {
            let offset = driver
                .get_offset_by_timestamp(&shard_name, timestamp, strategy.clone())
                .await?;
            results.push(offset);
        }

        Ok(results.iter().min().copied().unwrap_or(0))
    }

    pub async fn get_offset_by_group(
        &self,
        tenant: &str,
        group_name: &str,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, CommonError> {
        self.offset_manager.get_offset(tenant, group_name).await
    }

    pub async fn commit_offset(
        &self,
        tenant: &str,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        self.offset_manager
            .commit_offset(tenant, group_name, offset)
            .await
    }

    async fn build_driver(
        &self,
        tenant: &str,
        topic_name: &str,
    ) -> Result<(Topic, ArcStorageAdapter), CommonError> {
        let topic = if let Some(topic) = self.broker_cache.get_topic_by_name(tenant, topic_name) {
            topic
        } else {
            return Err(CommonError::TopicNotFoundInBrokerCache(
                tenant.to_string(),
                topic_name.to_string(),
            ));
        };

        let driver = self.get_storage_driver_by_topic(&topic).await?;
        Ok((topic, driver))
    }

    async fn get_storage_driver_by_topic(
        &self,
        topic: &Topic,
    ) -> Result<ArcStorageAdapter, CommonError> {
        let storage_type_str = format!("{:?}", topic.storage_type);
        if let Some(driver) = self.driver_list.get(&storage_type_str) {
            return Ok(driver.clone());
        }

        let driver = match topic.storage_type {
            StorageType::EngineMemory | StorageType::EngineRocksDB | StorageType::EngineSegment => {
                Arc::new(EngineStorageAdapter::new(self.engine_storage_handler.clone()).await)
            }
            _ => {
                return Err(CommonError::CommonError(format!(
                    "Unsupported storage type '{:?}' for topic '{}'",
                    topic.storage_type, topic.topic_name
                )));
            }
        };
        self.driver_list.insert(storage_type_str, driver.clone());
        Ok(driver)
    }
}
