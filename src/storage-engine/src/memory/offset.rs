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
        shard::ShardState,
        shard_offset::{
            get_earliest_offset, get_latest_offset, save_earliest_offset_by_shard,
            save_latest_offset_by_shard,
        },
    },
    memory::engine::MemoryStorageEngine,
};
use metadata_struct::storage::adapter_offset::AdapterOffsetStrategy;

impl MemoryStorageEngine {
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
            AdapterOffsetStrategy::Earliest => Ok(self.get_earliest_offset(shard)?),
            AdapterOffsetStrategy::Latest => Ok(self.get_latest_offset(shard)?),
        }
    }

    pub fn save_latest_offset(&self, shard: &str, offset: u64) -> Result<(), StorageEngineError> {
        save_latest_offset_by_shard(&self.rocksdb_engine_handler, shard, offset)?;

        if let Some(mut state) = self.shard_state.get_mut(shard) {
            state.latest_offset = offset;
            Ok(())
        } else {
            Err(StorageEngineError::CommonErrorStr(format!(
                "Shard [{}] state not found when saving latest offset",
                shard
            )))
        }
    }

    pub fn get_latest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        if let Some(state) = self.shard_state.get(shard_name) {
            Ok(state.latest_offset)
        } else {
            let (_, latest_offset) = self.recover_shard_data(shard_name)?;
            Ok(latest_offset)
        }
    }

    pub fn save_earliest_offset(&self, shard: &str, offset: u64) -> Result<(), StorageEngineError> {
        save_earliest_offset_by_shard(&self.rocksdb_engine_handler, shard, offset)?;

        if let Some(mut state) = self.shard_state.get_mut(shard) {
            state.earliest_offset = offset;
            Ok(())
        } else {
            Err(StorageEngineError::CommonErrorStr(format!(
                "Shard [{}] state not found when saving earliest offset",
                shard
            )))
        }
    }

    pub fn get_earliest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        if let Some(state) = self.shard_state.get(shard_name) {
            Ok(state.earliest_offset)
        } else {
            let (earliest_offset, _) = self.recover_shard_data(shard_name)?;
            Ok(earliest_offset)
        }
    }

    fn search_index_by_timestamp(&self, shard: &str, timestamp: u64) -> Option<u64> {
        let ts_map = self.timestamp_index.get(shard)?;

        ts_map
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
        let data_map = self.shard_data.get(shard)?;
        let shard_state = self.shard_state.get(shard)?;

        let start = start_offset.unwrap_or(0);
        let end = shard_state.latest_offset;

        const MAX_SCAN: u64 = 10000;
        let scan_end = end.min(start + MAX_SCAN);

        for offset in start..scan_end {
            let Some(record) = data_map.get(&offset) else {
                continue;
            };

            if record.metadata.create_t >= timestamp {
                return Some(offset);
            }
        }

        None
    }

    fn recover_shard_data(&self, shard_name: &str) -> Result<(u64, u64), StorageEngineError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_build_memory_engine;
    use common_base::tools::unique_id;

    #[tokio::test]
    async fn test_latest_offset() {
        let engine = test_build_memory_engine();
        let shard_name = unique_id();
        engine
            .shard_state
            .insert(shard_name.clone(), ShardState::default());
        engine.save_latest_offset(&shard_name, 100).unwrap();
        let offset = engine.get_latest_offset(&shard_name).unwrap();
        assert_eq!(offset, 100);
    }

    #[tokio::test]
    async fn test_earliest_offset() {
        let engine = test_build_memory_engine();
        let shard_name = unique_id();
        engine
            .shard_state
            .insert(shard_name.clone(), ShardState::default());
        engine.save_earliest_offset(&shard_name, 50).unwrap();
        let offset = engine.get_earliest_offset(&shard_name).unwrap();
        assert_eq!(offset, 50);
    }
}
