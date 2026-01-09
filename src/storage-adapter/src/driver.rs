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
use broker_core::cache::BrokerCacheManager;
use common_base::error::common::CommonError;
use common_config::storage::StorageType;
use dashmap::DashMap;
use metadata_struct::{
    mqtt::topic::Topic,
    storage::{
        adapter_offset::{AdapterOffsetStrategy, AdapterShardInfo},
        adapter_read_config::{AdapterReadConfig, AdapterWriteRespRow},
        adapter_record::AdapterWriteRecord,
        shard::EngineShardConfig,
        storage_record::StorageRecord,
    },
};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU64, Arc},
};
use storage_engine::{group::OffsetManager, handler::adapter::StorageEngineHandler};

pub type ArcStorageAdapter = Arc<dyn StorageAdapter + Send + Sync>;
#[derive(Clone)]
pub struct StorageDriverManager {
    pub driver_list: DashMap<String, ArcStorageAdapter>,
    pub engine_storage_handler: Arc<StorageEngineHandler>,
    pub broker_cache: Arc<BrokerCacheManager>,
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
        topic_name: &str,
        config: &EngineShardConfig,
    ) -> Result<(), CommonError> {
        let (topic, driver) = self.build_driver(topic_name).await?;
        for partition in topic.storage_name_list {
            driver
                .create_shard(&AdapterShardInfo {
                    shard_name: partition,
                    config: config.clone(),
                })
                .await?;
        }
        Ok(())
    }

    pub async fn list_storage_resource(
        &self,
        topic_name: &str,
    ) -> Result<Vec<String>, CommonError> {
        let (topic, driver) = self.build_driver(topic_name).await?;
        let mut results = Vec::new();
        for partition in topic.storage_name_list {
            let resp = driver.list_shard(Some(partition)).await?;
            for raw in resp {
                results.push(serde_json::to_string(&raw)?);
            }
        }
        Ok(results)
    }

    pub async fn delete_storage_resource(&self, topic_name: &str) -> Result<(), CommonError> {
        let (topic, driver) = self.build_driver(topic_name).await?;
        for partition in topic.storage_name_list {
            driver.delete_shard(&partition).await?;
        }
        Ok(())
    }

    pub async fn write(
        &self,
        topic_name: &str,
        data: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, CommonError> {
        let (topic, driver) = self.build_driver(topic_name).await?;

        // todo write-up strategy needs to be further improved.
        let partition = self
            .message_seq
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            % topic.partition as u64;
        let partition_name = Topic::build_storage_name(&topic.topic_id, partition as u32);
        driver.batch_write(&partition_name, data).await
    }

    pub async fn read_by_offset(
        &self,
        topic_name: &str,
        offsets: &HashMap<String, u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        let (topic, driver) = self.build_driver(topic_name).await?;
        let mut results = Vec::new();
        for partition in topic.storage_name_list {
            let offset = if let Some(offset) = offsets.get(&partition) {
                *offset
            } else {
                0
            };
            let resp = driver
                .read_by_offset(&partition, offset, read_config)
                .await?;
            results.extend(resp);
        }
        Ok(results)
    }

    pub async fn read_by_tag(
        &self,
        topic_name: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        let (topic, driver) = self.build_driver(topic_name).await?;
        let mut results = Vec::new();
        for partition in topic.storage_name_list {
            let resp = driver
                .read_by_tag(&partition, tag, start_offset, read_config)
                .await?;
            results.extend(resp);
        }
        Ok(results)
    }

    pub async fn read_by_key(
        &self,
        topic_name: &str,
        key: &str,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        let (topic, driver) = self.build_driver(topic_name).await?;
        let mut results = Vec::new();
        for partition in topic.storage_name_list {
            let resp = driver.read_by_key(&partition, key).await?;
            results.extend(resp);
        }
        Ok(results)
    }

    pub async fn delete_by_key(&self, topic_name: &str, key: &str) -> Result<(), CommonError> {
        let (topic, driver) = self.build_driver(topic_name).await?;
        for partition in topic.storage_name_list {
            driver.delete_by_key(&partition, key).await?
        }
        Ok(())
    }

    pub async fn delete_by_offset(&self, topic_name: &str, offset: u64) -> Result<(), CommonError> {
        let (topic, driver) = self.build_driver(topic_name).await?;
        for partition in topic.storage_name_list {
            driver.delete_by_offset(&partition, offset).await?
        }
        Ok(())
    }

    pub async fn get_offset_by_timestamp(
        &self,
        topic_name: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, CommonError> {
        let (topic, driver) = self.build_driver(topic_name).await?;
        let mut results = Vec::new();
        for partition in topic.storage_name_list {
            let offset = driver
                .get_offset_by_timestamp(&partition, timestamp, strategy.clone())
                .await?;
            results.push(offset);
        }

        Ok(results.iter().min().copied().unwrap_or(0))
    }

    async fn build_driver(
        &self,
        topic_name: &str,
    ) -> Result<(Topic, ArcStorageAdapter), CommonError> {
        let topic = if let Some(topic) = self.broker_cache.get_topic_by_name(topic_name) {
            topic
        } else {
            return Err(CommonError::CommonError(format!(
                "Topic '{}' not found in broker cache",
                topic_name
            )));
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
