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

use common_base::utils::serialize::deserialize;
use metadata_struct::storage::{
    adapter_read_config::AdapterReadConfig, storage_record::StorageRecord,
};
use rocksdb_engine::keys::storage::{key_index_key, shard_record_key, tag_index_tag_prefix};

use crate::{
    core::error::StorageEngineError,
    rocksdb::engine::{IndexInfo, RocksDBStorageEngine},
};

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
impl RocksDBStorageEngine {
    pub async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let cf = self.get_cf()?;

        let keys: Vec<String> = (offset..offset.saturating_add(read_config.max_record_num))
            .map(|i| shard_record_key(shard, i))
            .collect();

        let mut records = Vec::new();
        let mut total_size = 0;

        let batch_results = self
            .rocksdb_engine_handler
            .multi_get::<StorageRecord>(cf, &keys)?;
        for record_opt in batch_results {
            let Some(record) = record_opt else {
                break;
            };

            let record_bytes = record.data.len() as u64;
            if total_size + record_bytes > read_config.max_size {
                break;
            }

            total_size += record_bytes;
            records.push(record);
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
        let tag_offset_key_prefix = tag_index_tag_prefix(shard, tag);
        let tag_entries = self
            .rocksdb_engine_handler
            .read_prefix(cf.clone(), &tag_offset_key_prefix)?;

        // Filter and collect offsets >= specified offset
        let mut offsets = Vec::new();
        for (_key, value) in tag_entries {
            let record_offset = deserialize::<IndexInfo>(&value)?;

            if let Some(so) = start_offset {
                if record_offset.offset < so {
                    continue;
                }
            }

            offsets.push(record_offset.offset);
            if offsets.len() >= read_config.max_record_num as usize {
                break;
            }
        }

        if offsets.is_empty() {
            return Ok(Vec::new());
        }

        // Build record keys from offsets
        let keys: Vec<String> = offsets
            .iter()
            .map(|off| shard_record_key(shard, *off))
            .collect();

        // Batch read records
        let batch_results = self
            .rocksdb_engine_handler
            .multi_get::<StorageRecord>(cf, &keys)?;
        let mut records = Vec::new();
        let mut total_size = 0;

        for record_opt in batch_results {
            let Some(record) = record_opt else {
                continue;
            };

            let record_bytes = record.data.len() as u64;
            if total_size + record_bytes > read_config.max_size {
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
        key: &str,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let cf = self.get_cf()?;
        let key_index = key_index_key(shard, key);

        let key_offset_bytes = match self.rocksdb_engine_handler.db.get_cf(&cf, &key_index) {
            Ok(Some(data)) => data,
            Ok(_) => return Ok(Vec::new()),
            Err(e) => {
                return Err(StorageEngineError::CommonErrorStr(format!(
                    "Failed to read key offset: {e:?}"
                )))
            }
        };

        let index = deserialize::<IndexInfo>(&key_offset_bytes)?;

        let shard_record_key = shard_record_key(shard, index.offset);
        let Some(record) = self
            .rocksdb_engine_handler
            .read::<StorageRecord>(cf, &shard_record_key)?
        else {
            return Ok(Vec::new());
        };

        Ok(vec![record])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::shard::{ShardState, StorageEngineRunType};
    use crate::core::test_tool::test_build_engine;
    use common_base::tools::unique_id;
    use metadata_struct::storage::adapter_offset::AdapterOffsetStrategy;
    use metadata_struct::storage::adapter_record::AdapterWriteRecord;

    #[tokio::test]
    async fn test_batch_write_and_read_by_offset() {
        let engine = test_build_engine(StorageEngineRunType::Standalone);
        let shard_name = unique_id();
        engine
            .shard_state
            .insert(shard_name.clone(), ShardState::default());

        let messages: Vec<AdapterWriteRecord> = (0..10)
            .map(|i| AdapterWriteRecord {
                pkid: i,
                key: Some(format!("key{}", i)),
                tags: Some(vec![format!("tag{}", i % 3)]),
                timestamp: 1000 + i * 100,
                ..Default::default()
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

        let key_records = engine.read_by_key(&shard_name, "key5").await.unwrap();
        assert_eq!(key_records.len(), 1);
        assert_eq!(key_records[0].metadata.offset, 5);

        let offset_by_ts = engine
            .get_offset_by_timestamp(&shard_name, 1500, AdapterOffsetStrategy::Latest)
            .await
            .unwrap();
        assert_eq!(offset_by_ts, 5);
    }
}
