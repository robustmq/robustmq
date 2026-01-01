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

use metadata_struct::storage::{
    adapter_read_config::AdapterReadConfig, storage_record::StorageRecord,
};

use crate::{core::error::StorageEngineError, memory::engine::MemoryStorageEngine};

impl MemoryStorageEngine {
    pub async fn read_by_offset(
        &self,
        shard: &str,
        start_offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let Some(data_map) = self.shard_data.get(shard) else {
            return Ok(Vec::new());
        };

        let mut records = Vec::with_capacity(read_config.max_record_num as usize);
        let mut total_size = 0;
        let end_offset = start_offset.saturating_add(read_config.max_record_num);

        for current_offset in start_offset..end_offset {
            let Some(record) = data_map.get(&current_offset) else {
                break;
            };

            let record_bytes = record.data.len() as u64;
            if total_size + record_bytes > read_config.max_size {
                break;
            }

            total_size += record_bytes;
            records.push(record.clone());

            if records.len() >= read_config.max_record_num as usize {
                break;
            }
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
        let Some(tag_map) = self.tag_index.get(shard) else {
            return Ok(Vec::new());
        };

        let Some(offsets_list) = tag_map.get(tag) else {
            return Ok(Vec::new());
        };

        let Some(data_map) = self.shard_data.get(shard) else {
            return Ok(Vec::new());
        };

        let capacity = read_config.max_record_num.min(offsets_list.len() as u64) as usize;
        let mut records = Vec::with_capacity(capacity);
        let mut total_size = 0;

        for &offset in offsets_list.iter() {
            if let Some(so) = start_offset {
                if offset < so {
                    continue;
                }
            }

            let Some(record) = data_map.get(&offset) else {
                continue;
            };

            let record_bytes = record.data.len() as u64;
            if total_size + record_bytes > read_config.max_size {
                break;
            }

            total_size += record_bytes;
            records.push(record.clone());

            if records.len() >= read_config.max_record_num as usize {
                break;
            }
        }

        Ok(records)
    }

    pub async fn read_by_key(
        &self,
        shard: &str,
        key: &str,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let Some(key_map) = self.key_index.get(shard) else {
            return Ok(Vec::new());
        };

        let Some(offset) = key_map.get(key) else {
            return Ok(Vec::new());
        };

        let Some(data_map) = self.shard_data.get(shard) else {
            return Ok(Vec::new());
        };

        let Some(record) = data_map.get(&offset) else {
            return Ok(Vec::new());
        };

        Ok(vec![record.clone()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::shard::ShardState;
    use crate::core::test_tool::test_build_memory_engine;
    use common_base::tools::unique_id;
    use metadata_struct::storage::adapter_record::AdapterWriteRecord;

    #[tokio::test]
    async fn test_batch_write_and_read_by_offset() {
        let engine = test_build_memory_engine(crate::core::shard::StorageEngineRunType::Standalone);
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
        let key_records = engine.read_by_key(&shard_name, "key5").await.unwrap();
        assert_eq!(key_records.len(), 1);
        assert_eq!(key_records[0].metadata.offset, 5);
        let offset_by_ts = engine
            .get_offset_by_timestamp(
                &shard_name,
                1500,
                metadata_struct::storage::adapter_offset::AdapterOffsetStrategy::Latest,
            )
            .await
            .unwrap();
        assert_eq!(offset_by_ts, Some(5));
    }
}
