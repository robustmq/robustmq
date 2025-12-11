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
use crate::file::key::*;
use crate::storage::{ShardInfo, ShardOffset, StorageAdapter};
use axum::async_trait;
use common_base::tools::now_second;
use common_base::utils::serialize::{deserialize, serialize};
use common_base::{error::common::CommonError, utils::serialize};
use dashmap::DashMap;
use metadata_struct::adapter::{read_config::ReadConfig, record::Record};
use rocksdb::WriteBatch;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_BROKER;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

mod expire;
mod key;

#[derive(Serialize, Deserialize, Clone)]
struct OffsetInfo {
    pub group_name: String,
    pub shard_name: String,
    pub offset: u64,
}

#[derive(Serialize, Deserialize, Clone)]
struct IndexInfo {
    pub shard_name: String,
    pub offset: u64,
    pub create_time: u64,
}

#[inline]
pub fn parse_offset_bytes(bytes: &[u8]) -> Result<u64, CommonError> {
    if bytes.len() != 8 {
        return Err(CommonError::CommonError(format!(
            "Invalid offset bytes length: expected 8, got {}",
            bytes.len()
        )));
    }

    Ok(u64::from_be_bytes(bytes.try_into().map_err(|_| {
        CommonError::CommonError("Failed to convert offset bytes".to_string())
    })?))
}

#[derive(Clone)]
pub struct RocksDBStorageAdapter {
    pub db: Arc<RocksDBEngine>,
    shard_write_locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
}

impl RocksDBStorageAdapter {
    pub fn new(db: Arc<RocksDBEngine>) -> Self {
        RocksDBStorageAdapter {
            db,
            shard_write_locks: DashMap::new(),
        }
    }

    fn get_cf(&self) -> Result<Arc<rocksdb::BoundColumnFamily<'_>>, CommonError> {
        self.db.cf_handle(DB_COLUMN_FAMILY_BROKER).ok_or_else(|| {
            CommonError::CommonError(format!(
                "Column family '{}' not found",
                DB_COLUMN_FAMILY_BROKER
            ))
        })
    }

    fn get_offset(&self, shard_name: &str) -> Result<u64, CommonError> {
        let cf = self.get_cf()?;
        let shard_offset_key = shard_offset_key(shard_name);
        let offset = match self.db.read::<u64>(cf.clone(), &shard_offset_key)? {
            Some(offset) => offset,
            None => {
                return Err(CommonError::CommonError(format!(
                    "shard {shard_name} not exists"
                )));
            }
        };
        Ok(offset)
    }

    fn save_offset(&self, shard_name: &str, offset: u64) -> Result<(), CommonError> {
        let cf = self.get_cf()?;
        let shard_offset_key = shard_offset_key(shard_name);
        self.db.write(cf, &shard_offset_key, &offset)?;
        Ok(())
    }

    async fn batch_write_internal(
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

        let cf = self.get_cf()?;
        let offset = self.get_offset(shard_name)?;

        let mut start_offset = offset;
        let mut offset_res = Vec::with_capacity(messages.len());
        let mut batch = WriteBatch::default();

        for msg in messages {
            offset_res.push(start_offset);

            // save message
            let mut record_to_save = msg.clone();
            record_to_save.offset = Some(start_offset);
            let shard_record_key = shard_record_key(shard_name, start_offset);
            let serialized_msg = serialize::serialize(&record_to_save)?;
            batch.put_cf(&cf, shard_record_key.as_bytes(), &serialized_msg);

            // save index
            let offset_info = IndexInfo {
                shard_name: shard_name.to_string(),
                offset: start_offset,
                create_time: now_second(),
            };
            let offset_info_data = serialize(&offset_info)?;

            // key index
            if let Some(key) = &msg.key {
                let key_offset_key = key_offset_key(shard_name, key);
                batch.put_cf(&cf, key_offset_key.as_bytes(), offset_info_data.clone());
            }

            // tag index
            if let Some(tags) = &msg.tags {
                for tag in tags.iter() {
                    let tag_offsets_key = tag_offsets_key(shard_name, tag, start_offset);
                    batch.put_cf(&cf, tag_offsets_key.as_bytes(), offset_info_data.clone());
                }
            }

            // timestamp index
            if msg.timestamp > 0 && start_offset % 5000 == 0 {
                let timestamp_offset_key =
                    timestamp_offset_key(shard_name, msg.timestamp, start_offset);
                batch.put_cf(
                    &cf,
                    timestamp_offset_key.as_bytes(),
                    offset_info_data.clone(),
                );
            }

            // offset incr
            start_offset += 1;
        }

        self.db.write_batch(batch)?;

        self.save_offset(shard_name, start_offset)?;
        Ok(offset_res)
    }

    async fn search_index_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<IndexInfo>, CommonError> {
        let cf = self.get_cf()?;
        let mut result = None;
        let timestamp_index_prefix = timestamp_offset_key_prefix(shard);
        let mut iter = self.db.db.raw_iterator_cf(&cf);
        iter.seek(&timestamp_index_prefix);

        let mut prefix_index = None;
        while iter.valid() {
            let Some(key_bytes) = iter.key() else {
                break;
            };

            let Some(value_byte) = iter.value() else {
                break;
            };

            let key = match String::from_utf8(key_bytes.to_vec()) {
                Ok(k) => k,
                Err(_) => {
                    iter.next();
                    continue;
                }
            };

            if !key.starts_with(&timestamp_index_prefix) {
                break;
            }

            let index = deserialize::<IndexInfo>(value_byte)?;
            if index.create_time > timestamp {
                result = prefix_index;
                break;
            }

            prefix_index = Some(index);
            iter.next();
        }

        Ok(result)
    }

    async fn read_data_by_time(
        &self,
        shard: &str,
        start_index: &Option<IndexInfo>,
        timestamp: u64,
    ) -> Result<Option<Record>, CommonError> {
        let cf = self.get_cf()?;
        let mut result = None;
        let timestamp_index_prefix = if let Some(si) = start_index {
            shard_record_key(shard, si.offset)
        } else {
            shard_record_key_prefix(shard)
        };

        let mut iter = self.db.db.raw_iterator_cf(&cf);
        iter.seek(&timestamp_index_prefix);

        let mut prefix_record = None;
        while iter.valid() {
            let Some(key_bytes) = iter.key() else {
                break;
            };

            let Some(value_byte) = iter.value() else {
                break;
            };

            let key = match String::from_utf8(key_bytes.to_vec()) {
                Ok(k) => k,
                Err(_) => {
                    iter.next();
                    continue;
                }
            };

            if !key.starts_with(&timestamp_index_prefix) {
                break;
            }

            let record = deserialize::<Record>(value_byte)?;
            if record.timestamp > timestamp {
                result = prefix_record;
                break;
            }

            prefix_record = Some(record);
            iter.next();
        }

        Ok(result)
    }
}

#[async_trait]
impl StorageAdapter for RocksDBStorageAdapter {
    async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
        let shard_name = &shard.shard_name;
        let cf = self.get_cf()?;
        let shard_info_key = shard_info_key(shard_name);

        if self.db.exist(cf.clone(), &shard_info_key) {
            return Err(CommonError::CommonError(format!(
                "shard {shard_name} already exists"
            )));
        }

        // init shard
        self.db.write(cf.clone(), &shard_info_key, shard)?;

        // init shard offset
        let shard_offset_key = shard_offset_key(shard_name);
        self.db.write(cf.clone(), &shard_offset_key, &0_u64)?;

        // init shard lock
        self.shard_write_locks
            .entry(shard_name.to_string())
            .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())));

        Ok(())
    }

    async fn list_shard(&self, shard: &str) -> Result<Vec<ShardInfo>, CommonError> {
        let cf = self.get_cf()?;
        let prefix_key = if shard.is_empty() {
            "/shard/".to_string()
        } else {
            shard_info_key(shard)
        };

        let raw_shard_info = self.db.read_prefix(cf, &prefix_key)?;
        raw_shard_info
            .into_iter()
            .map(|(_, v)| serialize::deserialize::<ShardInfo>(v.as_slice()))
            .collect::<Result<Vec<ShardInfo>, CommonError>>()
    }

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError> {
        let cf = self.get_cf()?;
        let shard_info_key = shard_info_key(shard);

        if !self.db.exist(cf.clone(), &shard_info_key) {
            return Err(CommonError::CommonError(format!(
                "shard {shard} does not exist"
            )));
        }

        // delete records
        let record_prefix = shard_record_key_prefix(shard);
        self.db.delete_prefix(cf.clone(), &record_prefix)?;

        // delete key index
        let key_index_prefix = format!("/key/{}/", shard);
        self.db.delete_prefix(cf.clone(), &key_index_prefix)?;

        // delete tag index
        let tag_index_prefix = format!("/tag/{}/", shard);
        self.db.delete_prefix(cf.clone(), &tag_index_prefix)?;

        // delete timestamp index
        let timestamp_index_prefix = timestamp_offset_key_prefix(shard);
        self.db.delete_prefix(cf.clone(), &timestamp_index_prefix)?;

        // delete shard offset
        self.db.delete(cf.clone(), &shard_offset_key(shard))?;

        // delete shard info
        self.db.delete(cf, &shard_info_key)?;

        // remove lock
        self.shard_write_locks.remove(shard);

        Ok(())
    }

    async fn write(&self, shard: &str, message: &Record) -> Result<u64, CommonError> {
        let offsets = self
            .batch_write_internal(shard, &vec![message.clone()])
            .await?;

        offsets
            .first()
            .cloned()
            .ok_or_else(|| CommonError::CommonError("Empty offset result from write".to_string()))
    }

    async fn batch_write(&self, shard: &str, messages: &[Record]) -> Result<Vec<u64>, CommonError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        self.batch_write_internal(shard, messages).await
    }

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let cf = self.get_cf()?;

        let keys: Vec<String> = (offset..offset.saturating_add(read_config.max_record_num as u64))
            .map(|i| shard_record_key(shard, i))
            .collect();

        let mut records = Vec::new();
        let mut total_size = 0;

        let batch_results = self.db.multi_get::<Record>(cf, &keys)?;
        for record_opt in batch_results {
            let Some(record) = record_opt else {
                break;
            };

            let record_bytes = record.data.len() as u64;
            if total_size + record_bytes > read_config.max_size {
                break;
            }

            total_size += record_bytes;
            records.push(record);
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
        let cf = self.get_cf()?;
        let tag_offset_key_prefix = tag_offsets_key_prefix(shard, tag);
        let tag_entries = self.db.read_prefix(cf.clone(), &tag_offset_key_prefix)?;

        // Filter and collect offsets >= specified offset
        let mut offsets = Vec::new();
        for (_key, value) in tag_entries {
            let record_offset = deserialize::<IndexInfo>(&value)?;

            if let Some(so) = start_offset {
                if record_offset.offset < so {
                    continue;
                }
            }

            offsets.push(record_offset.offset);
            if offsets.len() >= read_config.max_record_num as usize {
                break;
            }
        }

        if offsets.is_empty() {
            return Ok(Vec::new());
        }

        // Build record keys from offsets
        let keys: Vec<String> = offsets
            .iter()
            .map(|off| shard_record_key(shard, *off))
            .collect();

        // Batch read records
        let batch_results = self.db.multi_get::<Record>(cf, &keys)?;
        let mut records = Vec::new();
        let mut total_size = 0;

        for record_opt in batch_results {
            let Some(record) = record_opt else {
                continue;
            };

            let record_bytes = record.data.len() as u64;
            if total_size + record_bytes > read_config.max_size {
                break;
            }

            total_size += record_bytes;
            records.push(record);
        }

        Ok(records)
    }

    async fn read_by_key(&self, shard: &str, key: &str) -> Result<Vec<Record>, CommonError> {
        let cf = self.get_cf()?;
        let key_offset_key = key_offset_key(shard, key);

        let key_offset_bytes = match self.db.db.get_cf(&cf, &key_offset_key) {
            Ok(Some(data)) => data,
            Ok(_) => return Ok(Vec::new()),
            Err(e) => {
                return Err(CommonError::CommonError(format!(
                    "Failed to read key offset: {e:?}"
                )))
            }
        };

        let index = deserialize::<IndexInfo>(&key_offset_bytes)?;

        let shard_record_key = shard_record_key(shard, index.offset);
        let Some(record) = self.db.read::<Record>(cf, &shard_record_key)? else {
            return Ok(Vec::new());
        };

        Ok(vec![record])
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        let index: Option<IndexInfo> = self.search_index_by_timestamp(shard, timestamp).await?;
        if let Some(record) = self.read_data_by_time(shard, &index, timestamp).await? {
            if let Some(offset) = record.offset {
                return Ok(Some(ShardOffset {
                    shard_name: shard.to_string(),
                    offset: offset,
                    ..Default::default()
                }));
            }
        }
        Ok(None)
    }

    async fn get_offset_by_group(&self, group_name: &str) -> Result<Vec<ShardOffset>, CommonError> {
        let cf = self.get_cf()?;
        let group_record_offsets_key_prefix = group_record_offsets_key_prefix(group_name);

        let mut offsets = Vec::new();
        for (_, v) in self.db.read_prefix(cf, &group_record_offsets_key_prefix)? {
            let info = deserialize::<OffsetInfo>(&v)?;
            offsets.push(ShardOffset {
                group: info.group_name,
                shard_name: info.shard_name,
                offset: info.offset,
                ..Default::default()
            });
        }

        Ok(offsets)
    }

    async fn commit_offset(
        &self,
        group_name: &str,
        offsets: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        if offsets.is_empty() {
            return Ok(());
        }

        let cf = self.get_cf()?;
        let mut batch = WriteBatch::default();

        for (shard_name, offset) in offsets.iter() {
            let group_record_offsets_key = group_record_offsets_key(group_name, shard_name);
            let info = OffsetInfo {
                group_name: group_name.to_string(),
                shard_name: shard_name.to_string(),
                offset: *offset,
            };
            batch.put_cf(&cf, group_record_offsets_key.as_bytes(), serialize(&info)?);
        }

        self.db.write_batch(batch)?;

        Ok(())
    }

    async fn message_expire(&self, config: &MessageExpireConfig) -> Result<(), CommonError> {
        expire::expire_messages_by_timestamp(self.db.clone(), config).await
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RocksDBStorageAdapter;
    use crate::storage::{ShardInfo, StorageAdapter};
    use common_base::tools::{now_millis, unique_id};
    use futures::future;
    use metadata_struct::adapter::{read_config::ReadConfig, record::Record};
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::{collections::HashMap, sync::Arc};

    async fn create_test_shard(adapter: &RocksDBStorageAdapter, name: &str) -> ShardInfo {
        let shard = ShardInfo {
            shard_name: name.to_string(),
            replica_num: 1,
            ..Default::default()
        };
        adapter.create_shard(&shard).await.unwrap();
        shard
    }

    #[tokio::test]
    async fn test_shard_lifecycle() {
        let adapter = RocksDBStorageAdapter::new(test_rocksdb_instance());
        let shard_name = "test-shard";

        let _shard = create_test_shard(&adapter, shard_name).await;

        let shards = adapter.list_shard(shard_name).await.unwrap();
        assert_eq!(shards.len(), 1);
        assert_eq!(shards[0].shard_name, shard_name);

        let all_shards = adapter.list_shard("").await.unwrap();
        assert!(!all_shards.is_empty());

        adapter.delete_shard(shard_name).await.unwrap();
        let shards = adapter.list_shard(shard_name).await.unwrap();
        assert_eq!(shards.len(), 0);

        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_basic_write_read() {
        let adapter = RocksDBStorageAdapter::new(test_rocksdb_instance());
        let shard = create_test_shard(&adapter, "test-shard").await;

        let messages: Vec<_> = (0..10)
            .map(|i| Record::from_bytes(format!("message-{}", i).as_bytes().to_vec()))
            .collect();

        let offsets = adapter
            .batch_write(&shard.shard_name, &messages)
            .await
            .unwrap();
        assert_eq!(offsets, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        let records = adapter
            .read_by_offset(
                &shard.shard_name,
                0,
                &ReadConfig {
                    max_record_num: 10,
                    max_size: u64::MAX,
                },
            )
            .await
            .unwrap();
        assert_eq!(records.len(), 10);
        assert_eq!(String::from_utf8_lossy(&records[5].data), "message-5");

        let records = adapter
            .read_by_offset(
                &shard.shard_name,
                5,
                &ReadConfig {
                    max_record_num: 3,
                    max_size: u64::MAX,
                },
            )
            .await
            .unwrap();
        assert_eq!(records.len(), 3);
        assert_eq!(String::from_utf8_lossy(&records[0].data), "message-5");

        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_group_offset() {
        let adapter = RocksDBStorageAdapter::new(test_rocksdb_instance());
        let shard1 = create_test_shard(&adapter, "shard1").await;
        let shard2 = create_test_shard(&adapter, "shard2").await;

        let group_id = unique_id();
        let mut offsets = HashMap::new();
        offsets.insert(shard1.shard_name.clone(), 100);
        offsets.insert(shard2.shard_name.clone(), 200);
        adapter.commit_offset(&group_id, &offsets).await.unwrap();

        let group_offsets = adapter.get_offset_by_group(&group_id).await.unwrap();
        assert_eq!(group_offsets.len(), 2);

        for offset_info in &group_offsets {
            if offset_info.shard_name == shard1.shard_name {
                assert_eq!(offset_info.offset, 100);
            } else if offset_info.shard_name == shard2.shard_name {
                assert_eq!(offset_info.offset, 200);
            } else {
                panic!("Unexpected shard_name: {}", offset_info.shard_name);
            }
        }

        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_write_single_record() {
        let adapter = RocksDBStorageAdapter::new(test_rocksdb_instance());
        let shard = create_test_shard(&adapter, "test-shard").await;

        let record = Record::from_string("single message".to_string());
        let offset = adapter.write(&shard.shard_name, &record).await.unwrap();
        assert_eq!(offset, 0);

        let records = adapter
            .read_by_offset(
                &shard.shard_name,
                0,
                &ReadConfig {
                    max_record_num: 1,
                    max_size: u64::MAX,
                },
            )
            .await
            .unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(String::from_utf8_lossy(&records[0].data), "single message");

        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_read_by_key() {
        let adapter = RocksDBStorageAdapter::new(test_rocksdb_instance());
        let shard = create_test_shard(&adapter, "test-shard").await;

        let records: Vec<_> = (0..10)
            .map(|i| Record::from_string(format!("msg-{}", i)).with_key(format!("key-{}", i)))
            .collect();

        adapter
            .batch_write(&shard.shard_name, &records)
            .await
            .unwrap();

        let found = adapter
            .read_by_key(&shard.shard_name, "key-5")
            .await
            .unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(String::from_utf8_lossy(&found[0].data), "msg-5");
        assert_eq!(found[0].key(), Some("key-5"));

        let not_found = adapter
            .read_by_key(&shard.shard_name, "nonexistent")
            .await
            .unwrap();
        assert_eq!(not_found.len(), 0);

        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_read_by_tag() {
        let adapter = RocksDBStorageAdapter::new(test_rocksdb_instance());
        let shard = create_test_shard(&adapter, "test-shard").await;

        let records: Vec<_> = (0..15)
            .map(|i| {
                let tag = if i % 3 == 0 {
                    "tag-a"
                } else if i % 3 == 1 {
                    "tag-b"
                } else {
                    "tag-c"
                };
                Record::from_string(format!("msg-{}", i)).with_tags(vec![tag.to_string()])
            })
            .collect();

        adapter
            .batch_write(&shard.shard_name, &records)
            .await
            .unwrap();

        let tag_a_records = adapter
            .read_by_tag(
                &shard.shard_name,
                "tag-a",
                None,
                &ReadConfig {
                    max_record_num: u64::MAX,
                    max_size: u64::MAX,
                },
            )
            .await
            .unwrap();
        assert_eq!(tag_a_records.len(), 5);

        let tag_b_records = adapter
            .read_by_tag(
                &shard.shard_name,
                "tag-b",
                None,
                &ReadConfig {
                    max_record_num: u64::MAX,
                    max_size: u64::MAX,
                },
            )
            .await
            .unwrap();
        assert_eq!(tag_b_records.len(), 5);

        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_get_offset_by_timestamp() {
        let adapter = RocksDBStorageAdapter::new(test_rocksdb_instance());
        let shard = create_test_shard(&adapter, "test-shard").await;

        let base_ts = 1000000u64;
        let mut records = Vec::new();
        // Sparse timestamp index is created every 5000 offsets
        for i in 0..15000 {
            let mut record = Record::from_string(format!("msg-{}", i));
            record.timestamp = base_ts + i;
            records.push(record);
        }

        adapter
            .batch_write(&shard.shard_name, &records)
            .await
            .unwrap();

        // Query at offset 5000 (has index)
        let result = adapter
            .get_offset_by_timestamp(&shard.shard_name, base_ts + 5000)
            .await
            .unwrap();
        assert!(
            result.is_some(),
            "Expected to find offset for timestamp {}",
            base_ts + 5000
        );
        assert_eq!(result.unwrap().offset, 5000);

        // Query at offset 10000 (has index)
        let result2 = adapter
            .get_offset_by_timestamp(&shard.shard_name, base_ts + 10000)
            .await
            .unwrap();
        assert!(result2.is_some());
        assert_eq!(result2.unwrap().offset, 10000);

        // Query before first data returns offset 0 (first available)
        let early_result = adapter
            .get_offset_by_timestamp(&shard.shard_name, base_ts - 1000)
            .await
            .unwrap();
        assert!(early_result.is_some());
        assert_eq!(early_result.unwrap().offset, 0);

        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_record_with_metadata() {
        let adapter = RocksDBStorageAdapter::new(test_rocksdb_instance());
        let shard = create_test_shard(&adapter, "test-shard").await;

        let timestamp = now_millis() as u64;
        let records: Vec<_> = (0..5)
            .map(|i| {
                Record::from_string(format!("data-{}", i))
                    .with_key(format!("key-{}", i))
                    .with_tags(vec![format!("tag-{}", i), "common-tag".to_string()])
                    .with_timestamp(timestamp + i * 1000)
            })
            .collect();

        let offsets = adapter
            .batch_write(&shard.shard_name, &records)
            .await
            .unwrap();
        assert_eq!(offsets.len(), 5);

        let read_records = adapter
            .read_by_offset(
                &shard.shard_name,
                0,
                &ReadConfig {
                    max_record_num: 10,
                    max_size: u64::MAX,
                },
            )
            .await
            .unwrap();

        for (i, record) in read_records.iter().enumerate() {
            assert_eq!(String::from_utf8_lossy(&record.data), format!("data-{}", i));
            assert_eq!(record.key(), Some(&format!("key-{}", i)[..]));
            assert_eq!(record.tags().len(), 2);
            assert!(record.tags().contains(&format!("tag-{}", i)));
            assert!(record.tags().contains(&"common-tag".to_string()));
            assert_eq!(record.timestamp, timestamp + i as u64 * 1000);
        }

        let common_tag_records = adapter
            .read_by_tag(
                &shard.shard_name,
                "common-tag",
                None,
                &ReadConfig::default(),
            )
            .await
            .unwrap();
        assert_eq!(common_tag_records.len(), 5);

        adapter.close().await.unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_write_offset_uniqueness() {
        let adapter = Arc::new(RocksDBStorageAdapter::new(test_rocksdb_instance()));
        let shard = create_test_shard(&adapter, "test-shard").await;

        let tasks: Vec<_> = (0..50)
            .map(|tid| {
                let adapter = adapter.clone();
                let shard = shard.clone();

                tokio::spawn(async move {
                    let records: Vec<_> = (0..20)
                        .map(|idx| {
                            Record::from_string(format!("data-{}-{}", tid, idx))
                                .with_key(format!("key-{}-{}", tid, idx))
                                .with_tags(vec![format!("tag-{}", tid)])
                        })
                        .collect();

                    adapter
                        .batch_write(&shard.shard_name, &records)
                        .await
                        .unwrap()
                })
            })
            .collect();

        let results = future::join_all(tasks).await;
        let mut all_offsets = Vec::new();
        for result in results {
            all_offsets.extend(result.unwrap());
        }

        assert_eq!(all_offsets.len(), 1000);

        let mut sorted_offsets = all_offsets.clone();
        sorted_offsets.sort();
        sorted_offsets.dedup();
        assert_eq!(sorted_offsets.len(), 1000);

        for (idx, offset) in sorted_offsets.iter().enumerate() {
            assert_eq!(*offset, idx as u64);
        }

        let read_records = adapter
            .read_by_offset(
                &shard.shard_name,
                0,
                &ReadConfig {
                    max_record_num: 1024,
                    max_size: u64::MAX,
                },
            )
            .await
            .unwrap();

        assert_eq!(read_records.len(), 1000);

        for (idx, record) in read_records.iter().enumerate() {
            assert_eq!(record.offset, Some(idx as u64));
        }

        adapter.close().await.unwrap();
    }
}
