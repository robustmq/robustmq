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

use crate::{
    commitlog::rocksdb::engine::{IndexInfo, RocksDBStorageEngine},
    core::{error::StorageEngineError, message_ttl::is_record_expired},
};
use common_base::utils::serialize::deserialize;
use metadata_struct::storage::{
    adapter_offset::AdapterOffsetStrategy, adapter_read_config::AdapterReadConfig,
    record::StorageRecord,
};
use rocksdb_engine::keys::engine::{
    key_index_key, record_key, record_prefix, tag_index_key, tag_index_tag_prefix,
    timestamp_index_prefix,
};

impl RocksDBStorageEngine {
    pub async fn read_by_offset(
        &self,
        shard: &str,
        start_offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let end_offset = self.commitlog_offset.get_latest_offset(shard)?;
        let cf = self.get_cf()?;

        let mut records = Vec::new();
        let mut total_size = 0u64;
        let mut cursor = start_offset;

        'outer: while cursor < end_offset {
            let batch_end = cursor.saturating_add(100).min(end_offset);
            let keys: Vec<String> = (cursor..batch_end)
                .map(|i| record_key(shard, 0, i))
                .collect();

            let batch_results = self
                .rocksdb_engine_handler
                .multi_get::<StorageRecord>(cf.clone(), &keys)?;

            for record_opt in batch_results {
                let Some(record) = record_opt else {
                    continue;
                };

                if is_record_expired(&record.metadata) {
                    continue;
                }

                if records.len() >= read_config.max_record_num as usize {
                    break 'outer;
                }
                let record_bytes = record.data.len() as u64;
                if !records.is_empty() && total_size + record_bytes > read_config.max_size {
                    break 'outer;
                }
                total_size += record_bytes;
                records.push(record);
            }

            cursor = batch_end;
        }

        Ok(records)
    }

    pub async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let cf = self.get_cf()?;
        let tag_prefix = tag_index_tag_prefix(shard, tag);
        // tag keys sort by offset, so seek straight to start_offset.
        let seek_key = match start_offset {
            Some(so) => tag_index_key(shard, tag, so),
            None => tag_prefix.clone(),
        };

        let mut offsets = Vec::new();
        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(seek_key.as_bytes());
        while iter.valid() {
            let Some(key_bytes) = iter.key() else {
                break;
            };
            if !key_bytes.starts_with(tag_prefix.as_bytes()) {
                break;
            }
            let Some(value) = iter.value() else {
                break;
            };
            offsets.push(deserialize::<IndexInfo>(value)?.offset);
            iter.next();
        }

        if offsets.is_empty() {
            return Ok(Vec::new());
        }

        // Limit after fetching so holes/expired entries don't cause under-reads.
        let keys: Vec<String> = offsets
            .iter()
            .map(|off| record_key(shard, 0, *off))
            .collect();
        let batch_results = self
            .rocksdb_engine_handler
            .multi_get::<StorageRecord>(cf, &keys)?;
        let mut records = Vec::new();
        let mut total_size = 0;

        for record_opt in batch_results {
            let Some(record) = record_opt else {
                continue;
            };

            if is_record_expired(&record.metadata) {
                continue;
            }

            if records.len() >= read_config.max_record_num as usize {
                break;
            }

            let record_bytes = record.data.len() as u64;
            if !records.is_empty() && total_size + record_bytes > read_config.max_size {
                break;
            }

            total_size += record_bytes;
            records.push(record);
        }

        Ok(records)
    }

    pub async fn read_by_key(
        &self,
        shard: &str,
        key: &[u8],
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let index = if let Some(index) = self.get_offset_by_key(shard, key).await? {
            index
        } else {
            return Ok(Vec::new());
        };

        let cf: std::sync::Arc<rocksdb::BoundColumnFamily<'_>> = self.get_cf()?;
        let record_key = record_key(shard, 0, index.offset);
        let Some(record) = self
            .rocksdb_engine_handler
            .read::<StorageRecord>(cf, &record_key)?
        else {
            return Ok(Vec::new());
        };

        if is_record_expired(&record.metadata) {
            return Ok(Vec::new());
        }

        Ok(vec![record])
    }

    pub async fn get_offset_by_key(
        &self,
        shard: &str,
        key: &[u8],
    ) -> Result<Option<IndexInfo>, StorageEngineError> {
        let cf = self.get_cf()?;
        let key_index = key_index_key(shard, key);

        let key_offset_bytes = match self.rocksdb_engine_handler.db.get_cf(&cf, &key_index) {
            Ok(Some(data)) => data,
            Ok(_) => return Ok(None),
            Err(e) => {
                return Err(StorageEngineError::CommonErrorStr(format!(
                    "Failed to read key offset: {e:?}"
                )))
            }
        };

        Ok(Some(deserialize::<IndexInfo>(&key_offset_bytes)?))
    }

    pub async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, StorageEngineError> {
        // Scan from the log head even without an index hint, else retention can stall.
        let index = self.search_index_by_timestamp(shard, timestamp).await?;
        if let Some(found_offset) = self.read_data_by_time(shard, &index, timestamp).await? {
            return Ok(found_offset);
        }
        match strategy {
            AdapterOffsetStrategy::Earliest => {
                Ok(self.commitlog_offset.get_earliest_offset(shard)?)
            }
            AdapterOffsetStrategy::Latest => Ok(self.commitlog_offset.get_latest_offset(shard)?),
        }
    }

    pub async fn search_index_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<IndexInfo>, StorageEngineError> {
        let cf = self.get_cf()?;
        let timestamp_index_prefix = timestamp_index_prefix(shard);
        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(&timestamp_index_prefix);

        let mut last_index = None;
        while iter.valid() {
            let Some(key_bytes) = iter.key() else {
                break;
            };
            if !key_bytes.starts_with(timestamp_index_prefix.as_bytes()) {
                break;
            }
            let Some(value_byte) = iter.value() else {
                break;
            };

            let index = deserialize::<IndexInfo>(value_byte)?;

            if last_index.is_none() {
                last_index = Some(index.clone());
            }

            if index.create_time > timestamp {
                return Ok(last_index);
            }
            last_index = Some(index);

            iter.next();
        }

        Ok(last_index)
    }

    async fn read_data_by_time(
        &self,
        shard: &str,
        start_index: &Option<IndexInfo>,
        timestamp: u64,
    ) -> Result<Option<u64>, StorageEngineError> {
        const MAX_SCAN: u64 = 10000;
        let cf = self.get_cf()?;
        let prefix = record_prefix(shard, 0);
        let seek_key = match start_index {
            Some(si) => record_key(shard, 0, si.offset),
            None => prefix.clone(),
        };

        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(seek_key.as_bytes());

        let mut scanned = 0u64;
        while iter.valid() && scanned < MAX_SCAN {
            let Some(key_bytes) = iter.key() else {
                break;
            };
            if !key_bytes.starts_with(prefix.as_bytes()) {
                break;
            }
            let Some(value_byte) = iter.value() else {
                break;
            };

            if let Ok(engine_record) = deserialize::<StorageRecord>(value_byte) {
                if engine_record.metadata.create_t >= timestamp {
                    return Ok(Some(engine_record.metadata.offset));
                }
            }

            scanned += 1;
            iter.next();
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::core::cache::StorageCacheManager;
    use crate::core::offset::ShardOffset;
    use crate::core::test_tool::test_build_rocksdb_engine;
    use broker_core::cache::NodeCacheManager;
    use common_base::uuid::unique_id;
    use common_config::config::BrokerConfig;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;

    #[tokio::test]
    async fn test_batch_write_and_read_by_offset() {
        let engine = test_build_rocksdb_engine();
        let shard_name = unique_id();
        let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
        let commit_offset =
            ShardOffset::new(cache_manager.clone(), engine.rocksdb_engine_handler.clone());

        commit_offset.save_earliest_offset(&shard_name, 0).unwrap();
        commit_offset.save_latest_offset(&shard_name, 0).unwrap();

        let messages: Vec<AdapterWriteRecord> = (0..10)
            .map(|i| {
                AdapterWriteRecord::new("", bytes::Bytes::default())
                    .with_key(format!("key{}", i))
                    .with_tags(vec![format!("tag{}", i % 3)])
            })
            .collect();

        let write_result = engine.batch_write(&shard_name, &messages).await.unwrap();
        assert_eq!(write_result.len(), 10);
        assert_eq!(write_result[0].offset, 0);
        assert_eq!(write_result[9].offset, 9);

        let read_config = AdapterReadConfig {
            max_record_num: 10,
            max_size: 1024 * 1024,
        };
        let records = engine
            .read_by_offset(&shard_name, 0, &read_config)
            .await
            .unwrap();
        assert_eq!(records.len(), 10);
        assert_eq!(records[0].metadata.offset, 0);
        assert_eq!(records[9].metadata.offset, 9);

        let tag_records = engine
            .read_by_tag(&shard_name, "tag0", None, &read_config)
            .await
            .unwrap();
        assert_eq!(tag_records.len(), 4);
        assert_eq!(tag_records[0].metadata.offset, 0);
        assert_eq!(tag_records[3].metadata.offset, 9);

        let key_records = engine.read_by_key(&shard_name, b"key5").await.unwrap();
        assert_eq!(key_records.len(), 1);
        assert_eq!(key_records[0].metadata.offset, 5);
    }
}
