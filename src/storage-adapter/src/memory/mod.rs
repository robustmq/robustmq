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
        let end = shard_state.next_offset;

        for offset in start..end {
            let Some(record) = data_map.get(&offset) else {
                continue;
            };

            if record.timestamp >= timestamp {
                return Some(offset);
            }
        }

        None
    }

    async fn internal_batch_write(
        &self,
        shard_name: &str,
        messages: &[Record],
    ) -> Result<Vec<u64>, CommonError> {
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
        let mut offset = shard_state.next_offset;
        let shard_name_str = shard_name.to_string();

        if !self.shard_data.contains_key(shard_name) {
            self.shard_data
                .insert(shard_name.to_string(), DashMap::with_capacity(2));
        }

        if let Some(data_map) = self.shard_data.get(shard_name) {
            for msg in messages.iter() {
                offset_res.push(offset);

                // save data
                let mut record_to_save = msg.clone();
                record_to_save.offset = Some(offset);
                data_map.insert(offset, record_to_save);

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
                    start_offset: shard_state.start_offset,
                    next_offset: offset,
                },
            );
            return Ok(offset_res);
        }

        Err(CommonError::CommonError(format!(
            "shard {} data not found",
            shard_name
        )))
    }
}

#[async_trait]
impl StorageAdapter for MemoryStorageAdapter {
    async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
        if self.shard_info.contains_key(&shard.shard_name) {
            return Err(CommonError::CommonError(format!(
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

    async fn delete_shard(&self, shard_name: &str) -> Result<(), CommonError> {
        if !self.shard_info.contains_key(shard_name) {
            return Err(CommonError::CommonError(format!(
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

    async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
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

    async fn read_by_key(&self, shard: &str, key: &str) -> Result<Vec<Record>, CommonError> {
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

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        let index_offset = self.search_index_by_timestamp(shard, timestamp);

        if let Some(offset) = self.read_data_by_time(shard, index_offset, timestamp) {
            return Ok(Some(ShardOffset {
                shard_name: shard.to_string(),
                offset,
                ..Default::default()
            }));
        }

        Ok(None)
    }

    async fn get_offset_by_group(&self, group_name: &str) -> Result<Vec<ShardOffset>, CommonError> {
        let Some(group_map) = self.group_data.get(group_name) else {
            return Ok(Vec::new());
        };

        let offsets = group_map
            .iter()
            .map(|entry| ShardOffset {
                group: group_name.to_string(),
                shard_name: entry.key().clone(),
                offset: *entry.value(),
                ..Default::default()
            })
            .collect();

        Ok(offsets)
    }

    async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
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

    async fn message_expire(&self, _config: &MessageExpireConfig) -> Result<(), CommonError> {
        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_shard_lifecycle() {
        let adapter = MemoryStorageAdapter::default();

        let shard1 = ShardInfo {
            shard_name: "shard1".to_string(),
            replica_num: 3,
            ..Default::default()
        };
        adapter.create_shard(&shard1).await.unwrap();
        adapter
            .create_shard(&ShardInfo {
                shard_name: "shard2".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        assert_eq!(adapter.list_shard("").await.unwrap().len(), 2);
        assert_eq!(
            adapter.list_shard("shard1").await.unwrap()[0].replica_num,
            3
        );

        adapter.delete_shard("shard1").await.unwrap();
        assert_eq!(adapter.list_shard("").await.unwrap().len(), 1);
        assert_eq!(adapter.list_shard("shard1").await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_write_and_read() {
        let adapter = MemoryStorageAdapter::default();
        let cfg = ReadConfig {
            max_record_num: 10,
            max_size: 1024 * 1024,
        };

        adapter
            .create_shard(&ShardInfo {
                shard_name: "s".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        let mut r1 = Record::from_bytes(b"msg1".to_vec());
        r1.key = Some("k1".to_string());
        r1.tags = Some(vec!["a".to_string(), "c".to_string()]);
        r1.timestamp = 1000;

        let mut r2 = Record::from_bytes(b"msg2".to_vec());
        r2.key = Some("k2".to_string());
        r2.tags = Some(vec!["b".to_string(), "c".to_string()]);
        r2.timestamp = 2000;

        assert_eq!(
            adapter.batch_write("s", &[r1, r2]).await.unwrap(),
            vec![0, 1]
        );
        assert_eq!(adapter.read_by_offset("s", 0, &cfg).await.unwrap().len(), 2);
        assert_eq!(adapter.read_by_offset("s", 1, &cfg).await.unwrap().len(), 1);

        assert_eq!(
            adapter
                .read_by_tag("s", "a", None, &cfg)
                .await
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            adapter
                .read_by_tag("s", "c", None, &cfg)
                .await
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            adapter
                .read_by_tag("s", "b", Some(1), &cfg)
                .await
                .unwrap()
                .len(),
            1
        );

        assert_eq!(
            adapter.read_by_key("s", "k2").await.unwrap()[0].data,
            b"msg2".to_vec()
        );
        assert_eq!(adapter.read_by_key("s", "k3").await.unwrap().len(), 0);

        assert_eq!(
            adapter
                .get_offset_by_timestamp("s", 1500)
                .await
                .unwrap()
                .unwrap()
                .offset,
            1
        );
        assert!(adapter
            .get_offset_by_timestamp("s", 5000)
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn test_consumer_group_offset() {
        let adapter = MemoryStorageAdapter::default();

        adapter
            .create_shard(&ShardInfo {
                shard_name: "s1".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();
        adapter
            .create_shard(&ShardInfo {
                shard_name: "s2".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        adapter
            .commit_offset(
                "g1",
                &HashMap::from([("s1".into(), 100), ("s2".into(), 200)]),
            )
            .await
            .unwrap();

        let offsets = adapter.get_offset_by_group("g1").await.unwrap();
        assert_eq!(offsets.len(), 2);
        assert_eq!(
            offsets
                .iter()
                .find(|o| o.shard_name == "s1")
                .unwrap()
                .offset,
            100
        );
        assert_eq!(
            offsets
                .iter()
                .find(|o| o.shard_name == "s2")
                .unwrap()
                .offset,
            200
        );

        adapter
            .commit_offset("g1", &HashMap::from([("s1".into(), 150)]))
            .await
            .unwrap();
        let offsets = adapter.get_offset_by_group("g1").await.unwrap();
        assert_eq!(
            offsets
                .iter()
                .find(|o| o.shard_name == "s1")
                .unwrap()
                .offset,
            150
        );

        adapter
            .commit_offset("g2", &HashMap::from([("s1".into(), 300)]))
            .await
            .unwrap();
        assert_eq!(adapter.get_offset_by_group("g2").await.unwrap().len(), 1);

        assert_eq!(adapter.get_offset_by_group("g3").await.unwrap().len(), 0);

        assert!(adapter
            .commit_offset("g1", &HashMap::from([("s3".into(), 100)]))
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn test_timestamp_index_with_multiple_entries() {
        let adapter = MemoryStorageAdapter::default();
        let cfg = ReadConfig {
            max_record_num: 100,
            max_size: 1024 * 1024,
        };

        adapter
            .create_shard(&ShardInfo {
                shard_name: "s".to_string(),
                ..Default::default()
            })
            .await
            .unwrap();

        let mut records = Vec::new();
        for i in 0..15000 {
            let mut r = Record::from_bytes(format!("msg{}", i).into_bytes());
            r.timestamp = 1000 + i;
            records.push(r);
        }

        let offsets = adapter.batch_write("s", &records).await.unwrap();
        assert_eq!(offsets.len(), 15000);
        assert_eq!(offsets[0], 0);
        assert_eq!(offsets[14999], 14999);

        let result = adapter.get_offset_by_timestamp("s", 1000).await.unwrap();
        assert_eq!(result.unwrap().offset, 0);

        let result = adapter.get_offset_by_timestamp("s", 3500).await.unwrap();
        assert_eq!(result.unwrap().offset, 2500);

        let result = adapter.get_offset_by_timestamp("s", 6000).await.unwrap();
        assert_eq!(result.unwrap().offset, 5000);

        let result = adapter.get_offset_by_timestamp("s", 8000).await.unwrap();
        assert_eq!(result.unwrap().offset, 7000);

        let result = adapter.get_offset_by_timestamp("s", 11000).await.unwrap();
        assert_eq!(result.unwrap().offset, 10000);

        let result = adapter.get_offset_by_timestamp("s", 14500).await.unwrap();
        assert_eq!(result.unwrap().offset, 13500);

        let result = adapter.get_offset_by_timestamp("s", 500).await.unwrap();
        assert_eq!(result.unwrap().offset, 0);

        let result = adapter.get_offset_by_timestamp("s", 20000).await.unwrap();
        assert!(result.is_none());

        let read_result = adapter.read_by_offset("s", 5000, &cfg).await.unwrap();
        assert!(!read_result.is_empty());
        assert_eq!(read_result[0].timestamp, 6000);
    }
}
