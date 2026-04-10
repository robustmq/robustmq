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
    commitlog::memory::engine::{MemoryStorageEngine, ShardState},
    core::error::StorageEngineError,
};
use common_base::{
    error::ResultCommonError,
    tools::{loop_select_ticket, now_second},
};
use common_config::storage::StorageType;
use metadata_struct::storage::shard::EngineShard;
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

    fn scan_and_delete_expire_data(&self) {
        let shard_infos: Vec<EngineShard> = self
            .cache_manager
            .shards
            .iter()
            .filter(|e| e.value().config.storage_type == StorageType::EngineMemory)
            .map(|e| e.value().clone())
            .collect();

        for shard_info in shard_infos {
            if let Some(shard) = self.shards.get(&shard_info.shard_name) {
                let _ = self.scan_and_delete_data_by_shard(&shard_info, &shard);
            }
            if let Some(shard) = self.shards.get(&shard_info.shard_name) {
                if let Some(max_record_num) = shard_info.config.max_record_num {
                    let _ = self.evict_shard(&shard_info.shard_name, max_record_num, &shard);
                }
            }
        }
    }

    fn scan_and_delete_data_by_shard(
        &self,
        shard_info: &EngineShard,
        shard: &Arc<ShardState>,
    ) -> Result<(), StorageEngineError> {
        if shard_info.config.retention_sec == 0 {
            return Ok(());
        }

        let earliest_timestamp = now_second().saturating_sub(shard_info.config.retention_sec);
        let earliest_offset = self
            .commit_log_offset
            .get_earliest_offset(&shard_info.shard_name)?;

        let mut offsets_to_delete: Vec<u64> = shard
            .data
            .iter()
            .filter(|e| {
                let offset = *e.key();
                offset >= earliest_offset && e.value().metadata.create_t < earliest_timestamp
            })
            .map(|e| *e.key())
            .collect();

        if offsets_to_delete.is_empty() {
            return Ok(());
        }

        offsets_to_delete.sort_unstable();

        for offset in &offsets_to_delete {
            if let Some((_, record)) = shard.data.remove(offset) {
                if let Some(key) = &record.metadata.key {
                    shard.key_index.remove(key);
                }
                if let Some(tags) = &record.metadata.tags {
                    for tag in tags.iter() {
                        if let Some(mut offsets) = shard.tag_index.get_mut(tag) {
                            offsets.retain(|&o| o != *offset);
                        }
                    }
                }
            }
        }

        shard.tag_index.retain(|_, v| !v.is_empty());

        let new_earliest = Self::contiguous_end(earliest_offset, &offsets_to_delete);
        if new_earliest > earliest_offset {
            self.commit_log_offset
                .save_earliest_offset(&shard_info.shard_name, new_earliest)?;
            self.cleanup_timestamp_index(shard, new_earliest);
        }

        Ok(())
    }

    // Only advance earliest_offset through a contiguous run from `from`.
    // Skipping a gap would make records at intermediate offsets permanently inaccessible.
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

    pub(crate) fn evict_shard(
        &self,
        shard_name: &str,
        max_record_num: u64,
        shard: &Arc<ShardState>,
    ) -> Result<(), StorageEngineError> {
        if max_record_num == 0 || shard.data.len() as u64 <= max_record_num {
            return Ok(());
        }

        let offset = self.commit_log_offset.get_earliest_offset(shard_name)?;
        // Evict evict_ratio of current size to avoid continuous eviction loops under
        // sustained write load.
        let discard_num = (shard.data.len() as f64 * self.config.evict_ratio) as u64;

        for i in offset..(offset + discard_num) {
            if let Some((_, record)) = shard.data.remove(&i) {
                if let Some(key) = &record.metadata.key {
                    shard.key_index.remove(key);
                }
                if let Some(tags) = &record.metadata.tags {
                    for tag in tags.iter() {
                        if let Some(mut tag_offsets) = shard.tag_index.get_mut(tag) {
                            tag_offsets.retain(|&o| o != i);
                        }
                    }
                }
            }
        }

        shard.tag_index.retain(|_, v| !v.is_empty());

        let new_earliest = offset + discard_num;
        self.commit_log_offset
            .save_earliest_offset(shard_name, new_earliest)?;
        self.cleanup_timestamp_index(shard, new_earliest);
        Ok(())
    }

    fn cleanup_timestamp_index(&self, shard: &Arc<ShardState>, earliest_offset: u64) {
        shard
            .timestamp_index
            .retain(|_, &mut o| o >= earliest_offset);
    }
}
