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
use common_base::{error::ResultCommonError, tools::loop_select_ticket};
use std::sync::Arc;
use tokio::sync::broadcast;

impl MemoryStorageEngine {
    pub fn start_expire_task(&self, stop_send: &broadcast::Sender<bool>) {
        let engine = self.clone();
        let stop_send = stop_send.clone();
        tokio::spawn(async move {
            let ac_fn = async || -> ResultCommonError {
                engine.run_expire_once();
                Ok(())
            };
            loop_select_ticket(ac_fn, 10000, &stop_send).await;
        });
    }

    fn run_expire_once(&self) {
        let shard_names: Vec<String> = self.shards.iter().map(|e| e.key().clone()).collect();
        for shard_name in shard_names {
            if let Some(shard) = self.shards.get(&shard_name) {
                let _ = self.evict_shard(&shard_name, &shard);
            }
        }
    }

    pub(crate) fn evict_shard(
        &self,
        shard_name: &str,
        shard: &Arc<ShardState>,
    ) -> Result<(), StorageEngineError> {
        if shard.data.len() <= self.config.max_records_per_shard {
            return Ok(());
        }

        let offset = self.commit_log_offset.get_earliest_offset(shard_name)?;
        let discard_num = (shard.data.len() as f64 * 0.2) as u64;

        for i in offset..(offset + discard_num) {
            if let Some((_, record)) = shard.data.remove(&i) {
                if let Some(key) = &record.metadata.key {
                    shard.key_index.remove(key);
                }
            }
        }

        let new_earliest = offset + discard_num;
        self.commit_log_offset
            .save_earliest_offset(shard_name, new_earliest)?;
        self.cleanup_indexes_by_offset(shard, new_earliest);
        Ok(())
    }

    fn cleanup_indexes_by_offset(&self, shard: &Arc<ShardState>, earliest_offset: u64) {
        for mut offsets in shard.tag_index.iter_mut() {
            offsets.value_mut().retain(|&o| o >= earliest_offset);
        }
        shard.tag_index.retain(|_, v| !v.is_empty());
        shard.key_index.retain(|_, &mut o| o >= earliest_offset);
        shard
            .timestamp_index
            .retain(|_, &mut o| o >= earliest_offset);
    }
}
