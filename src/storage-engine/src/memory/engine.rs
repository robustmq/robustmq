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

use common_config::storage::memory::StorageDriverMemoryConfig;
use dashmap::DashMap;
use metadata_struct::storage::adapter_offset::{
    AdapterConsumerGroupOffset, AdapterOffsetStrategy, AdapterShardInfo,
};
use metadata_struct::storage::adapter_read_config::{AdapterReadConfig, AdapterWriteRespRow};
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::convert::convert_adapter_record_to_engine;
use metadata_struct::storage::storage_record::StorageRecord;
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::error::StorageEngineError;

#[derive(Clone, Debug, Default)]
pub struct ShardState {
    pub earliest_offset: u64,
    pub latest_offset: u64,
}

#[derive(Clone)]
pub struct MemoryStorageEngine {
    //(shard, (ShardInfo))
    pub shard_info: DashMap<String, AdapterShardInfo>,
    //(shard, (offset,Record))
    pub shard_data: DashMap<String, DashMap<u64, StorageRecord>>,
    //(group, (shard, offset))
    pub group_data: DashMap<String, DashMap<String, u64>>,
    //(shard, (tag, (offset)))
    pub tag_index: DashMap<String, DashMap<String, Vec<u64>>>,
    //(shard, (key, offset))
    pub key_index: DashMap<String, DashMap<String, u64>>,
    //(shard, (timestamp, offset))
    pub timestamp_index: DashMap<String, DashMap<u64, u64>>,
    //(shard, ShardState)
    pub shard_state: DashMap<String, ShardState>,
    //(shard, lock)
    pub shard_write_locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
    pub config: StorageDriverMemoryConfig,
}

impl Default for MemoryStorageEngine {
    fn default() -> Self {
        Self::new(StorageDriverMemoryConfig::default())
    }
}

impl MemoryStorageEngine {
    pub fn new(config: StorageDriverMemoryConfig) -> Self {
        MemoryStorageEngine {
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

    fn search_index_by_timestamp(&self, shard: &str, timestamp: u64) -> Option<u64> {
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

    async fn internal_batch_write(
        &self,
        shard_name: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let lock = self
            .shard_write_locks
            .entry(shard_name.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
            .clone();

        let _guard = lock.lock().await;
        let shard_state = self
            .shard_state
            .entry(shard_name.to_owned())
            .or_default()
            .clone();

        let mut offset_res = Vec::with_capacity(messages.len());
        let mut offset = shard_state.latest_offset;
        let shard_name_str = shard_name.to_string();

        if !self.shard_data.contains_key(shard_name) {
            self.shard_data
                .insert(shard_name.to_string(), DashMap::with_capacity(2));
        }

        if let Some(data_map) = self.shard_data.get(shard_name) {
            for msg in messages.iter() {
                offset_res.push(AdapterWriteRespRow {
                    pkid: msg.pkid,
                    offset,
                    ..Default::default()
                });

                // Convert StorageAdapterRecord to StorageEngineRecord
                let engine_record =
                    convert_adapter_record_to_engine(msg.clone(), shard_name, offset);

                // save data
                data_map.insert(offset, engine_record);

                // key index
                if let Some(key) = &msg.key {
                    let key_map = self.key_index.entry(shard_name_str.clone()).or_default();
                    key_map.insert(key.clone(), offset);
                }

                // tag index
                if let Some(tags) = &msg.tags {
                    let tag_map = self.tag_index.entry(shard_name_str.clone()).or_default();
                    for tag in tags.iter() {
                        tag_map.entry(tag.clone()).or_default().push(offset);
                    }
                }

                // timestamp index
                if msg.timestamp > 0 && offset.is_multiple_of(5000) {
                    let timestamp_map = self
                        .timestamp_index
                        .entry(shard_name_str.clone())
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
                    earliest_offset: shard_state.earliest_offset,
                    latest_offset: offset,
                },
            );
            return Ok(offset_res);
        }

        Err(StorageEngineError::CommonErrorStr(format!(
            "shard {} data not found",
            shard_name
        )))
    }
}

impl MemoryStorageEngine {
    pub async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), StorageEngineError> {
        if self.shard_info.contains_key(&shard.shard_name) {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "shard {} data already exist",
                shard.shard_name
            )));
        }

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

    pub async fn batch_write(
        &self,
        shard: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        self.internal_batch_write(shard, messages).await
    }

    pub async fn write(
        &self,
        shard: &str,
        data: &AdapterWriteRecord,
    ) -> Result<AdapterWriteRespRow, StorageEngineError> {
        let results = self
            .internal_batch_write(shard, std::slice::from_ref(data))
            .await?;

        if results.is_empty() {
            return Err(StorageEngineError::CommonErrorStr("".to_string()));
        }

        Ok(results.first().unwrap().clone())
    }

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

    pub async fn get_offset_by_group(
        &self,
        group_name: &str,
        _strategy: AdapterOffsetStrategy,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, StorageEngineError> {
        let Some(group_map) = self.group_data.get(group_name) else {
            return Ok(Vec::new());
        };

        let offsets = group_map
            .iter()
            .map(|entry| AdapterConsumerGroupOffset {
                group: group_name.to_string(),
                shard_name: entry.key().clone(),
                offset: *entry.value(),
                ..Default::default()
            })
            .collect();

        Ok(offsets)
    }

    pub async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), StorageEngineError> {
        if offset.is_empty() {
            return Ok(());
        }

        let group_map = self
            .group_data
            .entry(group_name.to_string())
            .or_insert_with(|| DashMap::with_capacity(offset.len()));

        for (shard_name, offset_val) in offset.iter() {
            group_map.insert(shard_name.clone(), *offset_val);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {}
