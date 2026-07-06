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

use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::core::error::StorageEngineError;
use crate::isr::log::ReplicaLog;
use async_trait::async_trait;
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use metadata_struct::storage::record::StorageRecord;

#[async_trait]
impl ReplicaLog for MemoryStorageEngine {
    async fn append_at(
        &self,
        shard: &str,
        _segment_seq: u32,
        base_offset: u64,
        records: Vec<StorageRecord>,
    ) -> Result<(), StorageEngineError> {
        let shard_state = self.get_or_create_shard(shard);
        let leo = self.commit_log_offset.get_latest_offset(shard)?;
        if base_offset != leo {
            return Err(StorageEngineError::OutOfOrder(
                shard.to_string(),
                base_offset,
                leo,
            ));
        }

        let mut new_leo = leo;
        for record in records {
            new_leo = record.metadata.offset + 1;
            shard_state.data.insert(record.metadata.offset, record);
        }
        self.commit_log_offset.save_latest_offset(shard, new_leo)?;
        Ok(())
    }

    async fn read_from(
        &self,
        shard: &str,
        _segment_seq: u32,
        offset: u64,
        max_bytes: u64,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        let read_config = AdapterReadConfig {
            max_record_num: u64::MAX,
            max_size: max_bytes,
        };
        self.read_by_offset(shard, offset, &read_config).await
    }

    fn latest_offset(&self, shard: &str, _segment_seq: u32) -> Result<u64, StorageEngineError> {
        self.commit_log_offset.get_latest_offset(shard)
    }

    async fn truncate_to(
        &self,
        shard: &str,
        _segment_seq: u32,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        let shard_state = self.get_or_create_shard(shard);
        shard_state.data.retain(|&o, _| o <= offset);
        let new_leo = self
            .commit_log_offset
            .get_latest_offset(shard)?
            .min(offset + 1);
        self.commit_log_offset.save_latest_offset(shard, new_leo)?;
        Ok(())
    }

    async fn clear(&self, shard: &str, _segment_seq: u32) -> Result<(), StorageEngineError> {
        let shard_state = self.get_or_create_shard(shard);
        shard_state.data.clear();
        self.commit_log_offset.save_latest_offset(shard, 0)?;
        Ok(())
    }

    fn log_start_offset(&self, shard: &str, _segment_seq: u32) -> Result<u64, StorageEngineError> {
        self.commit_log_offset.get_earliest_offset(shard)
    }

    fn update_high_watermark(&self, shard: &str, hw: u64) -> Result<(), StorageEngineError> {
        self.commit_log_offset
            .save_high_watermark_offset(shard, hw)?;
        Ok(())
    }
}
