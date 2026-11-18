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
        adapter_offset::{
            AdapterCommitOffset, AdapterConsumerGroupOffset, AdapterOffsetStrategy,
            AdapterShardInfo,
        },
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
        for shard_name in topic.storage_name_list.values() {
            driver
                .create_shard(&AdapterShardInfo {
                    shard_name: shard_name.to_string(),
                    topic_name: topic_name.to_string(),
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
        acks: i8,
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
        driver.write(&partition_name, data, acks).await
    }

    /// Write to a specific partition (Kafka Produce: the client selects the
    /// partition explicitly, unlike the round-robin `write` above).
    pub async fn write_to_partition(
        &self,
        tenant: &str,
        topic_name: &str,
        partition: u32,
        data: &[AdapterWriteRecord],
        acks: i8,
    ) -> Result<Vec<AdapterWriteRespRow>, CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        let partition_name = topic
            .storage_name_list
            .get(&partition)
            .cloned()
            .ok_or_else(|| {
                CommonError::CommonError(format!(
                    "partition {} does not exist for topic {}",
                    partition, topic_name
                ))
            })?;
        driver.write(&partition_name, data, acks).await
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
        keys: &[&[u8]],
    ) -> Result<HashMap<Vec<u8>, Vec<StorageRecord>>, CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        let mut results: HashMap<Vec<u8>, Vec<StorageRecord>> = HashMap::new();
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
        keys: &[&[u8]],
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

    /// Per-partition Kafka DeleteRecords: delete all records with offset <
    /// the requested target on each named partition. Returns the achieved
    /// low_watermark per partition.
    pub async fn delete_records_before(
        &self,
        tenant: &str,
        topic_name: &str,
        targets: &HashMap<u32, u64>,
    ) -> Result<HashMap<u32, u64>, CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        let mut results = HashMap::with_capacity(targets.len());
        for (partition, target_offset) in targets {
            let Some(shard_name) = topic.storage_name_list.get(partition) else {
                continue;
            };
            let achieved = driver
                .delete_records_before(shard_name, *target_offset)
                .await?;
            results.insert(*partition, achieved);
        }
        Ok(results)
    }

    pub async fn get_offset_by_timestamp(
        &self,
        tenant: &str,
        topic_name: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<HashMap<u32, u64>, CommonError> {
        let (topic, driver) = self.build_driver(tenant, topic_name).await?;
        let mut results = HashMap::with_capacity(topic.storage_name_list.len());
        for (partition, shard_name) in topic.storage_name_list {
            let offset = driver
                .get_offset_by_timestamp(&shard_name, timestamp, strategy.clone())
                .await?;
            results.insert(partition, offset);
        }

        Ok(results)
    }

    pub async fn get_offset_by_group(
        &self,
        tenant: &str,
        group_name: &str,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, CommonError> {
        self.offset_manager.get_offset(tenant, group_name).await
    }

    pub async fn commit_group_offset(
        &self,
        tenant: &str,
        group_name: &str,
        offsets: &[AdapterCommitOffset],
    ) -> Result<(), CommonError> {
        self.offset_manager
            .commit_group_offset(tenant, group_name, offsets)
            .await
    }

    /// Delete a group's committed offsets for the given shards (Kafka
    /// OffsetDelete semantics: removes the checkpoint, not the topic data —
    /// see `delete_by_offsets`/`delete_records_before` for that).
    pub async fn delete_group_offset(
        &self,
        tenant: &str,
        group_name: &str,
        shard_names: &[String],
    ) -> Result<(), CommonError> {
        self.offset_manager
            .delete_group_offset(tenant, group_name, shard_names)
            .await
    }

    /// Resolve a topic and its storage driver once, for callers that need to
    /// issue several per-partition operations against the same topic without
    /// re-resolving (and re-cloning `Topic`) on every call.
    pub async fn resolve_topic_driver(
        &self,
        tenant: &str,
        topic_name: &str,
    ) -> Result<(Topic, ArcStorageAdapter), CommonError> {
        self.build_driver(tenant, topic_name).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{test_add_topic, test_build_storage_driver_manager};
    use common_base::uuid::unique_id;
    use metadata_struct::{
        adapter::adapter_read_config::AdapterReadConfig,
        storage::{
            segment::EngineSegment,
            segment_meta::EngineSegmentMetadata,
            shard::{EngineShard, EngineShardConfig},
        },
        tenant::DEFAULT_TENANT,
        topic::Topic,
    };
    use storage_engine::core::offset::{ShardOffset, ShardOffsetState};

    fn add_partitioned_topic(
        manager: &Arc<StorageDriverManager>,
        topic_name: &str,
        partitions: u32,
    ) {
        let topic = Topic::new(DEFAULT_TENANT, topic_name, StorageType::EngineMemory)
            .with_partition(partitions);
        manager.broker_cache.add_topic(&topic);

        let cache = &manager.engine_storage_handler.cache_manager;
        for shard_name in topic.storage_name_list.values() {
            cache.set_shard(EngineShard {
                shard_uid: unique_id(),
                shard_name: shard_name.clone(),
                topic_name: topic_name.to_string(),
                config: EngineShardConfig {
                    storage_type: StorageType::EngineMemory,
                    ..Default::default()
                },
                ..Default::default()
            });
            cache.set_segment(&EngineSegment {
                shard_name: shard_name.clone(),
                segment_seq: 0,
                leader: 1,
                ..Default::default()
            });
            cache.set_segment_meta(EngineSegmentMetadata {
                shard_name: shard_name.clone(),
                segment_seq: 0,
                ..Default::default()
            });
            cache.save_offset_state(shard_name.clone(), ShardOffsetState::default());
            let offsets = ShardOffset::new(
                cache.clone(),
                manager
                    .engine_storage_handler
                    .rocksdb_engine_handler
                    .clone(),
            );
            offsets.save_earliest_offset(shard_name, 0).unwrap();
            offsets.save_latest_offset(shard_name, 0).unwrap();
            offsets.save_high_watermark_offset(shard_name, 0).unwrap();
        }
    }

    fn read_config() -> AdapterReadConfig {
        AdapterReadConfig {
            max_record_num: 100,
            max_size: 1024 * 1024,
        }
    }

    #[tokio::test]
    async fn missing_topic_is_reported_before_driver_lookup() {
        let manager = test_build_storage_driver_manager().await.unwrap();

        let error = match manager
            .resolve_topic_driver(DEFAULT_TENANT, &unique_id())
            .await
        {
            Ok(_) => panic!("missing topic unexpectedly resolved"),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            CommonError::TopicNotFoundInBrokerCache(_, _)
        ));
        assert!(manager.driver_list.is_empty());
    }

    #[tokio::test]
    async fn supported_driver_is_cached_and_round_robin_writes_are_readable() {
        let manager = test_build_storage_driver_manager().await.unwrap();
        let topic_name = unique_id();
        add_partitioned_topic(&manager, &topic_name, 2);

        let (_, first_driver) = manager
            .resolve_topic_driver(DEFAULT_TENANT, &topic_name)
            .await
            .unwrap();
        let (_, second_driver) = manager
            .resolve_topic_driver(DEFAULT_TENANT, &topic_name)
            .await
            .unwrap();
        assert!(Arc::ptr_eq(&first_driver, &second_driver));
        assert_eq!(manager.driver_list.len(), 1);

        let first = manager
            .write(
                DEFAULT_TENANT,
                &topic_name,
                &[AdapterWriteRecord::new(&topic_name, b"first".as_ref())],
                1,
            )
            .await
            .unwrap();
        let second = manager
            .write(
                DEFAULT_TENANT,
                &topic_name,
                &[AdapterWriteRecord::new(&topic_name, b"second".as_ref())],
                1,
            )
            .await
            .unwrap();
        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
        assert_eq!(first[0].offset, 0);
        assert_eq!(second[0].offset, 0);

        let records = manager
            .read_by_offset(DEFAULT_TENANT, &topic_name, &HashMap::new(), &read_config())
            .await
            .unwrap();
        assert_eq!(records.len(), 2);
        let partition_names = manager
            .broker_cache
            .get_topic_by_name(DEFAULT_TENANT, &topic_name)
            .unwrap()
            .storage_name_list;
        assert_eq!(
            records
                .iter()
                .map(|record| record.data.as_ref())
                .collect::<std::collections::HashSet<_>>(),
            [b"first".as_ref(), b"second".as_ref()]
                .into_iter()
                .collect::<std::collections::HashSet<_>>()
        );
        assert_eq!(
            records
                .iter()
                .map(|record| record.metadata.shard.clone())
                .collect::<std::collections::HashSet<_>>(),
            partition_names
                .into_values()
                .collect::<std::collections::HashSet<_>>()
        );

        let topic = manager
            .broker_cache
            .get_topic_by_name(DEFAULT_TENANT, &topic_name)
            .unwrap();
        let offsets = HashMap::from([(topic.storage_name_list[&0].clone(), 1)]);
        let from_second_offset = manager
            .read_by_offset(DEFAULT_TENANT, &topic_name, &offsets, &read_config())
            .await
            .unwrap();
        assert_eq!(from_second_offset.len(), 1);
        assert_eq!(from_second_offset[0].data.as_ref(), b"second");

        let explicit = manager
            .write_to_partition(
                DEFAULT_TENANT,
                &topic_name,
                1,
                &[AdapterWriteRecord::new(&topic_name, b"explicit".as_ref())],
                1,
            )
            .await
            .unwrap();
        assert_eq!(explicit[0].offset, 1);

        let timestamp_offsets = manager
            .get_offset_by_timestamp(
                DEFAULT_TENANT,
                &topic_name,
                0,
                AdapterOffsetStrategy::Earliest,
            )
            .await
            .unwrap();
        assert_eq!(timestamp_offsets.len(), 2);
        assert_eq!(
            timestamp_offsets.values().copied().collect::<Vec<_>>(),
            [0, 0]
        );
    }

    #[tokio::test]
    async fn partition_write_rejects_unknown_partition() {
        let manager = test_build_storage_driver_manager().await.unwrap();
        let topic_name = unique_id();
        test_add_topic(&manager, &topic_name);

        let error = manager
            .write_to_partition(
                DEFAULT_TENANT,
                &topic_name,
                1,
                &[AdapterWriteRecord::new(&topic_name, b"payload".as_ref())],
                1,
            )
            .await
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("partition 1 does not exist for topic {topic_name}")
        );
    }

    #[tokio::test]
    async fn list_storage_resource_returns_partition_details_from_local_cache() {
        let manager = test_build_storage_driver_manager().await.unwrap();
        let topic_name = unique_id();
        test_add_topic(&manager, &topic_name);

        let resources = manager
            .list_storage_resource(DEFAULT_TENANT, &topic_name)
            .await
            .unwrap();

        assert_eq!(resources.len(), 1);
        let resource = resources.get(&0).unwrap();
        assert_eq!(
            resource.shard_name,
            Topic::build_storage_name(
                &manager
                    .broker_cache
                    .get_topic_by_name(DEFAULT_TENANT, &topic_name)
                    .unwrap()
                    .topic_id,
                0,
            )
        );
        assert_eq!(resource.offset.start_offset, 0);
        assert_eq!(resource.offset.high_watermark, 0);
    }

    #[tokio::test]
    async fn reads_by_tag_and_key_and_deletes_update_indexes() {
        let manager = test_build_storage_driver_manager().await.unwrap();
        let topic_name = unique_id();
        test_add_topic(&manager, &topic_name);

        let records = vec![
            AdapterWriteRecord::new(&topic_name, b"one".as_ref())
                .with_key("key-one")
                .with_tags(vec!["blue".to_string()]),
            AdapterWriteRecord::new(&topic_name, b"two".as_ref())
                .with_key("key-two")
                .with_tags(vec!["blue".to_string(), "green".to_string()]),
            AdapterWriteRecord::new(&topic_name, b"three".as_ref())
                .with_key("key-three")
                .with_tags(vec!["green".to_string()]),
        ];
        let offsets = manager
            .write(DEFAULT_TENANT, &topic_name, &records, 1)
            .await
            .unwrap();
        assert_eq!(
            offsets.iter().map(|row| row.offset).collect::<Vec<_>>(),
            [0, 1, 2]
        );

        let by_tag = manager
            .read_by_tag(
                DEFAULT_TENANT,
                &topic_name,
                "green",
                &HashMap::new(),
                &read_config(),
            )
            .await
            .unwrap();
        assert_eq!(by_tag.len(), 2);

        let by_key = manager
            .read_by_keys(
                DEFAULT_TENANT,
                &topic_name,
                &[b"key-two".as_slice(), b"missing".as_slice()],
            )
            .await
            .unwrap();
        assert_eq!(by_key.get(b"key-two".as_slice()).unwrap().len(), 1);
        assert!(by_key.get(b"missing".as_slice()).unwrap().is_empty());

        manager
            .delete_by_keys(DEFAULT_TENANT, &topic_name, &[b"key-two".as_slice()])
            .await
            .unwrap();
        let by_key = manager
            .read_by_keys(DEFAULT_TENANT, &topic_name, &[b"key-two".as_slice()])
            .await
            .unwrap();
        assert!(by_key.get(b"key-two".as_slice()).unwrap().is_empty());

        manager
            .delete_by_offsets(DEFAULT_TENANT, &topic_name, &[0])
            .await
            .unwrap();
        let remaining = manager
            .read_by_offset(DEFAULT_TENANT, &topic_name, &HashMap::new(), &read_config())
            .await
            .unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].data.as_ref(), b"three");
    }

    #[tokio::test]
    async fn delete_records_before_returns_per_partition_watermark() {
        let manager = test_build_storage_driver_manager().await.unwrap();
        let topic_name = unique_id();
        test_add_topic(&manager, &topic_name);
        manager
            .write(
                DEFAULT_TENANT,
                &topic_name,
                &[
                    AdapterWriteRecord::new(&topic_name, b"zero".as_ref()),
                    AdapterWriteRecord::new(&topic_name, b"one".as_ref()),
                    AdapterWriteRecord::new(&topic_name, b"two".as_ref()),
                ],
                1,
            )
            .await
            .unwrap();

        let targets = HashMap::from([(0, 2), (99, 100)]);
        let result = manager
            .delete_records_before(DEFAULT_TENANT, &topic_name, &targets)
            .await
            .unwrap();
        assert_eq!(result, HashMap::from([(0, 2)]));

        let records = manager
            .read_by_offset(DEFAULT_TENANT, &topic_name, &HashMap::new(), &read_config())
            .await
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].data.as_ref(), b"two");
    }

    #[tokio::test]
    async fn committed_group_offsets_are_available_from_local_cache() {
        let manager = test_build_storage_driver_manager().await.unwrap();
        let shard_name = unique_id();
        let offsets = [AdapterCommitOffset {
            shard_name: shard_name.clone(),
            topic_name: "topic".to_string(),
            partition: 0,
            offset: 42,
        }];

        manager
            .commit_group_offset(DEFAULT_TENANT, "group", &offsets)
            .await
            .unwrap();
        let result = manager
            .get_offset_by_group(DEFAULT_TENANT, "group")
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].shard_name, shard_name);
        assert_eq!(result[0].offset, 42);
    }

    #[tokio::test]
    async fn unsupported_storage_type_is_rejected() {
        let manager = test_build_storage_driver_manager().await.unwrap();
        let topic_name = unique_id();
        manager.broker_cache.add_topic(&Topic::new(
            DEFAULT_TENANT,
            &topic_name,
            StorageType::Mysql,
        ));

        let error = match manager
            .resolve_topic_driver(DEFAULT_TENANT, &topic_name)
            .await
        {
            Ok(_) => panic!("unsupported storage type unexpectedly resolved"),
            Err(error) => error,
        };
        assert!(error
            .to_string()
            .contains("Unsupported storage type 'Mysql'"));
    }
}
