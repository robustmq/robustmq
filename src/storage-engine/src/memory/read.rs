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
