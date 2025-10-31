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

use crate::storage::{ShardInfo, ShardOffset, StorageAdapter};
use axum::async_trait;
use common_base::error::common::CommonError;
use dashmap::DashMap;
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

const MAX_SHARD_SIZE: usize = 10_000_000;
const DEFAULT_LRU_CACHE_SIZE: usize = 100;
const DEFAULT_SHARD_CAPACITY: usize = 16;
const DEFAULT_GROUP_CAPACITY: usize = 8;

#[derive(Clone, Debug, Default)]
pub struct ShardState {
    pub start_offset: u64,
    pub next_offset: u64,
}

#[derive(Clone)]
pub struct MemoryStorageAdapter {
    max_shard_size: usize,
    pub shard_info: DashMap<String, ShardInfo>,
    pub shard_data: DashMap<String, Vec<Arc<Record>>>,
    pub group_data: DashMap<String, DashMap<String, u64>>,
    pub tag_index: DashMap<String, HashMap<String, Vec<u64>>>,
    pub key_index: DashMap<String, HashMap<String, Vec<u64>>>,
    pub timestamp_index: DashMap<String, BTreeMap<u64, u64>>,
    pub shard_state: DashMap<String, ShardState>,
}

impl Default for MemoryStorageAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStorageAdapter {
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_LRU_CACHE_SIZE)
    }

    pub fn with_capacity(max_shard_size: usize) -> Self {
        Self::with_shard_capacity(max_shard_size, DEFAULT_SHARD_CAPACITY)
    }

    pub fn with_shard_capacity(max_shard_size: usize, shard_capacity: usize) -> Self {
        let effective_size = if max_shard_size == 0 {
            DEFAULT_LRU_CACHE_SIZE
        } else {
            max_shard_size.min(MAX_SHARD_SIZE)
        };

        let effective_shard_capacity = if shard_capacity == 0 {
            DEFAULT_SHARD_CAPACITY
        } else {
            shard_capacity
        };

        MemoryStorageAdapter {
            max_shard_size: effective_size,
            shard_info: DashMap::with_capacity(effective_shard_capacity),
            shard_data: DashMap::with_capacity(effective_shard_capacity),
            shard_state: DashMap::with_capacity(effective_shard_capacity),
            tag_index: DashMap::with_capacity(effective_shard_capacity),
            key_index: DashMap::with_capacity(effective_shard_capacity),
            timestamp_index: DashMap::with_capacity(effective_shard_capacity),
            group_data: DashMap::with_capacity(DEFAULT_GROUP_CAPACITY),
        }
    }

    #[inline]
    pub fn shard_key(&self, namespace: &str, shard_name: &str) -> String {
        let mut key = String::with_capacity(namespace.len() + shard_name.len() + 1);
        key.push_str(namespace);
        key.push('_');
        key.push_str(shard_name);
        key
    }

    #[inline]
    fn get_start_offset(&self, shard_key: &str) -> u64 {
        self.shard_state
            .get(shard_key)
            .map(|s| s.start_offset)
            .unwrap_or(0)
    }

    fn update_indexes(&self, shard_key: &str, record: &Record, offset: u64) {
        if !record.tags.is_empty() {
            let mut tag_map = self.tag_index.entry(shard_key.to_string()).or_default();
            for tag in &record.tags {
                tag_map.entry(tag.clone()).or_default().push(offset);
            }
        }

        if !record.key.is_empty() {
            let mut key_map = self.key_index.entry(shard_key.to_string()).or_default();
            key_map.entry(record.key.clone()).or_default().push(offset);
        }

        if record.timestamp > 0 {
            let mut ts_map = self
                .timestamp_index
                .entry(shard_key.to_string())
                .or_default();
            ts_map.insert(record.timestamp, offset);
        }
    }

    fn remove_offset_from_indexes(&self, shard_key: &str, record: &Record, offset: u64) {
        if !record.tags.is_empty() {
            if let Some(mut tag_map) = self.tag_index.get_mut(shard_key) {
                for tag in &record.tags {
                    if let Some(offsets) = tag_map.get_mut(tag) {
                        offsets.retain(|&o| o != offset);
                        if offsets.is_empty() {
                            tag_map.remove(tag);
                        }
                    }
                }
            }
        }

        if !record.key.is_empty() {
            if let Some(mut key_map) = self.key_index.get_mut(shard_key) {
                if let Some(offsets) = key_map.get_mut(&record.key) {
                    offsets.retain(|&o| o != offset);
                    if offsets.is_empty() {
                        key_map.remove(&record.key);
                    }
                }
            }
        }

        if record.timestamp > 0 {
            if let Some(mut ts_map) = self.timestamp_index.get_mut(shard_key) {
                ts_map.remove(&record.timestamp);
            }
        }
    }

    fn remove_indexes(&self, shard_key: &str) {
        self.tag_index.remove(shard_key);
        self.key_index.remove(shard_key);
        self.timestamp_index.remove(shard_key);
    }

    fn process_and_insert_messages(
        &self,
        shard_key: &str,
        messages: &[Record],
        start_offset: u64,
        data_list: &mut Vec<Arc<Record>>,
    ) -> Vec<u64> {
        let mut offsets = Vec::with_capacity(messages.len());
        data_list.reserve(messages.len());

        for (idx, msg) in messages.iter().enumerate() {
            let offset = start_offset + idx as u64;
            offsets.push(offset);

            let mut msg_owned = msg.clone();
            msg_owned.offset = Some(offset);
            let record_arc = Arc::new(msg_owned);

            self.update_indexes(shard_key, &record_arc, offset);
            data_list.push(record_arc);
        }

        offsets
    }

    async fn internal_batch_write(
        &self,
        namespace: &str,
        shard_name: &str,
        messages: &[Record],
    ) -> Result<Vec<u64>, CommonError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        let shard_key = self.shard_key(namespace, shard_name);
        self.shard_state.entry(shard_key.clone()).or_default();

        let offsets = if let Some(mut data_list) = self.shard_data.get_mut(&shard_key) {
            let start_offset = self.shard_state.get(&shard_key).unwrap().next_offset;
            let offsets = self.process_and_insert_messages(
                &shard_key,
                messages,
                start_offset,
                &mut data_list,
            );

            drop(data_list);

            self.shard_state.get_mut(&shard_key).unwrap().next_offset =
                start_offset + messages.len() as u64;

            offsets
        } else {
            let mut data_list = Vec::with_capacity(messages.len().min(self.max_shard_size));
            let offsets = self.process_and_insert_messages(&shard_key, messages, 0, &mut data_list);

            self.shard_data.insert(shard_key.clone(), data_list);

            let mut state = self.shard_state.get_mut(&shard_key).unwrap();
            state.next_offset = messages.len() as u64;
            state.start_offset = 0;

            offsets
        };

        Ok(offsets)
    }

    fn read_by_index(
        &self,
        shard_key: &str,
        offset: u64,
        offsets: &[u64],
        data_list: &[Arc<Record>],
        read_config: &ReadConfig,
    ) -> Vec<Record> {
        let start_offset = self.get_start_offset(shard_key);
        let start_pos = offsets.binary_search(&offset).unwrap_or_else(|pos| pos);

        offsets
            .iter()
            .skip(start_pos)
            .filter(|&&idx| idx >= start_offset)
            .take(read_config.max_record_num as usize)
            .filter_map(|&idx| {
                let relative_idx = (idx - start_offset) as usize;
                data_list.get(relative_idx).map(|r| (**r).clone())
            })
            .collect()
    }

    fn linear_scan<F>(
        &self,
        shard_key: &str,
        offset: u64,
        data_list: &[Arc<Record>],
        read_config: &ReadConfig,
        predicate: F,
    ) -> Vec<Record>
    where
        F: Fn(&Record) -> bool,
    {
        let start_offset = self.get_start_offset(shard_key);
        let start_idx = if offset < start_offset {
            0
        } else {
            (offset - start_offset) as usize
        };

        if start_idx >= data_list.len() {
            return Vec::new();
        }

        let max_to_scan = (read_config.max_record_num as usize) * 10;
        let end_idx = (start_idx + max_to_scan).min(data_list.len());

        data_list[start_idx..end_idx]
            .iter()
            .filter(|record| predicate(record))
            .take(read_config.max_record_num as usize)
            .map(|record| (**record).clone())
            .collect()
    }
}

#[async_trait]
impl StorageAdapter for MemoryStorageAdapter {
    async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
        let key = self.shard_key(&shard.namespace, &shard.shard_name);
        self.shard_data
            .insert(key.clone(), Vec::with_capacity(self.max_shard_size));
        self.shard_info.insert(key.clone(), shard.clone());
        self.shard_state.insert(key, ShardState::default());
        Ok(())
    }

    async fn list_shard(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Result<Vec<ShardInfo>, CommonError> {
        if shard_name.is_empty() {
            return Ok(self
                .shard_info
                .iter()
                .map(|entry| entry.value().clone())
                .collect());
        }

        let key = self.shard_key(namespace, shard_name);
        Ok(self
            .shard_info
            .get(&key)
            .map(|info| vec![info.clone()])
            .unwrap_or_default())
    }

    async fn delete_shard(&self, namespace: &str, shard_name: &str) -> Result<(), CommonError> {
        let key = self.shard_key(namespace, shard_name);
        self.shard_data.remove(&key);
        self.shard_info.remove(&key);
        self.shard_state.remove(&key);
        self.remove_indexes(&key);
        Ok(())
    }

    async fn batch_write(
        &self,
        namespace: &str,
        shard_name: &str,
        messages: &[Record],
    ) -> Result<Vec<u64>, CommonError> {
        self.internal_batch_write(namespace, shard_name, messages)
            .await
    }

    async fn write(
        &self,
        namespace: &str,
        shard_name: &str,
        data: &Record,
    ) -> Result<u64, CommonError> {
        let offsets = self
            .internal_batch_write(namespace, shard_name, &[data.clone()])
            .await?;
        Ok(offsets[0])
    }

    async fn read_by_offset(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);

        let Some(data_list) = self.shard_data.get(&shard_key) else {
            return Ok(Vec::new());
        };

        if data_list.is_empty() {
            return Ok(Vec::new());
        }

        let start_offset = self.get_start_offset(&shard_key);

        if offset < start_offset {
            return Ok(Vec::new());
        }

        let relative_offset = (offset - start_offset) as usize;
        if relative_offset >= data_list.len() {
            return Ok(Vec::new());
        }

        let remaining = data_list.len() - relative_offset;
        let max_records = (read_config.max_record_num as usize).min(remaining);

        Ok((relative_offset..(relative_offset + max_records))
            .filter_map(|i| data_list.get(i))
            .map(|value| (**value).clone())
            .collect())
    }

    async fn read_by_tag(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        tag: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);

        if let Some(tag_map) = self.tag_index.get(&shard_key) {
            if let Some(offsets) = tag_map.get(tag) {
                if let Some(data_list) = self.shard_data.get(&shard_key) {
                    return Ok(self.read_by_index(
                        &shard_key,
                        offset,
                        offsets,
                        &data_list,
                        read_config,
                    ));
                }
            }
        }

        if let Some(data_list) = self.shard_data.get(&shard_key) {
            return Ok(
                self.linear_scan(&shard_key, offset, &data_list, read_config, |record| {
                    record.tags.contains(&tag.to_string())
                }),
            );
        }

        Ok(Vec::new())
    }

    async fn read_by_key(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        key: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);

        if let Some(key_map) = self.key_index.get(&shard_key) {
            if let Some(offsets) = key_map.get(key) {
                if let Some(data_list) = self.shard_data.get(&shard_key) {
                    return Ok(self.read_by_index(
                        &shard_key,
                        offset,
                        offsets,
                        &data_list,
                        read_config,
                    ));
                }
            }
        }

        if let Some(data_list) = self.shard_data.get(&shard_key) {
            return Ok(
                self.linear_scan(&shard_key, offset, &data_list, read_config, |record| {
                    record.key == key
                }),
            );
        }

        Ok(Vec::new())
    }

    async fn get_offset_by_timestamp(
        &self,
        namespace: &str,
        shard_name: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        let shard_key = self.shard_key(namespace, shard_name);
        let start_offset = self.get_start_offset(&shard_key);

        if let Some(ts_map) = self.timestamp_index.get(&shard_key) {
            for (_ts, &offset) in ts_map.range(timestamp..) {
                if offset >= start_offset {
                    return Ok(Some(ShardOffset {
                        namespace: namespace.to_string(),
                        shard_name: shard_name.to_string(),
                        segment_no: 0,
                        offset,
                    }));
                }
            }
        }

        if let Some(record_list) = self.shard_data.get(&shard_key) {
            for record in record_list.iter() {
                if record.timestamp >= timestamp {
                    if let Some(offset) = record.offset {
                        return Ok(Some(ShardOffset {
                            namespace: namespace.to_string(),
                            shard_name: shard_name.to_string(),
                            segment_no: 0,
                            offset,
                        }));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn get_offset_by_group(&self, group_name: &str) -> Result<Vec<ShardOffset>, CommonError> {
        Ok(self
            .group_data
            .get(group_name)
            .map(|data| {
                data.iter()
                    .map(|entry| ShardOffset {
                        offset: *entry.value(),
                        ..Default::default()
                    })
                    .collect()
            })
            .unwrap_or_default())
    }

    async fn commit_offset(
        &self,
        group_name: &str,
        namespace: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let data = self
            .group_data
            .entry(group_name.to_string())
            .or_insert_with(|| DashMap::with_capacity(offset.len()));

        for (shard_name, offset_val) in offset.iter() {
            let group_key = self.shard_key(namespace, shard_name);
            data.insert(group_key, *offset_val);
        }

        Ok(())
    }

    async fn message_expire(&self) -> Result<(), CommonError> {
        let shard_keys: Vec<String> = self
            .shard_data
            .iter()
            .filter(|entry| entry.value().len() > self.max_shard_size)
            .map(|entry| entry.key().clone())
            .collect();

        for shard_key in shard_keys {
            if let Some(mut data_list) = self.shard_data.get_mut(&shard_key) {
                if data_list.len() <= self.max_shard_size {
                    continue;
                }

                let to_evict = data_list.len() - self.max_shard_size;

                for record in data_list.iter().take(to_evict) {
                    if let Some(offset) = record.offset {
                        self.remove_offset_from_indexes(&shard_key, record, offset);
                    }
                }

                data_list.drain(0..to_evict);

                drop(data_list);

                if let Some(mut state) = self.shard_state.get_mut(&shard_key) {
                    state.start_offset += to_evict as u64;
                }
            }
        }

        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::tools::unique_id;

    const READ_CFG: ReadConfig = ReadConfig {
        max_record_num: 10,
        max_size: 1024 * 1024,
    };

    fn msg(data: &[u8]) -> Record {
        Record::build_byte(data.to_vec())
    }

    fn msg_with_meta(data: &[u8], tags: Vec<&str>, key: &str, ts: u64) -> Record {
        let mut msg = msg(data);
        msg.tags = tags.into_iter().map(|s| s.to_string()).collect();
        msg.key = key.to_string();
        msg.timestamp = ts;
        msg
    }

    #[tokio::test]
    async fn test_write_and_read() {
        let adapter = MemoryStorageAdapter::new();
        let ns = unique_id();

        assert_eq!(
            adapter
                .batch_write(&ns, "shard", &[msg(b"msg1"), msg(b"msg2")])
                .await
                .unwrap(),
            vec![0, 1]
        );
        let result = adapter
            .read_by_offset(&ns, "shard", 0, &READ_CFG)
            .await
            .unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].data, b"msg1");

        assert_eq!(
            adapter.batch_write(&ns, "shard", &[]).await.unwrap().len(),
            0
        );
        assert_eq!(
            adapter
                .read_by_offset(&ns, "none", 0, &READ_CFG)
                .await
                .unwrap()
                .len(),
            0
        );
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let adapter = MemoryStorageAdapter::with_capacity(3);
        let ns = unique_id();

        for i in 0..5 {
            adapter
                .write(
                    &ns,
                    "lru",
                    &msg_with_meta(
                        format!("msg{}", i).as_bytes(),
                        vec![&format!("tag-{}", i)],
                        "",
                        1000 + i * 100,
                    ),
                )
                .await
                .unwrap();
        }

        let key = adapter.shard_key(&ns, "lru");
        assert_eq!(adapter.shard_data.get(&key).unwrap().len(), 5);

        adapter.message_expire().await.unwrap();

        assert_eq!(adapter.shard_data.get(&key).unwrap().len(), 3);
        let state = adapter.shard_state.get(&key).unwrap();
        assert_eq!((state.start_offset, state.next_offset), (2, 5));

        assert_eq!(
            adapter
                .read_by_offset(&ns, "lru", 0, &READ_CFG)
                .await
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            adapter
                .read_by_offset(&ns, "lru", 2, &READ_CFG)
                .await
                .unwrap()
                .len(),
            3
        );
        assert_eq!(
            adapter
                .read_by_tag(&ns, "lru", 0, "tag-0", &READ_CFG)
                .await
                .unwrap()
                .len(),
            0
        );
        assert_eq!(
            adapter
                .read_by_tag(&ns, "lru", 0, "tag-3", &READ_CFG)
                .await
                .unwrap()
                .len(),
            1
        );
        assert!(
            adapter
                .get_offset_by_timestamp(&ns, "lru", 1150)
                .await
                .unwrap()
                .unwrap()
                .offset
                >= 2
        );
    }

    #[tokio::test]
    async fn test_indexed_queries() {
        let adapter = MemoryStorageAdapter::new();
        let ns = unique_id();

        adapter
            .write(
                &ns,
                "idx",
                &msg_with_meta(b"msg1", vec!["tag-a"], "key-1", 0),
            )
            .await
            .unwrap();
        adapter
            .write(
                &ns,
                "idx",
                &msg_with_meta(b"msg2", vec!["tag-b"], "key-2", 0),
            )
            .await
            .unwrap();

        assert_eq!(
            adapter
                .read_by_tag(&ns, "idx", 0, "tag-a", &READ_CFG)
                .await
                .unwrap()[0]
                .data,
            b"msg1"
        );
        assert_eq!(
            adapter
                .read_by_key(&ns, "idx", 0, "key-2", &READ_CFG)
                .await
                .unwrap()[0]
                .data,
            b"msg2"
        );
        assert_eq!(
            adapter
                .read_by_tag(&ns, "idx", 0, "none", &READ_CFG)
                .await
                .unwrap()
                .len(),
            0
        );

        adapter.tag_index.remove(&adapter.shard_key(&ns, "idx"));
        adapter.key_index.remove(&adapter.shard_key(&ns, "idx"));
        assert_eq!(
            adapter
                .read_by_tag(&ns, "idx", 0, "tag-a", &READ_CFG)
                .await
                .unwrap()
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn test_shard_management() {
        let adapter = MemoryStorageAdapter::new();
        let ns = unique_id();

        for name in ["s1", "s2"] {
            adapter
                .create_shard(&ShardInfo {
                    namespace: ns.clone(),
                    shard_name: name.to_string(),
                    replica_num: 3,
                })
                .await
                .unwrap();
        }

        assert_eq!(adapter.list_shard(&ns, "").await.unwrap().len(), 2);
        assert_eq!(
            adapter.list_shard(&ns, "s1").await.unwrap()[0].shard_name,
            "s1"
        );

        adapter.delete_shard(&ns, "s1").await.unwrap();

        assert_eq!(adapter.list_shard(&ns, "").await.unwrap().len(), 1);
        assert_eq!(adapter.list_shard(&ns, "s1").await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_consumer_group_offset() {
        let adapter = MemoryStorageAdapter::new();
        let ns = unique_id();

        adapter
            .batch_write(&ns, "grp", &[msg(b"m1"), msg(b"m2"), msg(b"m3")])
            .await
            .unwrap();

        let mut offsets = HashMap::from([("grp".to_string(), 1u64)]);
        adapter.commit_offset("cg1", &ns, &offsets).await.unwrap();
        assert_eq!(
            adapter.get_offset_by_group("cg1").await.unwrap()[0].offset,
            1
        );

        offsets.insert("grp".to_string(), 2);
        adapter.commit_offset("cg1", &ns, &offsets).await.unwrap();
        assert_eq!(
            adapter.get_offset_by_group("cg1").await.unwrap()[0].offset,
            2
        );
        assert_eq!(adapter.get_offset_by_group("none").await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn test_timestamp_query() {
        let adapter = MemoryStorageAdapter::new();
        let ns = unique_id();

        for (i, ts) in [1000u64, 2000, 3000].iter().enumerate() {
            adapter
                .write(
                    &ns,
                    "ts",
                    &msg_with_meta(format!("m{}", i).as_bytes(), vec![], "", *ts),
                )
                .await
                .unwrap();
        }

        assert_eq!(
            adapter
                .get_offset_by_timestamp(&ns, "ts", 1500)
                .await
                .unwrap()
                .unwrap()
                .offset,
            1
        );
        assert_eq!(
            adapter
                .get_offset_by_timestamp(&ns, "ts", 500)
                .await
                .unwrap()
                .unwrap()
                .offset,
            0
        );
        assert!(adapter
            .get_offset_by_timestamp(&ns, "ts", 4000)
            .await
            .unwrap()
            .is_none());
        assert!(adapter
            .get_offset_by_timestamp(&ns, "none", 1000)
            .await
            .unwrap()
            .is_none());
    }
}
