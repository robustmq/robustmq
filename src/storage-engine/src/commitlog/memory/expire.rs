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
    commitlog::memory::engine::{MemoryShardData, MemoryStorageEngine},
    core::error::StorageEngineError,
};
use common_base::{
    error::ResultCommonError,
    tools::{loop_select_ticket, now_second},
};
use common_config::storage::StorageType;
use metadata_struct::storage::shard::EngineShard;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast;

impl MemoryStorageEngine {
    pub async fn start_expire_task(&self, stop_send: &broadcast::Sender<bool>) {
        let ac_fn = async || -> ResultCommonError {
            self.scan_and_delete_expire_data();
            Ok(())
        };
        loop_select_ticket(ac_fn, 10000, stop_send).await;
    }

    pub(crate) fn scan_and_delete_expire_data(&self) {
        let shard_infos: Vec<EngineShard> = self
            .cache_manager
            .shards
            .iter()
            .filter(|e| e.value().config.storage_type == StorageType::EngineMemory)
            .map(|e| e.value().clone())
            .collect();

        for shard_info in shard_infos {
            let Some(shard) = self.shards.get(&shard_info.shard_name) else {
                continue;
            };
            let _ = self.expire_by_time(&shard_info, &shard);
            if let Some(max_record_num) = shard_info.config.max_record_num {
                let _ = self.evict_by_size(&shard_info.shard_name, max_record_num, &shard);
            }
        }
    }

    fn expire_by_time(
        &self,
        shard_info: &EngineShard,
        shard: &Arc<MemoryShardData>,
    ) -> Result<(), StorageEngineError> {
        if shard_info.config.retention_sec == 0 {
            return Ok(());
        }

        let earliest_timestamp = now_second().saturating_sub(shard_info.config.retention_sec);
        let earliest_offset = self
            .commit_log_offset
            .get_earliest_offset(&shard_info.shard_name)?;

        let mut offsets: Vec<u64> = shard
            .data
            .iter()
            .filter(|e| {
                *e.key() >= earliest_offset && e.value().metadata.create_t < earliest_timestamp
            })
            .map(|e| *e.key())
            .collect();

        if offsets.is_empty() {
            return Ok(());
        }
        offsets.sort_unstable();

        Self::remove_offsets(shard, &offsets);
        let new_earliest = Self::contiguous_end(earliest_offset, &offsets);
        self.advance_earliest(&shard_info.shard_name, shard, earliest_offset, new_earliest)
    }

    pub(crate) fn evict_by_size(
        &self,
        shard_name: &str,
        max_record_num: u64,
        shard: &Arc<MemoryShardData>,
    ) -> Result<(), StorageEngineError> {
        if max_record_num == 0 || shard.data.len() as u64 <= max_record_num {
            return Ok(());
        }

        let earliest_offset = self.commit_log_offset.get_earliest_offset(shard_name)?;
        let discard_num = (shard.data.len() as f64 * self.config.evict_ratio) as u64;
        if discard_num == 0 {
            return Ok(());
        }

        let offsets: Vec<u64> = (earliest_offset..earliest_offset + discard_num).collect();
        Self::remove_offsets(shard, &offsets);
        self.advance_earliest(
            shard_name,
            shard,
            earliest_offset,
            earliest_offset + discard_num,
        )
    }

    fn contiguous_end(from: u64, sorted_offsets: &[u64]) -> u64 {
        let mut next = from;
        for &o in sorted_offsets {
            if o == next {
                next += 1;
            } else if o > next {
                break;
            }
        }
        next
    }

    fn remove_offsets(shard: &Arc<MemoryShardData>, offsets: &[u64]) {
        let mut removed: HashSet<u64> = HashSet::with_capacity(offsets.len());
        for &offset in offsets {
            if let Some((_, record)) = shard.data.remove(&offset) {
                removed.insert(offset);
                if let Some(key) = &record.metadata.key {
                    shard.key_index.remove(key);
                }
            }
        }

        if removed.is_empty() {
            return;
        }

        shard.tag_index.retain(|_, tag_offsets| {
            tag_offsets.retain(|o| !removed.contains(o));
            !tag_offsets.is_empty()
        });
    }

    fn advance_earliest(
        &self,
        shard_name: &str,
        shard: &Arc<MemoryShardData>,
        old: u64,
        new: u64,
    ) -> Result<(), StorageEngineError> {
        if new <= old {
            return Ok(());
        }
        self.commit_log_offset
            .save_earliest_offset(shard_name, new)?;
        shard.timestamp_index.retain(|_, &mut o| o >= new);
        Ok(())
    }
}
