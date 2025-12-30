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

use std::sync::Arc;

use dashmap::DashMap;
use metadata_struct::storage::adapter_offset::AdapterShardInfo;

use crate::{
    core::error::StorageEngineError,
    memory::engine::{MemoryStorageEngine, ShardState},
};

impl MemoryStorageEngine {
    pub async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), StorageEngineError> {
        self.storage_type_check()?;

        self.shard_data
            .insert(shard.shard_name.clone(), DashMap::with_capacity(8));
        self.shard_info
            .insert(shard.shard_name.clone(), shard.clone());
        self.shard_state
            .insert(shard.shard_name.clone(), ShardState::default());
        self.shard_write_locks
            .entry(shard.shard_name.clone())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())));
        Ok(())
    }

    pub async fn list_shard(
        &self,
        shard: Option<String>,
    ) -> Result<Vec<AdapterShardInfo>, StorageEngineError> {
        self.storage_type_check()?;

        if let Some(shard_name) = shard {
            Ok(self
                .shard_info
                .get(&shard_name)
                .map(|info| vec![info.clone()])
                .unwrap_or_default())
        } else {
            Ok(self
                .shard_info
                .iter()
                .map(|entry| entry.value().clone())
                .collect())
        }
    }

    pub async fn delete_shard(&self, shard_name: &str) -> Result<(), StorageEngineError> {
        self.storage_type_check()?;

        if !self.shard_info.contains_key(shard_name) {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "shard {} data not found",
                shard_name
            )));
        }

        self.shard_data.remove(shard_name);
        self.shard_info.remove(shard_name);
        self.shard_state.remove(shard_name);
        self.shard_write_locks.remove(shard_name);
        self.remove_indexes(shard_name);

        for mut group_entry in self.group_data.iter_mut() {
            group_entry.value_mut().remove(shard_name);
        }

        Ok(())
    }
}
