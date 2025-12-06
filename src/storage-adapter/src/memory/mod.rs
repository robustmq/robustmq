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
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct ShardState {
    pub start_offset: u64,
    pub next_offset: u64,
}

#[derive(Clone)]
pub struct MemoryStorageAdapter {
    config: StorageDriverMemoryConfig,
    pub shard_info: DashMap<String, ShardInfo>,
    pub shard_data: DashMap<String, Vec<Arc<Record>>>,
    pub group_data: DashMap<String, DashMap<String, u64>>,
    pub tag_index: DashMap<String, HashMap<String, Vec<u64>>>,
    pub key_index: DashMap<String, HashMap<String, Vec<u64>>>,
    pub timestamp_index: DashMap<String, BTreeMap<u64, Vec<u64>>>,
    pub shard_state: DashMap<String, ShardState>,
}

impl Default for MemoryStorageAdapter {
    fn default() -> Self {
        Self::new(StorageDriverMemoryConfig::default())
    }
}

impl MemoryStorageAdapter {
    pub fn new(config: StorageDriverMemoryConfig) -> Self {
        MemoryStorageAdapter {
            shard_info: DashMap::with_capacity(config.initial_shard_capacity),
            shard_data: DashMap::with_capacity(config.initial_shard_capacity),
            shard_state: DashMap::with_capacity(config.initial_shard_capacity),
            tag_index: DashMap::with_capacity(config.initial_shard_capacity),
            key_index: DashMap::with_capacity(config.initial_shard_capacity),
            timestamp_index: DashMap::with_capacity(config.initial_shard_capacity),
            group_data: DashMap::with_capacity(config.initial_group_capacity),
            config,
        }
    }

    #[inline]
    fn get_start_offset(&self, shard_key: &str) -> u64 {
        self.shard_state
            .get(shard_key)
            .map(|s| s.start_offset)
            .unwrap_or(0)
    }

    fn update_indexes(&self, shard_key: &str, record: &Record, offset: u64) {
        if self.config.enable_tag_index {
            if let Some(tags) = &record.tags {
                let mut tag_map = self.tag_index.entry(shard_key.to_string()).or_default();
                for tag in tags {
                    tag_map.entry(tag.clone()).or_default().push(offset);
                }
            }
        }

        if self.config.enable_key_index {
            if let Some(key) = &record.key {
                let mut key_map = self.key_index.entry(shard_key.to_string()).or_default();
                key_map.entry(key.clone()).or_default().push(offset);
            }
        }

        if self.config.enable_timestamp_index && record.timestamp > 0 {
            let mut ts_map = self
                .timestamp_index
                .entry(shard_key.to_string())
                .or_default();
            ts_map.entry(record.timestamp).or_default().push(offset);
        }
    }

    fn remove_offset_from_indexes(&self, shard_key: &str, record: &Record, offset: u64) {
        if self.config.enable_tag_index {
            if let Some(tags) = &record.tags {
                if let Some(mut tag_map) = self.tag_index.get_mut(shard_key) {
                    for tag in tags {
                        if let Some(offsets) = tag_map.get_mut(tag) {
                            offsets.retain(|&o| o != offset);
                            if offsets.is_empty() {
                                tag_map.remove(tag);
                            }
                        }
                    }
                }
            }
        }

        if self.config.enable_key_index {
            if let Some(key) = &record.key {
                if let Some(mut key_map) = self.key_index.get_mut(shard_key) {
                    if let Some(offsets) = key_map.get_mut(key) {
                        offsets.retain(|&o| o != offset);
                        if offsets.is_empty() {
                            key_map.remove(key);
                        }
                    }
                }
            }
        }

        if self.config.enable_timestamp_index && record.timestamp > 0 {
            if let Some(mut ts_map) = self.timestamp_index.get_mut(shard_key) {
                if let Some(offsets) = ts_map.get_mut(&record.timestamp) {
                    offsets.retain(|&o| o != offset);
                    if offsets.is_empty() {
                        ts_map.remove(&record.timestamp);
                    }
                }
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
        shard_name: &str,
        messages: &[Record],
    ) -> Result<Vec<u64>, CommonError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        // Ensure shard_data exists atomically
        self.shard_data
            .entry(shard_name.to_string())
            .or_insert_with(|| Vec::with_capacity(self.config.max_records_per_shard));

        self.shard_state.entry(shard_name.to_string()).or_default();

        // Now we can safely get mutable reference
        let mut data_list = self.shard_data.get_mut(shard_name).unwrap();

        // Reserve offset range atomically before inserting data
        let start_offset = {
            let mut state = self.shard_state.get_mut(shard_name).unwrap();
            let start = state.next_offset;
            state.next_offset = start + messages.len() as u64;
            start
        };

        let offsets =
            self.process_and_insert_messages(shard_name, messages, start_offset, &mut data_list);

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
        self.shard_data.insert(
            shard.shard_name.clone(),
            Vec::with_capacity(self.config.max_records_per_shard),
        );
        self.shard_info
            .insert(shard.shard_name.clone(), shard.clone());
        self.shard_state
            .insert(shard.shard_name.clone(), ShardState::default());
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
        let Some(data_list) = self.shard_data.get(shard) else {
            return Ok(Vec::new());
        };

        if data_list.is_empty() {
            return Ok(Vec::new());
        }

        let start_offset = self.get_start_offset(shard);

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
        shard: &str,
        offset: u64,
        tag: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        if let Some(tag_map) = self.tag_index.get(shard) {
            if let Some(offsets) = tag_map.get(tag) {
                if let Some(data_list) = self.shard_data.get(shard) {
                    return Ok(self.read_by_index(shard, offset, offsets, &data_list, read_config));
                }
            }
        }

        if let Some(data_list) = self.shard_data.get(shard) {
            return Ok(
                self.linear_scan(shard, offset, &data_list, read_config, |record| {
                    record
                        .tags
                        .as_ref()
                        .is_some_and(|tags| tags.contains(&tag.to_string()))
                }),
            );
        }

        Ok(Vec::new())
    }

    async fn read_by_key(
        &self,
        shard: &str,
        offset: u64,
        key: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        if let Some(key_map) = self.key_index.get(shard) {
            if let Some(offsets) = key_map.get(key) {
                if let Some(data_list) = self.shard_data.get(shard) {
                    return Ok(self.read_by_index(shard, offset, offsets, &data_list, read_config));
                }
            }
        }

        if let Some(data_list) = self.shard_data.get(shard) {
            return Ok(
                self.linear_scan(shard, offset, &data_list, read_config, |record| {
                    record.key.as_deref() == Some(key)
                }),
            );
        }

        Ok(Vec::new())
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        let start_offset = self.get_start_offset(shard);

        if let Some(ts_map) = self.timestamp_index.get(shard) {
            for (_ts, offsets) in ts_map.range(timestamp..) {
                // Find the first offset in the list that is >= start_offset
                if let Some(&offset) = offsets.iter().find(|&&o| o >= start_offset) {
                    return Ok(Some(ShardOffset {
                        shard_name: shard.to_string(),
                        offset,
                        ..Default::default()
                    }));
                }
            }
        }

        if let Some(record_list) = self.shard_data.get(shard) {
            for record in record_list.iter() {
                if record.timestamp >= timestamp {
                    if let Some(offset) = record.offset {
                        return Ok(Some(ShardOffset {
                            shard_name: shard.to_string(),
                            offset,
                            ..Default::default()
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
                        shard_name: entry.key().clone(),
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
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let data = self
            .group_data
            .entry(group_name.to_string())
            .or_insert_with(|| DashMap::with_capacity(offset.len()));

        for (shard_name, offset_val) in offset.iter() {
            data.insert(shard_name.to_string(), *offset_val);
        }

        Ok(())
    }

    async fn message_expire(&self, _config: &MessageExpireConfig) -> Result<(), CommonError> {
        let shard_keys: Vec<String> = self
            .shard_data
            .iter()
            .filter(|entry| entry.value().len() > self.config.max_records_per_shard)
            .map(|entry| entry.key().clone())
            .collect();

        for shard_key in shard_keys {
            if let Some(mut data_list) = self.shard_data.get_mut(&shard_key) {
                if data_list.len() <= self.config.max_records_per_shard {
                    continue;
                }

                let to_evict = data_list.len() - self.config.max_records_per_shard;

                for record in data_list.iter().take(to_evict) {
                    if let Some(offset) = record.offset {
                        self.remove_offset_from_indexes(&shard_key, record, offset);
                    }
                }

                // Update start_offset BEFORE draining to maintain consistency
                // This ensures any concurrent reads will see the correct offset range
                if let Some(mut state) = self.shard_state.get_mut(&shard_key) {
                    state.start_offset += to_evict as u64;
                }

                data_list.drain(0..to_evict);
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
    use std::sync::Arc;

    fn msg(data: &[u8]) -> Record {
        Record::from_bytes(data.to_vec())
    }

    fn msg_meta(data: &[u8], tags: Vec<&str>, key: &str, ts: u64) -> Record {
        let mut r = msg(data);
        r.tags = Some(tags.into_iter().map(|s| s.to_string()).collect());
        r.key = Some(key.to_string());
        r.timestamp = ts;
        r
    }

    #[tokio::test]
    async fn test_core_write_read() {
        let a = MemoryStorageAdapter::default();
        let cfg = ReadConfig {
            max_record_num: 10,
            max_size: 1024,
        };

        // Write & read basic
        let offsets = a
            .batch_write("s1", &[msg(b"m1"), msg(b"m2")])
            .await
            .unwrap();
        assert_eq!(offsets, vec![0, 1]);

        let recs = a.read_by_offset("s1", 0, &cfg).await.unwrap();
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].data.as_ref(), b"m1");

        // Tag/key indexes
        a.batch_write("s1", &[msg_meta(b"m3", vec!["t1"], "k1", 0)])
            .await
            .unwrap();
        assert_eq!(a.read_by_tag("s1", 0, "t1", &cfg).await.unwrap().len(), 1);
        assert_eq!(
            a.read_by_key("s1", 0, "k1", &cfg).await.unwrap()[0]
                .data
                .as_ref(),
            b"m3"
        );

        // Duplicate timestamps
        a.batch_write(
            "s2",
            &[
                msg_meta(b"a", vec![], "", 100),
                msg_meta(b"b", vec![], "", 100),
            ],
        )
        .await
        .unwrap();
        let ts_idx = a.timestamp_index.get("s2").unwrap();
        assert_eq!(ts_idx.get(&100).unwrap(), &vec![0, 1]);
        assert_eq!(
            a.get_offset_by_timestamp("s2", 100)
                .await
                .unwrap()
                .unwrap()
                .offset,
            0
        );
    }

    #[tokio::test]
    async fn test_group_offset_with_shard_name() {
        let a = MemoryStorageAdapter::default();

        a.batch_write("s1", &[msg(b"x")]).await.unwrap();
        a.batch_write("s2", &[msg(b"y")]).await.unwrap();

        // Commit & verify shard_name is preserved
        let offs = HashMap::from([("s1".to_string(), 10u64), ("s2".to_string(), 20u64)]);
        a.commit_offset("g1", &offs).await.unwrap();

        let result = a.get_offset_by_group("g1").await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(
            result.iter().find(|o| o.shard_name == "s1").unwrap().offset,
            10
        );
        assert_eq!(
            result.iter().find(|o| o.shard_name == "s2").unwrap().offset,
            20
        );
    }

    #[tokio::test]
    async fn test_expire_consistency() {
        let cfg = StorageDriverMemoryConfig {
            max_records_per_shard: 2,
            ..Default::default()
        };
        let a = MemoryStorageAdapter::new(cfg);
        let rd = ReadConfig {
            max_record_num: 10,
            max_size: 1024,
        };

        // Write 4, expire to 2
        for i in 0..4 {
            a.write(
                "s",
                &msg_meta(
                    format!("{}", i).as_bytes(),
                    vec![&format!("t{}", i)],
                    "",
                    1000 + i,
                ),
            )
            .await
            .unwrap();
        }
        a.message_expire(&MessageExpireConfig::default())
            .await
            .unwrap();

        // Verify state & data
        let state = a.shard_state.get("s").unwrap();
        assert_eq!(state.start_offset, 2);
        assert_eq!(state.next_offset, 4);
        assert_eq!(a.shard_data.get("s").unwrap().len(), 2);

        // Evicted inaccessible, remaining accessible
        assert_eq!(a.read_by_offset("s", 0, &rd).await.unwrap().len(), 0);
        assert_eq!(a.read_by_tag("s", 0, "t0", &rd).await.unwrap().len(), 0);
        assert_eq!(a.read_by_offset("s", 2, &rd).await.unwrap().len(), 2);
        assert_eq!(a.read_by_tag("s", 0, "t2", &rd).await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_concurrent_writes() {
        let a = Arc::new(MemoryStorageAdapter::default());
        let mut handles = vec![];

        // 10 threads, each write 5 messages
        for t in 0..10 {
            let a_clone = a.clone();
            let h = tokio::spawn(async move {
                let mut offs = vec![];
                for i in 0..5 {
                    let data = format!("t{}m{}", t, i);
                    let o = a_clone
                        .write("concurrent", &msg(data.as_bytes()))
                        .await
                        .unwrap();
                    offs.push(o);
                }
                offs
            });
            handles.push(h);
        }

        // Collect all offsets
        let mut all_offsets = vec![];
        for h in handles {
            all_offsets.extend(h.await.unwrap());
        }

        // Verify: 50 unique offsets [0..49]
        all_offsets.sort();
        assert_eq!(all_offsets.len(), 50);
        assert_eq!(all_offsets, (0..50).collect::<Vec<u64>>());

        // Verify final state
        let state = a.shard_state.get("concurrent").unwrap();
        assert_eq!(state.next_offset, 50);
        assert_eq!(a.shard_data.get("concurrent").unwrap().len(), 50);
    }

    #[tokio::test]
    async fn test_shard_lifecycle() {
        let a = MemoryStorageAdapter::default();

        // create_shard
        let s1 = ShardInfo {
            shard_name: "s1".into(),
            replica_num: 3,
            ..Default::default()
        };
        let s2 = ShardInfo {
            shard_name: "s2".into(),
            replica_num: 1,
            ..Default::default()
        };
        a.create_shard(&s1).await.unwrap();
        a.create_shard(&s2).await.unwrap();

        // list_shard: all
        let all = a.list_shard("").await.unwrap();
        assert_eq!(all.len(), 2);

        // list_shard: specific
        let one = a.list_shard("s1").await.unwrap();
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].shard_name, "s1");
        assert_eq!(one[0].replica_num, 3);

        // delete_shard
        a.delete_shard("s1").await.unwrap();
        assert_eq!(a.list_shard("").await.unwrap().len(), 1);
        assert_eq!(a.list_shard("s1").await.unwrap().len(), 0);

        // close (no-op, just verify it doesn't panic)
        a.close().await.unwrap();
    }
}
