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

use crate::expire::MessageExpireConfig;
use crate::storage::{ShardInfo, ShardOffset, StorageAdapter};
use axum::async_trait;
use common_base::error::common::CommonError;
use common_config::storage::memory::StorageDriverMemoryConfig;
use dashmap::DashMap;
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct ShardState {
    pub start_offset: u64,
    pub next_offset: u64,
}

#[derive(Clone)]
pub struct MemoryStorageAdapter {
    //(shard, (ShardInfo))
    pub shard_info: DashMap<String, ShardInfo>,
    //(shard, (offset,Record))
    pub shard_data: DashMap<String, DashMap<u64, Record>>,
    //(group, (shard, offset))
    pub group_data: DashMap<String, DashMap<String, u64>>,
    //(shard, (tag, (offset)))
    pub tag_index: DashMap<String, DashMap<String, Vec<u64>>>,
    //(shard, (key, offset))
    pub key_index: DashMap<String, DashMap<String, u64>>,
    //(shard, (timestamp, offset))
    pub timestamp_index: DashMap<String, DashMap<u64, u64>>,
    pub shard_state: DashMap<String, ShardState>,
    pub shard_write_locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
    pub config: StorageDriverMemoryConfig,
}

impl Default for MemoryStorageAdapter {
    fn default() -> Self {
        Self::new(StorageDriverMemoryConfig::default())
    }
}

impl MemoryStorageAdapter {
    pub fn new(config: StorageDriverMemoryConfig) -> Self {
        MemoryStorageAdapter {
            shard_info: DashMap::with_capacity(8),
            shard_data: DashMap::with_capacity(8),
            shard_state: DashMap::with_capacity(8),
            tag_index: DashMap::with_capacity(8),
            key_index: DashMap::with_capacity(8),
            timestamp_index: DashMap::with_capacity(8),
            group_data: DashMap::with_capacity(8),
            shard_write_locks: DashMap::with_capacity(8),
            config,
        }
    }

    fn remove_indexes(&self, shard_key: &str) {
        self.tag_index.remove(shard_key);
        self.key_index.remove(shard_key);
        self.timestamp_index.remove(shard_key);
    }

    async fn internal_batch_write(
        &self,
        shard_name: &str,
        messages: &[Record],
    ) -> Result<Vec<u64>, CommonError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        if !self.shard_info.contains_key(shard_name) {
            return Err(CommonError::CommonError(format!(
                "shard {} not exists",
                shard_name
            )));
        }

        let lock = self
            .shard_write_locks
            .entry(shard_name.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone();

        let _guard = lock.lock().await;

        let shard_state = if let Some(state) = self.shard_state.get(shard_name) {
            state.clone()
        } else {
            return Err(CommonError::CommonError(format!(
                "shard {} not exists",
                shard_name
            )));
        };

        let mut offset_res = Vec::new();
        let mut offset = shard_state.next_offset;
        if let Some(data_map) = self.shard_data.get(shard_name) {
            for msg in messages.iter() {
                offset_res.push(offset);

                // save data
                let mut record_to_save = msg.clone();
                record_to_save.offset = Some(offset);
                data_map.insert(offset, record_to_save.clone());

                // key index
                if let Some(key) = &msg.key {
                    let key_map = self.key_index.entry(shard_name.to_string()).or_default();
                    key_map.insert(key.clone(), offset);
                }

                // tag index
                if let Some(tags) = &msg.tags {
                    let tag_map = self.tag_index.entry(shard_name.to_string()).or_default();
                    for tag in tags.iter() {
                        tag_map.entry(tag.clone()).or_default().push(offset);
                    }
                }

                // timestamp index
                if msg.timestamp > 0 && offset % 5000 == 0 {
                    let timestamp_map = self
                        .timestamp_index
                        .entry(shard_name.to_string())
                        .or_default();
                    if !timestamp_map.contains_key(&msg.timestamp) {
                        timestamp_map.insert(msg.timestamp, offset);
                    }
                }
                offset += 1;
            }

            self.shard_state.insert(
                shard_name.to_string(),
                ShardState {
                    start_offset: shard_state.start_offset,
                    next_offset: offset,
                },
            );
            return Ok(offset_res);
        }

        return Err(CommonError::CommonError(format!(
            "shard {} data not found",
            shard_name
        )));
    }
}

#[async_trait]
impl StorageAdapter for MemoryStorageAdapter {
    async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
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

    async fn list_shard(&self, shard: &str) -> Result<Vec<ShardInfo>, CommonError> {
        if shard.is_empty() {
            return Ok(self
                .shard_info
                .iter()
                .map(|entry| entry.value().clone())
                .collect());
        }

        Ok(self
            .shard_info
            .get(shard)
            .map(|info| vec![info.clone()])
            .unwrap_or_default())
    }

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError> {
        self.shard_data.remove(shard);
        self.shard_info.remove(shard);
        self.shard_state.remove(shard);
        self.remove_indexes(shard);
        self.shard_write_locks.remove(shard);

        for mut group_entry in self.group_data.iter_mut() {
            group_entry.value_mut().remove(shard);
        }

        Ok(())
    }

    async fn batch_write(&self, shard: &str, messages: &[Record]) -> Result<Vec<u64>, CommonError> {
        self.internal_batch_write(shard, messages).await
    }

    async fn write(&self, shard: &str, data: &Record) -> Result<u64, CommonError> {
        let offsets = self
            .internal_batch_write(shard, std::slice::from_ref(data))
            .await?;
        Ok(offsets[0])
    }

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        todo!()
    }

    async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        todo!()
    }

    async fn read_by_key(&self, shard: &str, key: &str) -> Result<Vec<Record>, CommonError> {
        todo!()
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        todo!()
    }

    async fn get_offset_by_group(&self, group_name: &str) -> Result<Vec<ShardOffset>, CommonError> {
        todo!()
    }

    async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        todo!()
    }

    async fn message_expire(&self, _config: &MessageExpireConfig) -> Result<(), CommonError> {
        todo!()
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}
