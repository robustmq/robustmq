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
        offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let Some(data_map) = self.shard_data.get(shard) else {
            return Ok(Vec::new());
        };

        let mut records = Vec::new();
        let mut total_size = 0;
        let end_offset = offset.saturating_add(read_config.max_record_num);

        for current_offset in offset..end_offset {
            let Some(record) = data_map.get(&current_offset) else {
                break;
            };

            let record_bytes = record.data.len() as u64;
            if total_size + record_bytes > read_config.max_size {
                break;
            }

            total_size += record_bytes;
            records.push(record.clone());
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

        let mut records = Vec::new();
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

    pub fn search_index_by_timestamp(&self, shard: &str, timestamp: u64) -> Option<u64> {
        let ts_map = self.timestamp_index.get(shard)?;

        let mut entries: Vec<(u64, u64)> = ts_map
            .iter()
            .map(|entry| (*entry.key(), *entry.value()))
            .collect();

        entries.sort_by_key(|(ts, _)| *ts);

        let mut found_offset = None;
        for (ts, offset) in entries {
            if ts > timestamp {
                break;
            }
            found_offset = Some(offset);
        }

        found_offset
    }

    pub fn read_data_by_time(
        &self,
        shard: &str,
        start_offset: Option<u64>,
        timestamp: u64,
    ) -> Option<u64> {
        let data_map = self.shard_data.get(shard)?;
        let shard_state = self.shard_state.get(shard)?;

        let start = start_offset.unwrap_or(0);
        let end = shard_state.latest_offset;

        for offset in start..end {
            let Some(record) = data_map.get(&offset) else {
                continue;
            };

            if record.metadata.create_t >= timestamp {
                return Some(offset);
            }
        }

        None
    }
}
