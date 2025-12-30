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
    core::{
        error::StorageEngineError,
        shard_offset::{
            get_earliest_offset, get_latest_offset, save_earliest_offset_by_shard,
            save_latest_offset_by_shard,
        },
    },
    memory::engine::{MemoryStorageEngine, MemoryStorageType, ShardState},
};
use metadata_struct::storage::adapter_offset::{AdapterConsumerGroupOffset, AdapterOffsetStrategy};

impl MemoryStorageEngine {
    pub async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
        _strategy: AdapterOffsetStrategy,
    ) -> Result<Option<AdapterConsumerGroupOffset>, StorageEngineError> {
        let index_offset = self.search_index_by_timestamp(shard, timestamp);

        if let Some(offset) = self.read_data_by_time(shard, index_offset, timestamp) {
            return Ok(Some(AdapterConsumerGroupOffset {
                shard_name: shard.to_string(),
                offset,
                ..Default::default()
            }));
        }

        Ok(None)
    }

    pub fn save_latest_offset(&self, shard: &str, offset: u64) -> Result<(), StorageEngineError> {
        if let Some(mut state) = self.shard_state.get_mut(shard) {
            state.latest_offset = offset;
            if self.engine_type == MemoryStorageType::EngineStorage {
                save_latest_offset_by_shard(&self.rocksdb_engine_handler, shard, offset)?;
            }
            Ok(())
        } else {
            Err(StorageEngineError::CommonErrorStr(format!(
                "Shard [{}] state not found when saving latest offset",
                shard
            )))
        }
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

    pub fn get_latest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        match self.engine_type {
            MemoryStorageType::Standalone => {
                let shard_state: ShardState = self
                    .shard_state
                    .entry(shard_name.to_owned())
                    .or_default()
                    .clone();

                Ok(shard_state.latest_offset)
            }

            MemoryStorageType::EngineStorage => {
                if let Some(state) = self.shard_state.get(shard_name) {
                    Ok(state.latest_offset)
                } else {
                    let (_, latest_offset) = self.recover_shard_data(shard_name)?;
                    Ok(latest_offset)
                }
            }
        }
    }

    pub fn save_earliest_offset(&self, shard: &str, offset: u64) -> Result<(), StorageEngineError> {
        if let Some(mut state) = self.shard_state.get_mut(shard) {
            state.earliest_offset = offset;
            if self.engine_type == MemoryStorageType::EngineStorage {
                save_earliest_offset_by_shard(&self.rocksdb_engine_handler, shard, offset)?;
            }
            Ok(())
        } else {
            Err(StorageEngineError::CommonErrorStr(format!(
                "Shard [{}] state not found when saving earliest offset",
                shard
            )))
        }
    }

    pub fn get_earliest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        match self.engine_type {
            MemoryStorageType::Standalone => {
                let shard_state: ShardState = self
                    .shard_state
                    .entry(shard_name.to_owned())
                    .or_default()
                    .clone();

                Ok(shard_state.earliest_offset)
            }

            MemoryStorageType::EngineStorage => {
                if let Some(state) = self.shard_state.get(shard_name) {
                    Ok(state.latest_offset)
                } else {
                    let (earliest_offset, _) = self.recover_shard_data(shard_name)?;
                    Ok(earliest_offset)
                }
            }
        }
    }

    pub fn recover_shard_data(&self, shard_name: &str) -> Result<(u64, u64), StorageEngineError> {
        let earliest_offset = get_earliest_offset(
            &self.rocksdb_engine_handler,
            &self.cache_manager,
            shard_name,
        )?;

        let latest_offset = get_latest_offset(
            &self.rocksdb_engine_handler,
            &self.cache_manager,
            shard_name,
        )?;

        self.shard_state.insert(
            shard_name.to_string(),
            ShardState {
                earliest_offset,
                latest_offset,
            },
        );

        Ok((earliest_offset, latest_offset))
    }
}
