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

use crate::message_expire::MessageExpireConfig;
use crate::storage::{ShardInfo, ShardOffset, StorageAdapter};
use axum::async_trait;
use common_base::error::common::CommonError;
use key::*;
use metadata_struct::adapter::{read_config::ReadConfig, record::Record};
use rocksdb::WriteBatch;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_BROKER;
use std::{collections::HashMap, sync::Arc};

mod expire;
mod key;
#[derive(Clone)]
pub struct RocksDBStorageAdapter {
    pub db: Arc<RocksDBEngine>,
}

impl RocksDBStorageAdapter {
    pub fn new(db: Arc<RocksDBEngine>) -> Self {
        RocksDBStorageAdapter { db }
    }

    fn get_cf(&self) -> Result<Arc<rocksdb::BoundColumnFamily<'_>>, CommonError> {
        self.db.cf_handle(DB_COLUMN_FAMILY_BROKER).ok_or_else(|| {
            CommonError::CommonError(format!(
                "Column family '{}' not found",
                DB_COLUMN_FAMILY_BROKER
            ))
        })
    }

    fn get_offset(&self, namespace: &str, shard_name: &str) -> Result<u64, CommonError> {
        let cf = self.get_cf()?;
        let shard_offset_key = shard_offset_key(namespace, shard_name);
        let offset = match self.db.read::<u64>(cf.clone(), &shard_offset_key)? {
            Some(offset) => offset,
            None => {
                return Err(CommonError::CommonError(format!(
                    "shard {shard_name} under {namespace} not exists"
                )));
            }
        };
        Ok(offset)
    }

    fn save_offset(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
    ) -> Result<(), CommonError> {
        let cf = self.get_cf()?;
        let shard_offset_key = shard_offset_key(namespace, shard_name);
        self.db.write(cf, &shard_offset_key, &offset)?;
        Ok(())
    }

    fn batch_write_internal(
        &self,
        namespace: &str,
        shard_name: &str,
        messages: &[Record],
    ) -> Result<Vec<u64>, CommonError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }
        let cf = self.get_cf()?;
        let offset = self.get_offset(namespace, shard_name)?;

        let mut start_offset = offset;
        let mut offset_res = Vec::with_capacity(messages.len());
        let mut batch = WriteBatch::default();

        for msg in messages {
            offset_res.push(start_offset);

            // Clone and set offset (unavoidable due to serialization requirement)
            let mut record_to_save = msg.clone();
            record_to_save.offset = Some(start_offset);

            // save record (using bincode for better performance)
            let shard_record_key = shard_record_key(namespace, shard_name, start_offset);
            let serialized_msg = bincode::serialize(&record_to_save).map_err(|e| {
                CommonError::CommonError(format!("Failed to serialize record: {e}"))
            })?;
            batch.put_cf(&cf, shard_record_key.as_bytes(), &serialized_msg);

            // save key (use original msg to avoid borrow issues)
            if !msg.key.is_empty() {
                let key_offset_key = key_offset_key(namespace, shard_name, &msg.key);
                batch.put_cf(&cf, key_offset_key.as_bytes(), start_offset.to_be_bytes());
            }

            // save tag
            for tag in msg.tags.iter() {
                let tag_offsets_key = tag_offsets_key(namespace, shard_name, tag, start_offset);
                batch.put_cf(&cf, tag_offsets_key.as_bytes(), start_offset.to_be_bytes());
            }

            // save timestamp
            // Time index every 5,000 records
            if msg.timestamp > 0 && start_offset % 5000 == 0 {
                let timestamp_offset_key =
                    timestamp_offset_key(namespace, shard_name, msg.timestamp, start_offset);
                batch.put_cf(
                    &cf,
                    timestamp_offset_key.as_bytes(),
                    start_offset.to_be_bytes(),
                );
            }

            start_offset += 1;
        }

        self.db.write_batch(batch)?;

        self.save_offset(namespace, shard_name, start_offset)?;
        Ok(offset_res)
    }
}

#[async_trait]
impl StorageAdapter for RocksDBStorageAdapter {
    async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
        let namespace = &shard.namespace;
        let shard_name = &shard.shard_name;
        let cf = self.get_cf()?;
        let shard_offset_key = shard_offset_key(namespace, shard_name);

        if self.get_offset(namespace, shard_name).is_ok() {
            return Err(CommonError::CommonError(format!(
                "shard {shard_name} under namespace {namespace} already exists"
            )));
        }

        self.db.write(cf.clone(), &shard_offset_key, &0_u64)?;
        self.db
            .write(cf, &shard_info_key(namespace, shard_name), shard)
    }

    async fn list_shard(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Result<Vec<ShardInfo>, CommonError> {
        let cf = self.get_cf()?;
        let prefix_key = if shard_name.is_empty() {
            if namespace.is_empty() {
                "/shard/".to_string()
            } else {
                format!("/shard/{}/", namespace)
            }
        } else {
            shard_info_key(namespace, shard_name)
        };

        let raw_shard_info = self.db.read_prefix(cf, &prefix_key)?;
        raw_shard_info
            .into_iter()
            .map(|(_, v)| {
                bincode::deserialize::<ShardInfo>(v.as_slice()).map_err(|e| {
                    CommonError::CommonError(format!("Failed to deserialize ShardInfo: {e}"))
                })
            })
            .collect::<Result<Vec<ShardInfo>, CommonError>>()
    }

    async fn delete_shard(&self, namespace: &str, shard_name: &str) -> Result<(), CommonError> {
        let cf = self.get_cf()?;
        self.get_offset(namespace, shard_name)?;

        let record_prefix = shard_record_key_prefix(namespace, shard_name);
        self.db.delete_prefix(cf.clone(), &record_prefix)?;

        let key_index_prefix = format!("/key/{}/{}/", namespace, shard_name);
        self.db.delete_prefix(cf.clone(), &key_index_prefix)?;

        let tag_index_prefix = format!("/tag/{}/{}/", namespace, shard_name);
        self.db.delete_prefix(cf.clone(), &tag_index_prefix)?;

        let timestamp_index_prefix = timestamp_offset_key_prefix(namespace, shard_name);
        self.db.delete_prefix(cf.clone(), &timestamp_index_prefix)?;

        self.db
            .delete(cf.clone(), &shard_offset_key(namespace, shard_name))?;
        self.db.delete(cf, &shard_info_key(namespace, shard_name))
    }

    async fn write(
        &self,
        namespace: &str,
        shard_name: &str,
        message: &Record,
    ) -> Result<u64, CommonError> {
        let offsets =
            self.batch_write_internal(namespace, shard_name, std::slice::from_ref(message))?;

        offsets
            .first()
            .cloned()
            .ok_or_else(|| CommonError::CommonError("Empty offset result from write".to_string()))
    }

    async fn batch_write(
        &self,
        namespace: &str,
        shard_name: &str,
        messages: &[Record],
    ) -> Result<Vec<u64>, CommonError> {
        if messages.is_empty() {
            return Ok(Vec::new());
        }

        self.batch_write_internal(namespace, shard_name, messages)
    }

    async fn read_by_offset(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let cf = self.get_cf()?;

        // Limit the number of keys to prevent overflow and excessive memory allocation
        let max_batch_size = read_config.max_record_num.min(1024) as usize;
        let capacity = max_batch_size.min(1024);

        // Generate keys for batch read (capped at reasonable size)
        let keys: Vec<String> = (offset..offset.saturating_add(max_batch_size as u64))
            .map(|i| shard_record_key(namespace, shard_name, i))
            .collect();

        // Batch read all records at once (much faster than individual reads)
        let batch_results = self.db.multi_get::<Record>(cf, &keys)?;

        // Process results and apply size limits
        let mut records = Vec::with_capacity(capacity);
        let mut total_size = 0;

        for record_opt in batch_results {
            let Some(record) = record_opt else {
                // Stop on first missing record (assumes sequential)
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
        namespace: &str,
        shard_name: &str,
        offset: u64,
        tag: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let cf = self.get_cf()?;
        let tag_offset_key_prefix = tag_offsets_key_prefix(namespace, shard_name, tag);
        let raw_offsets = self.db.read_prefix(cf.clone(), &tag_offset_key_prefix)?;

        let capacity = (read_config.max_record_num as usize).min(1024);
        let mut offsets = Vec::with_capacity(capacity);

        for (_, v) in raw_offsets {
            if v.len() != 8 {
                continue;
            }
            let record_offset = u64::from_be_bytes(
                v.as_slice()
                    .try_into()
                    .map_err(|_| CommonError::CommonError("Invalid offset bytes".to_string()))?,
            );

            if record_offset >= offset {
                offsets.push(record_offset);
                if offsets.len() >= read_config.max_record_num as usize {
                    break;
                }
            }
        }

        // Use batch read for better performance
        if offsets.is_empty() {
            return Ok(Vec::new());
        }

        let keys: Vec<String> = offsets
            .iter()
            .map(|off| shard_record_key(namespace, shard_name, *off))
            .collect();

        let batch_results = self.db.multi_get::<Record>(cf, &keys)?;

        let mut records = Vec::with_capacity(offsets.len());
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

    async fn read_by_key(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        key: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        if read_config.max_record_num == 0 {
            return Ok(Vec::new());
        }

        let cf = self.get_cf()?;
        let key_offset_key = key_offset_key(namespace, shard_name, key);

        let key_offset_bytes = match self.db.db.get_cf(&cf, &key_offset_key) {
            Ok(Some(data)) if data.len() == 8 => data,
            Ok(_) => return Ok(Vec::new()),
            Err(e) => {
                return Err(CommonError::CommonError(format!(
                    "Failed to read key offset: {e:?}"
                )))
            }
        };

        let key_offset = u64::from_be_bytes(
            key_offset_bytes
                .as_slice()
                .try_into()
                .map_err(|_| CommonError::CommonError("Invalid offset bytes".to_string()))?,
        );

        if key_offset >= offset {
            let shard_record_key = shard_record_key(namespace, shard_name, key_offset);
            let Some(record) = self.db.read::<Record>(cf, &shard_record_key)? else {
                return Ok(Vec::new());
            };

            if record.data.len() as u64 > read_config.max_size {
                return Ok(Vec::new());
            }

            Ok(vec![record])
        } else {
            Ok(Vec::new())
        }
    }

    async fn get_offset_by_timestamp(
        &self,
        namespace: &str,
        shard_name: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        let cf = self.get_cf()?;
        let timestamp_prefix = timestamp_offset_key_search_prefix(namespace, shard_name, timestamp);
        let raw_res = self.db.read_prefix(cf.clone(), &timestamp_prefix)?;

        if let Some((_, v)) = raw_res.first() {
            if v.len() == 8 {
                let offset =
                    u64::from_be_bytes(v.as_slice().try_into().map_err(|_| {
                        CommonError::CommonError("Invalid offset bytes".to_string())
                    })?);
                return Ok(Some(ShardOffset {
                    namespace: namespace.to_string(),
                    shard_name: shard_name.to_string(),
                    segment_no: 0,
                    offset,
                }));
            }
        }

        let timestamp_index_prefix = timestamp_offset_key_prefix(namespace, shard_name);
        let all_timestamps = self.db.read_prefix(cf, &timestamp_index_prefix)?;

        for (key, v) in all_timestamps {
            let parts: Vec<&str> = key.split('/').collect();
            if parts.len() >= 5 {
                if let Ok(ts) = parts[4].parse::<u64>() {
                    if ts >= timestamp && v.len() == 8 {
                        let offset = u64::from_be_bytes(v.as_slice().try_into().map_err(|_| {
                            CommonError::CommonError("Invalid offset bytes".to_string())
                        })?);
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
        let cf = self.get_cf()?;
        let group_record_offsets_key_prefix = group_record_offsets_key_prefix(group_name);
        let raw_offsets = self.db.read_prefix(cf, &group_record_offsets_key_prefix)?;

        let mut offsets = Vec::with_capacity(raw_offsets.len());

        for (_, v) in raw_offsets {
            if v.len() != 8 {
                continue;
            }
            let offset = u64::from_be_bytes(
                v.as_slice()
                    .try_into()
                    .map_err(|_| CommonError::CommonError("Invalid offset bytes".to_string()))?,
            );
            offsets.push(ShardOffset {
                offset,
                ..Default::default()
            });
        }

        Ok(offsets)
    }

    async fn commit_offset(
        &self,
        group_name: &str,
        namespace: &str,
        offsets: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        if offsets.is_empty() {
            return Ok(());
        }

        let cf = self.get_cf()?;

        for (shard_name, offset) in offsets.iter() {
            let group_record_offsets_key =
                group_record_offsets_key(group_name, namespace, shard_name);
            self.db
                .write(cf.clone(), &group_record_offsets_key, &offset.to_be_bytes())?;
        }

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
    use common_base::{tools::unique_id, utils::crc::calc_crc32};
    use futures::future;
    use metadata_struct::adapter::{
        read_config::ReadConfig,
        record::{Header, Record},
    };
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::{collections::HashMap, sync::Arc, vec};

    #[tokio::test]
    async fn stream_read_write() {
        let db = test_rocksdb_instance();

        let adapter = RocksDBStorageAdapter::new(db);
        let namespace = unique_id();
        let shard_name = "test-shard".to_string();

        // Test create and list shard
        adapter
            .create_shard(&ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            })
            .await
            .unwrap();

        let shards = adapter.list_shard(&namespace, &shard_name).await.unwrap();
        assert_eq!(shards.len(), 1);
        assert_eq!(shards[0].shard_name, shard_name);

        // Test batch write and read by offset
        let messages: Vec<_> = (0..4)
            .map(|i| Record::build_byte(format!("test{}", i).as_bytes().to_vec()))
            .collect();

        let offsets = adapter
            .batch_write(&namespace, &shard_name, &messages)
            .await
            .unwrap();
        assert_eq!(offsets, vec![0, 1, 2, 3]);

        let records = adapter
            .read_by_offset(
                &namespace,
                &shard_name,
                0,
                &ReadConfig {
                    max_record_num: u64::MAX,
                    max_size: u64::MAX,
                },
            )
            .await
            .unwrap();
        assert_eq!(records.len(), 4);

        // Test commit and get offset by group
        let group_id = unique_id();
        let mut commit_offsets = HashMap::new();
        commit_offsets.insert(shard_name.clone(), 2);
        adapter
            .commit_offset(&group_id, &namespace, &commit_offsets)
            .await
            .unwrap();

        let group_offsets = adapter.get_offset_by_group(&group_id).await.unwrap();
        assert_eq!(group_offsets[0].offset, 2);

        // Test delete shard
        adapter.delete_shard(&namespace, &shard_name).await.unwrap();
        let shards = adapter.list_shard(&namespace, &shard_name).await.unwrap();
        assert_eq!(shards.len(), 0);

        adapter.close().await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn concurrency_test() {
        let db = test_rocksdb_instance();
        let adapter = Arc::new(RocksDBStorageAdapter::new(db));
        let namespace = unique_id();
        let shards: Vec<_> = (0..4).map(|i| format!("shard-{i}")).collect();

        // Create shards
        for shard in &shards {
            adapter
                .create_shard(&ShardInfo {
                    namespace: namespace.clone(),
                    shard_name: shard.clone(),
                    replica_num: 1,
                })
                .await
                .unwrap();
        }

        assert_eq!(adapter.list_shard(&namespace, "").await.unwrap().len(), 4);

        // Concurrent write and read test
        let tasks: Vec<_> = (0..100)
            .map(|tid| {
                let adapter = adapter.clone();
                let namespace = namespace.clone();
                let shard_name = shards[tid % shards.len()].clone();

                tokio::spawn(async move {
                    let records: Vec<_> = (0..10)
                        .map(|idx| {
                            let value = format!("data-{tid}-{idx}").as_bytes().to_vec();
                            Record {
                                offset: None,
                                header: vec![Header {
                                    name: "test".to_string(),
                                    value: "value".to_string(),
                                }],
                                key: format!("key-{tid}-{idx}"),
                                data: value.clone(),
                                tags: vec![format!("tag-{tid}")],
                                timestamp: 0,
                                crc_num: calc_crc32(&value),
                            }
                        })
                        .collect();

                    let offsets = adapter
                        .batch_write(&namespace, &shard_name, &records)
                        .await
                        .unwrap();

                    assert_eq!(offsets.len(), 10);

                    // Verify read by offset
                    let read_records = adapter
                        .read_by_offset(
                            &namespace,
                            &shard_name,
                            offsets[0],
                            &ReadConfig {
                                max_record_num: 10,
                                max_size: u64::MAX,
                            },
                        )
                        .await
                        .unwrap();

                    assert_eq!(read_records.len(), 10);

                    // Verify read by tag
                    let tag_records = adapter
                        .read_by_tag(
                            &namespace,
                            &shard_name,
                            0,
                            &format!("tag-{tid}"),
                            &ReadConfig {
                                max_record_num: u64::MAX,
                                max_size: u64::MAX,
                            },
                        )
                        .await
                        .unwrap();

                    assert_eq!(tag_records.len(), 10);
                })
            })
            .collect();

        future::join_all(tasks).await;

        // Verify total records per shard
        for shard in &shards {
            let records = adapter
                .read_by_offset(
                    &namespace,
                    shard,
                    0,
                    &ReadConfig {
                        max_record_num: u64::MAX,
                        max_size: u64::MAX,
                    },
                )
                .await
                .unwrap();

            assert_eq!(records.len(), (100 / shards.len()) * 10);
        }

        // Cleanup
        for shard in &shards {
            adapter.delete_shard(&namespace, shard).await.unwrap();
        }

        assert_eq!(adapter.list_shard(&namespace, "").await.unwrap().len(), 0);
        adapter.close().await.unwrap();
    }
}
