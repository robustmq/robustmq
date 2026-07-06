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
    commitlog::memory::engine::MemoryStorageEngine,
    core::{error::StorageEngineError, message_ttl::is_record_expired},
};
use metadata_struct::storage::{
    adapter_offset::AdapterOffsetStrategy, adapter_read_config::AdapterReadConfig,
    record::StorageRecord,
};

impl MemoryStorageEngine {
    pub async fn read_by_offset(
        &self,
        shard: &str,
        start_offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let Some(shard_state) = self.shards.get(shard).map(|s| s.clone()) else {
            return Ok(Vec::new());
        };

        let mut records = Vec::with_capacity(read_config.max_record_num.min(1024) as usize);
        let mut total_size = 0;
        let end_offset = self.commit_log_offset.get_latest_offset(shard)?;
        for current_offset in start_offset..end_offset {
            let Some(record) = shard_state.data.get(&current_offset) else {
                continue;
            };

            if is_record_expired(&record.metadata) {
                continue;
            }

            let record_bytes = record.data.len() as u64;
            if !records.is_empty() && total_size + record_bytes > read_config.max_size {
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
        let Some(shard_state) = self.shards.get(shard).map(|s| s.clone()) else {
            return Ok(Vec::new());
        };

        let Some(offsets_list) = shard_state.tag_index.get(tag) else {
            return Ok(Vec::new());
        };

        let start_idx = match start_offset {
            Some(so) => offsets_list.partition_point(|&o| o < so),
            None => 0,
        };

        let capacity = read_config
            .max_record_num
            .min((offsets_list.len() - start_idx) as u64) as usize;
        let mut records = Vec::with_capacity(capacity);
        let mut total_size = 0;

        for &offset in &offsets_list[start_idx..] {
            let Some(record) = shard_state.data.get(&offset) else {
                continue;
            };

            if is_record_expired(&record.metadata) {
                continue;
            }

            let record_bytes = record.data.len() as u64;
            if !records.is_empty() && total_size + record_bytes > read_config.max_size {
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
        key: &[u8],
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let Some(shard_state) = self.shards.get(shard).map(|s| s.clone()) else {
            return Ok(Vec::new());
        };

        let Some(offset) = shard_state.key_index.get(key).map(|o| *o) else {
            return Ok(Vec::new());
        };

        let Some(record) = shard_state.data.get(&offset) else {
            return Ok(Vec::new());
        };

        if is_record_expired(&record.metadata) {
            return Ok(Vec::new());
        }

        Ok(vec![record.clone()])
    }

    pub async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, StorageEngineError> {
        let index_offset = self.search_index_by_timestamp(shard, timestamp);

        if let Some(offset) = self.read_data_by_time(shard, index_offset, timestamp) {
            return Ok(offset);
        }

        match strategy {
            AdapterOffsetStrategy::Earliest => {
                Ok(self.commit_log_offset.get_earliest_offset(shard)?)
            }
            AdapterOffsetStrategy::Latest => Ok(self.commit_log_offset.get_latest_offset(shard)?),
        }
    }

    fn search_index_by_timestamp(&self, shard: &str, timestamp: u64) -> Option<u64> {
        let shard_state = self.shards.get(shard)?;

        shard_state
            .timestamp_index
            .iter()
            .filter(|entry| *entry.key() <= timestamp)
            .max_by_key(|entry| *entry.key())
            .map(|entry| *entry.value())
    }

    fn read_data_by_time(
        &self,
        shard: &str,
        start_offset: Option<u64>,
        timestamp: u64,
    ) -> Option<u64> {
        let shard_state = self.shards.get(shard).map(|s| s.clone())?;
        let shard_offset_state = self
            .commit_log_offset
            .cache_manager
            .get_offset_state(shard)?;

        let start = start_offset.unwrap_or(0);
        let end = shard_offset_state.latest_offset;

        const MAX_SCAN: u64 = 10000;
        let scan_end = end.min(start + MAX_SCAN);

        for offset in start..scan_end {
            let Some(record) = shard_state.data.get(&offset) else {
                continue;
            };

            if record.metadata.create_t >= timestamp {
                return Some(offset);
            }
        }

        None
    }
}
