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
    pub timestamp_index: DashMap<String, BTreeMap<u64, u64>>,
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
            ts_map.insert(record.timestamp, offset);
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
            let mut data_list =
                Vec::with_capacity(messages.len().min(self.config.max_records_per_shard));
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
        self.shard_data.insert(
            key.clone(),
            Vec::with_capacity(self.config.max_records_per_shard),
        );
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
            .internal_batch_write(namespace, shard_name, std::slice::from_ref(data))
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
                    record.key.as_deref() == Some(key)
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
                        offset,
                        ..Default::default()
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

    // Helper: Default read configuration for tests
    fn read_config() -> ReadConfig {
        ReadConfig {
            max_record_num: 10,
            max_size: 1024 * 1024,
        }
    }

    // Helper: Create a simple message record
    fn create_message(data: &[u8]) -> Record {
        Record::from_bytes(data.to_vec())
    }

    // Helper: Create message with metadata (tags, key, timestamp)
    fn create_message_with_metadata(
        data: &[u8],
        tags: Vec<&str>,
        key: &str,
        timestamp: u64,
    ) -> Record {
        let mut record = create_message(data);
        record.tags = Some(tags.into_iter().map(|s| s.to_string()).collect());
        record.key = Some(key.to_string());
        record.timestamp = timestamp;
        record
    }

    #[tokio::test]
    async fn test_write_and_read() {
        // Setup
        let adapter = MemoryStorageAdapter::new(StorageDriverMemoryConfig::default());
        let namespace = unique_id();
        let shard_name = "test-shard";
        let config = read_config();

        // Test batch write and verify offsets
        let messages = vec![create_message(b"msg1"), create_message(b"msg2")];
        let offsets = adapter
            .batch_write(&namespace, shard_name, &messages)
            .await
            .unwrap();
        assert_eq!(offsets, vec![0, 1]);

        // Test read by offset
        let records = adapter
            .read_by_offset(&namespace, shard_name, 0, &config)
            .await
            .unwrap();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].data.as_ref(), b"msg1");

        // Test empty batch write
        let empty_offsets = adapter
            .batch_write(&namespace, shard_name, &[])
            .await
            .unwrap();
        assert_eq!(empty_offsets.len(), 0);

        // Test read from non-existent shard
        let empty_records = adapter
            .read_by_offset(&namespace, "nonexistent", 0, &config)
            .await
            .unwrap();
        assert_eq!(empty_records.len(), 0);
    }

    #[tokio::test]
    async fn test_lru_eviction() {
        let config_adapter = StorageDriverMemoryConfig {
            max_records_per_shard: 3,
            ..Default::default()
        };
        let adapter = MemoryStorageAdapter::new(config_adapter);
        let namespace = unique_id();
        let shard_name = "lru-test";
        let config = read_config();

        // Write 5 messages (exceeds capacity)
        for i in 0..5 {
            let msg = create_message_with_metadata(
                format!("msg{}", i).as_bytes(),
                vec![&format!("tag-{}", i)],
                "",
                1000 + i * 100,
            );
            adapter.write(&namespace, shard_name, &msg).await.unwrap();
        }

        // Verify all 5 messages are stored before eviction
        let shard_key = adapter.shard_key(&namespace, shard_name);
        assert_eq!(adapter.shard_data.get(&shard_key).unwrap().len(), 5);

        // Trigger LRU eviction
        adapter
            .message_expire(&MessageExpireConfig::default())
            .await
            .unwrap();

        // Verify only 3 newest messages remain
        assert_eq!(adapter.shard_data.get(&shard_key).unwrap().len(), 3);

        // Verify offset state is correct
        let state = adapter.shard_state.get(&shard_key).unwrap();
        assert_eq!(state.start_offset, 2); // First 2 messages evicted
        assert_eq!(state.next_offset, 5); // Next offset to assign

        // Test read evicted records (offset 0-1)
        let evicted = adapter
            .read_by_offset(&namespace, shard_name, 0, &config)
            .await
            .unwrap();
        assert_eq!(evicted.len(), 0);

        // Test read remaining records (offset 2-4)
        let remaining = adapter
            .read_by_offset(&namespace, shard_name, 2, &config)
            .await
            .unwrap();
        assert_eq!(remaining.len(), 3);

        // Test evicted tag is not found
        let evicted_tag = adapter
            .read_by_tag(&namespace, shard_name, 0, "tag-0", &config)
            .await
            .unwrap();
        assert_eq!(evicted_tag.len(), 0);

        // Test remaining tag is found
        let remaining_tag = adapter
            .read_by_tag(&namespace, shard_name, 0, "tag-3", &config)
            .await
            .unwrap();
        assert_eq!(remaining_tag.len(), 1);

        // Test timestamp query after eviction
        let offset_result = adapter
            .get_offset_by_timestamp(&namespace, shard_name, 1150)
            .await
            .unwrap()
            .unwrap();
        assert!(offset_result.offset >= 2); // Should point to remaining messages
    }

    #[tokio::test]
    async fn test_indexed_queries() {
        // Setup
        let adapter = MemoryStorageAdapter::new(StorageDriverMemoryConfig::default());
        let namespace = unique_id();
        let shard_name = "index-test";
        let config = read_config();

        // Write messages with tags and keys
        let msg1 = create_message_with_metadata(b"msg1", vec!["tag-a"], "key-1", 0);
        let msg2 = create_message_with_metadata(b"msg2", vec!["tag-b"], "key-2", 0);
        adapter.write(&namespace, shard_name, &msg1).await.unwrap();
        adapter.write(&namespace, shard_name, &msg2).await.unwrap();

        // Test read by tag (using index)
        let tag_results = adapter
            .read_by_tag(&namespace, shard_name, 0, "tag-a", &config)
            .await
            .unwrap();
        assert_eq!(tag_results.len(), 1);
        assert_eq!(tag_results[0].data.as_ref(), b"msg1");

        // Test read by key (using index)
        let key_results = adapter
            .read_by_key(&namespace, shard_name, 0, "key-2", &config)
            .await
            .unwrap();
        assert_eq!(key_results.len(), 1);
        assert_eq!(key_results[0].data.as_ref(), b"msg2");

        // Test read with non-existent tag
        let empty_tag = adapter
            .read_by_tag(&namespace, shard_name, 0, "nonexistent", &config)
            .await
            .unwrap();
        assert_eq!(empty_tag.len(), 0);

        // Test fallback to linear scan when index is removed
        let shard_key = adapter.shard_key(&namespace, shard_name);
        adapter.tag_index.remove(&shard_key);
        adapter.key_index.remove(&shard_key);

        let fallback_results = adapter
            .read_by_tag(&namespace, shard_name, 0, "tag-a", &config)
            .await
            .unwrap();
        assert_eq!(fallback_results.len(), 1); // Still finds via linear scan
    }

    #[tokio::test]
    async fn test_shard_management() {
        // Setup
        let adapter = MemoryStorageAdapter::new(StorageDriverMemoryConfig::default());
        let namespace = unique_id();

        // Create two shards
        let shard1 = ShardInfo {
            namespace: namespace.clone(),
            shard_name: "shard-1".to_string(),
            replica_num: 3,
        };
        let shard2 = ShardInfo {
            namespace: namespace.clone(),
            shard_name: "shard-2".to_string(),
            replica_num: 3,
        };
        adapter.create_shard(&shard1).await.unwrap();
        adapter.create_shard(&shard2).await.unwrap();

        // Test list all shards
        let all_shards = adapter.list_shard(&namespace, "").await.unwrap();
        assert_eq!(all_shards.len(), 2);

        // Test list specific shard
        let specific_shard = adapter.list_shard(&namespace, "shard-1").await.unwrap();
        assert_eq!(specific_shard.len(), 1);
        assert_eq!(specific_shard[0].shard_name, "shard-1");

        // Test delete shard
        adapter.delete_shard(&namespace, "shard-1").await.unwrap();

        // Verify shard-1 is deleted
        let remaining_shards = adapter.list_shard(&namespace, "").await.unwrap();
        assert_eq!(remaining_shards.len(), 1);

        let deleted_shard = adapter.list_shard(&namespace, "shard-1").await.unwrap();
        assert_eq!(deleted_shard.len(), 0);
    }

    #[tokio::test]
    async fn test_consumer_group_offset() {
        // Setup
        let adapter = MemoryStorageAdapter::new(StorageDriverMemoryConfig::default());
        let namespace = unique_id();
        let shard_name = "group-test";
        let group_name = "consumer-group-1";

        // Write some messages
        let messages = vec![
            create_message(b"msg1"),
            create_message(b"msg2"),
            create_message(b"msg3"),
        ];
        adapter
            .batch_write(&namespace, shard_name, &messages)
            .await
            .unwrap();

        // Commit offset 1 for the consumer group
        let mut offsets = HashMap::from([(shard_name.to_string(), 1u64)]);
        adapter
            .commit_offset(group_name, &namespace, &offsets)
            .await
            .unwrap();

        // Verify committed offset
        let group_offsets = adapter.get_offset_by_group(group_name).await.unwrap();
        assert_eq!(group_offsets.len(), 1);
        assert_eq!(group_offsets[0].offset, 1);

        // Update offset to 2
        offsets.insert(shard_name.to_string(), 2);
        adapter
            .commit_offset(group_name, &namespace, &offsets)
            .await
            .unwrap();

        // Verify updated offset
        let updated_offsets = adapter.get_offset_by_group(group_name).await.unwrap();
        assert_eq!(updated_offsets[0].offset, 2);

        // Test get offset for non-existent group
        let empty_offsets = adapter
            .get_offset_by_group("nonexistent-group")
            .await
            .unwrap();
        assert_eq!(empty_offsets.len(), 0);
    }

    #[tokio::test]
    async fn test_timestamp_query() {
        // Setup
        let adapter = MemoryStorageAdapter::new(StorageDriverMemoryConfig::default());
        let namespace = unique_id();
        let shard_name = "timestamp-test";

        // Write messages with different timestamps
        let timestamps = [1000u64, 2000, 3000];
        for (i, &timestamp) in timestamps.iter().enumerate() {
            let msg =
                create_message_with_metadata(format!("msg{}", i).as_bytes(), vec![], "", timestamp);
            adapter.write(&namespace, shard_name, &msg).await.unwrap();
        }

        // Test query: timestamp 1500 should return offset 1 (closest match)
        let result = adapter
            .get_offset_by_timestamp(&namespace, shard_name, 1500)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.offset, 1);

        // Test query: timestamp 500 should return offset 0 (first message)
        let result = adapter
            .get_offset_by_timestamp(&namespace, shard_name, 500)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(result.offset, 0);

        // Test query: timestamp 4000 (beyond all messages) should return None
        let result = adapter
            .get_offset_by_timestamp(&namespace, shard_name, 4000)
            .await
            .unwrap();
        assert!(result.is_none());

        // Test query: non-existent shard should return None
        let result = adapter
            .get_offset_by_timestamp(&namespace, "nonexistent", 1000)
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
