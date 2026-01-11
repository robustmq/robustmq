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

use crate::{commitlog::memory::engine::MemoryStorageEngine, core::error::StorageEngineError};
use dashmap::DashMap;
use metadata_struct::storage::{
    adapter_read_config::AdapterWriteRespRow, adapter_record::AdapterWriteRecord,
    convert::convert_adapter_record_to_engine,
};
use std::sync::Arc;

impl MemoryStorageEngine {
    pub async fn batch_write(
        &self,
        shard: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        self.internal_batch_write(shard, messages).await
    }

    pub async fn write(
        &self,
        shard: &str,
        data: &AdapterWriteRecord,
    ) -> Result<AdapterWriteRespRow, StorageEngineError> {
        let results = self
            .internal_batch_write(shard, std::slice::from_ref(data))
            .await?;

        if results.is_empty() {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "Write to shard [{}] returned empty result",
                shard
            )));
        }

        Ok(results.first().unwrap().clone())
    }

    pub async fn delete_by_key(&self, shard: &str, key: &str) -> Result<(), StorageEngineError> {
        let offset = if let Some(key_map) = self.key_index.get(shard) {
            if let Some(offset) = key_map.get(key) {
                *offset
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        self.delete_by_offset(shard, offset).await
    }

    pub async fn delete_by_offset(
        &self,
        shard: &str,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        let record = if let Some(data_map) = self.shard_data.get(shard) {
            if let Some((_, record)) = data_map.remove(&offset) {
                record
            } else {
                return Ok(());
            }
        } else {
            return Ok(());
        };

        if let Some(key) = &record.metadata.key {
            if let Some(key_map) = self.key_index.get_mut(shard) {
                key_map.remove(key);
            }
        }

        if let Some(tags) = &record.metadata.tags {
            if let Some(tag_map) = self.tag_index.get_mut(shard) {
                for tag in tags.iter() {
                    if let Some(mut offsets) = tag_map.get_mut(tag) {
                        offsets.retain(|&o| o != offset);
                    }
                }
            }
        }

        if record.metadata.create_t > 0 && offset.is_multiple_of(5000) {
            if let Some(timestamp_map) = self.timestamp_index.get_mut(shard) {
                timestamp_map.remove(&record.metadata.create_t);
            }
        }

        Ok(())
    }

    async fn internal_batch_write(
        &self,
        shard_name: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        // lock
        let shard_name_str = shard_name.to_string();
        let lock = self
            .shard_write_locks
            .entry(shard_name_str.clone())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone();

        let _guard = lock.lock().await;

        // init
        let capacity = self.config.max_records_per_shard.min(1024);
        let current_shard_data_list = self
            .shard_data
            .entry(shard_name_str.clone())
            .or_insert_with(|| DashMap::with_capacity(capacity));

        self.try_remove_old_data(shard_name, &current_shard_data_list, messages.len())?;

        // save data
        let mut offset_res = Vec::with_capacity(messages.len());
        let mut offset = self.commit_log_offset.get_latest_offset(shard_name)?;
        for msg in messages.iter() {
            offset_res.push(AdapterWriteRespRow {
                pkid: msg.pkid,
                offset,
                ..Default::default()
            });

            let engine_record = convert_adapter_record_to_engine(msg.clone(), shard_name, offset);

            current_shard_data_list.insert(offset, engine_record);

            self.save_index(shard_name, offset, msg);

            offset += 1;
        }

        self.commit_log_offset
            .save_latest_offset(shard_name, offset)?;
        Ok(offset_res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        commitlog::offset::CommitLogOffset,
        core::{cache::StorageCacheManager, test_tool::test_build_memory_engine},
    };
    use broker_core::cache::BrokerCacheManager;
    use bytes::Bytes;
    use common_base::tools::unique_id;
    use common_config::config::BrokerConfig;
    use metadata_struct::storage::adapter_read_config::AdapterReadConfig;

    #[tokio::test]
    async fn test_try_remove_old_data() {
        let mut engine = test_build_memory_engine();
        engine.config.max_records_per_shard = 10;
        let shard_name = unique_id();
        let broker_cache = Arc::new(BrokerCacheManager::new(BrokerConfig::default()));
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        let commit_offset = CommitLogOffset::new(
            cache_manager.clone(),
            engine.commit_log_offset.rocksdb_engine_handler.clone(),
        );

        commit_offset.save_earliest_offset(&shard_name, 0).unwrap();
        commit_offset.save_latest_offset(&shard_name, 0).unwrap();

        let messages: Vec<AdapterWriteRecord> = (0..10)
            .map(|i| AdapterWriteRecord {
                pkid: i,
                key: Some(format!("key{}", i)),
                tags: Some(vec![format!("tag{}", i % 3)]),
                timestamp: 1000 + i * 100,
                ..Default::default()
            })
            .collect();

        engine.batch_write(&shard_name, &messages).await.unwrap();
        let new_message = AdapterWriteRecord {
            pkid: 100,
            key: Some("key100".to_string()),
            tags: Some(vec!["tag100".to_string()]),
            timestamp: 2000,
            ..Default::default()
        };
        engine.write(&shard_name, &new_message).await.unwrap();
        let data = engine.shard_data.get(&shard_name).unwrap();
        assert_eq!(data.len(), 9);
        assert!(!data.contains_key(&0));
        assert!(!data.contains_key(&1));
        assert!(data.contains_key(&2));
        assert!(data.contains_key(&10));
        let earliest = engine
            .commit_log_offset
            .get_earliest_offset(&shard_name)
            .unwrap();
        assert_eq!(earliest, 2);
        let key_map = engine.key_index.get(&shard_name).unwrap();
        assert!(!key_map.contains_key("key0"));
        assert!(!key_map.contains_key("key1"));
        assert!(key_map.contains_key("key2"));
    }

    #[tokio::test]
    async fn test_write_and_delete() {
        let engine = test_build_memory_engine();
        let shard_name = unique_id();
        let broker_cache = Arc::new(BrokerCacheManager::new(BrokerConfig::default()));
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        let commit_offset = CommitLogOffset::new(
            cache_manager.clone(),
            engine.commit_log_offset.rocksdb_engine_handler.clone(),
        );

        commit_offset.save_earliest_offset(&shard_name, 0).unwrap();
        commit_offset.save_latest_offset(&shard_name, 0).unwrap();

        let messages: Vec<AdapterWriteRecord> = (0..5)
            .map(|i| AdapterWriteRecord {
                pkid: i,
                key: Some(format!("key{}", i)),
                tags: Some(vec![format!("tag{}", i)]),
                data: Bytes::from(format!("data{}", i)),
                timestamp: 1000 + i * 100,
                ..Default::default()
            })
            .collect();

        engine.batch_write(&shard_name, &messages).await.unwrap();

        engine.delete_by_key(&shard_name, "key2").await.unwrap();

        let read_config = AdapterReadConfig {
            max_record_num: 10,
            max_size: 1024 * 1024,
        };

        let key_records = engine.read_by_key(&shard_name, "key2").await.unwrap();
        assert_eq!(key_records.len(), 0);

        let tag_records = engine
            .read_by_tag(&shard_name, "tag2", None, &read_config)
            .await
            .unwrap();
        assert_eq!(tag_records.len(), 0);
    }
}
