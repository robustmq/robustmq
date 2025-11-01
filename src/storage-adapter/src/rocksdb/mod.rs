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
            let mut msg = msg.clone();
            offset_res.push(start_offset);
            msg.offset = Some(start_offset);

            let shard_record_key = shard_record_key(namespace, shard_name, start_offset);
            let serialized_msg = serde_json::to_string(&msg).map_err(|e| {
                CommonError::CommonError(format!("Failed to serialize record: {e}"))
            })?;
            batch.put_cf(&cf, shard_record_key.as_bytes(), serialized_msg.as_bytes());

            if !msg.key.is_empty() {
                let key_offset_key = key_offset_key(namespace, shard_name, &msg.key);
                let serialized_offset = serde_json::to_string(&start_offset).map_err(|e| {
                    CommonError::CommonError(format!("Failed to serialize offset: {e}"))
                })?;
                batch.put_cf(&cf, key_offset_key.as_bytes(), serialized_offset.as_bytes());
            }

            for tag in msg.tags.iter() {
                let tag_offsets_key = tag_offsets_key(namespace, shard_name, tag, start_offset);
                let serialized_offset = serde_json::to_string(&start_offset).map_err(|e| {
                    CommonError::CommonError(format!("Failed to serialize offset: {e}"))
                })?;
                batch.put_cf(
                    &cf,
                    tag_offsets_key.as_bytes(),
                    serialized_offset.as_bytes(),
                );
            }

            if msg.timestamp > 0 {
                let timestamp_offset_key =
                    timestamp_offset_key(namespace, shard_name, msg.timestamp, start_offset);
                let serialized_offset = serde_json::to_string(&start_offset).map_err(|e| {
                    CommonError::CommonError(format!("Failed to serialize offset: {e}"))
                })?;
                batch.put_cf(
                    &cf,
                    timestamp_offset_key.as_bytes(),
                    serialized_offset.as_bytes(),
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
        self.get_offset(namespace, shard_name)?;

        let shard_offset_key = shard_offset_key(namespace, shard_name);
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
            .map(|(_, v)| serde_json::from_slice::<ShardInfo>(v.as_slice()).map_err(Into::into))
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
        let capacity = (read_config.max_record_num as usize).min(1024);
        let mut records = Vec::with_capacity(capacity);
        let mut total_size = 0;

        for i in offset..offset.saturating_add(read_config.max_record_num) {
            let shard_record_key = shard_record_key(namespace, shard_name, i);
            let Some(record) = self.db.read::<Record>(cf.clone(), &shard_record_key)? else {
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
            let record_offset = serde_json::from_slice::<u64>(&v)?;

            if record_offset >= offset {
                offsets.push(record_offset);
                if offsets.len() >= read_config.max_record_num as usize {
                    break;
                }
            }
        }

        let mut records = Vec::with_capacity(offsets.len());
        let mut total_size = 0;

        for off in offsets {
            let shard_record_key = shard_record_key(namespace, shard_name, off);
            let Some(record) = self.db.read::<Record>(cf.clone(), &shard_record_key)? else {
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

        match self.db.read::<u64>(cf.clone(), &key_offset_key)? {
            Some(key_offset) if key_offset >= offset => {
                let shard_record_key = shard_record_key(namespace, shard_name, key_offset);
                let Some(record) = self.db.read::<Record>(cf, &shard_record_key)? else {
                    return Ok(Vec::new());
                };

                if record.data.len() as u64 > read_config.max_size {
                    return Ok(Vec::new());
                }

                Ok(vec![record])
            }
            _ => Ok(Vec::new()),
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
            let offset = serde_json::from_slice::<u64>(v)?;
            return Ok(Some(ShardOffset {
                namespace: namespace.to_string(),
                shard_name: shard_name.to_string(),
                segment_no: 0,
                offset,
            }));
        }

        let timestamp_index_prefix = timestamp_offset_key_prefix(namespace, shard_name);
        let all_timestamps = self.db.read_prefix(cf, &timestamp_index_prefix)?;

        for (key, v) in all_timestamps {
            let parts: Vec<&str> = key.split('/').collect();
            if parts.len() >= 5 {
                if let Ok(ts) = parts[4].parse::<u64>() {
                    if ts >= timestamp {
                        let offset = serde_json::from_slice::<u64>(&v)?;
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
            let offset = serde_json::from_slice::<u64>(&v)?;
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
                .write(cf.clone(), &group_record_offsets_key, offset)?;
        }

        Ok(())
    }

    async fn message_expire(&self, _config: &MessageExpireConfig) -> Result<(), CommonError> {
        // TODO: Implement message expiration logic
        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc, vec};

    use common_base::{tools::unique_id, utils::crc::calc_crc32};
    use futures::future;
    use metadata_struct::adapter::{
        read_config::ReadConfig,
        record::{Header, Record},
    };
    use rocksdb_engine::rocksdb::RocksDBEngine;

    use crate::storage::{ShardInfo, StorageAdapter};

    use super::RocksDBStorageAdapter;

    async fn read_and_commit_offset(
        adapter: &RocksDBStorageAdapter,
        namespace: &str,
        shard_name: &str,
        group_id: &str,
        expected_data: &str,
        read_config: &ReadConfig,
    ) {
        let offset_data = adapter.get_offset_by_group(group_id).await.unwrap();
        let next_offset = if offset_data.is_empty() {
            0
        } else {
            offset_data.first().unwrap().offset + 1
        };

        let res = adapter
            .read_by_offset(namespace, shard_name, next_offset, read_config)
            .await
            .unwrap();

        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            expected_data
        );

        let mut offsets = HashMap::new();
        offsets.insert(
            shard_name.to_string(),
            res.first().unwrap().clone().offset.unwrap(),
        );
        adapter
            .commit_offset(group_id, namespace, &offsets)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn stream_read_write() {
        let db_path = format!("/tmp/robustmq_{}", unique_id());
        let db = Arc::new(RocksDBEngine::new(
            &db_path,
            100,
            vec![rocksdb_engine::storage::family::DB_COLUMN_FAMILY_BROKER.to_string()],
        ));

        let storage_adapter = RocksDBStorageAdapter::new(db);
        let namespace = unique_id();
        let shard_name = "test-11".to_string();

        storage_adapter
            .create_shard(&ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                replica_num: 1,
            })
            .await
            .unwrap();

        let shards = storage_adapter
            .list_shard(&namespace, &shard_name)
            .await
            .unwrap();

        assert_eq!(shards.len(), 1);
        assert_eq!(shards.first().unwrap().shard_name, shard_name);
        assert_eq!(shards.first().unwrap().namespace, namespace);
        assert_eq!(shards.first().unwrap().replica_num, 1);

        let ms1 = "test1".to_string();
        let ms2 = "test2".to_string();
        let data = vec![
            Record::build_byte(ms1.clone().as_bytes().to_vec()),
            Record::build_byte(ms2.clone().as_bytes().to_vec()),
        ];

        let result = storage_adapter
            .batch_write(&namespace, &shard_name, &data)
            .await
            .unwrap();

        assert_eq!(result.first().unwrap().clone(), 0);
        assert_eq!(result.get(1).unwrap().clone(), 1);

        assert_eq!(
            storage_adapter
                .read_by_offset(
                    &namespace.clone(),
                    &shard_name,
                    0,
                    &ReadConfig {
                        max_record_num: u64::MAX,
                        max_size: u64::MAX,
                    }
                )
                .await
                .unwrap()
                .len(),
            2
        );

        let ms3 = "test3".to_string();
        let ms4 = "test4".to_string();
        let data = vec![
            Record::build_byte(ms3.clone().as_bytes().to_vec()),
            Record::build_byte(ms4.clone().as_bytes().to_vec()),
        ];

        let result = storage_adapter
            .batch_write(&namespace, &shard_name, &data)
            .await
            .unwrap();

        let result_read = storage_adapter
            .read_by_offset(
                &namespace.clone(),
                &shard_name,
                2,
                &ReadConfig {
                    max_record_num: u64::MAX,
                    max_size: u64::MAX,
                },
            )
            .await
            .unwrap();

        assert_eq!(result.first().unwrap().clone(), 2);
        assert_eq!(result.get(1).unwrap().clone(), 3);
        assert_eq!(result_read.len(), 2);

        let group_id = unique_id();
        let read_config = &ReadConfig {
            max_record_num: 1,
            max_size: u64::MAX,
        };

        read_and_commit_offset(
            &storage_adapter,
            &namespace,
            &shard_name,
            &group_id,
            &ms1,
            read_config,
        )
        .await;
        read_and_commit_offset(
            &storage_adapter,
            &namespace,
            &shard_name,
            &group_id,
            &ms2,
            read_config,
        )
        .await;
        read_and_commit_offset(
            &storage_adapter,
            &namespace,
            &shard_name,
            &group_id,
            &ms3,
            read_config,
        )
        .await;
        read_and_commit_offset(
            &storage_adapter,
            &namespace,
            &shard_name,
            &group_id,
            &ms4,
            read_config,
        )
        .await;

        storage_adapter
            .delete_shard(&namespace, &shard_name)
            .await
            .unwrap();

        let shards = storage_adapter
            .list_shard(&namespace, &shard_name)
            .await
            .unwrap();

        assert_eq!(shards.len(), 0);

        storage_adapter.close().await.unwrap();

        let _ = std::fs::remove_dir_all(&db_path);
    }

    #[tokio::test]
    #[ignore]
    async fn concurrency_test() {
        let db_path = format!("/tmp/robustmq_{}", unique_id());
        let db = Arc::new(RocksDBEngine::new(
            &db_path,
            100,
            vec![rocksdb_engine::storage::family::DB_COLUMN_FAMILY_BROKER.to_string()],
        ));

        let storage_adapter = Arc::new(RocksDBStorageAdapter::new(db));

        let namespace = unique_id();
        let shards = (0..4).map(|i| format!("test-{i}")).collect::<Vec<_>>();

        for i in 0..shards.len() {
            storage_adapter
                .create_shard(&ShardInfo {
                    namespace: namespace.clone(),
                    shard_name: shards.get(i).unwrap().clone(),
                    replica_num: 1,
                })
                .await
                .unwrap();
        }

        let list_res = storage_adapter.list_shard(&namespace, "").await.unwrap();

        assert_eq!(list_res.len(), 4);

        let header = vec![Header {
            name: "name".to_string(),
            value: "value".to_string(),
        }];

        let mut tasks = vec![];
        for tid in 0..10000 {
            let storage_adapter = storage_adapter.clone();
            let namespace = namespace.clone();
            let shard_name = shards.get(tid % shards.len()).unwrap().clone();
            let header = header.clone();

            let task = tokio::spawn(async move {
                let mut batch_data = Vec::new();

                for idx in 0..100 {
                    let value = format!("data-{tid}-{idx}").as_bytes().to_vec();
                    let data = Record {
                        offset: None,
                        header: header.clone(),
                        key: format!("key-{tid}-{idx}"),
                        data: value.clone(),
                        tags: vec![format!("task-{}", tid)],
                        timestamp: 0,
                        crc_num: calc_crc32(&value),
                    };

                    batch_data.push(data);
                }

                let write_offsets = storage_adapter
                    .batch_write(&namespace, &shard_name, &batch_data)
                    .await
                    .unwrap();

                assert_eq!(write_offsets.len(), 100);

                let mut read_records = Vec::new();

                for offset in write_offsets.iter() {
                    let records = storage_adapter
                        .read_by_offset(
                            &namespace.clone(),
                            &shard_name,
                            *offset,
                            &ReadConfig {
                                max_record_num: 1,
                                max_size: u64::MAX,
                            },
                        )
                        .await
                        .unwrap();

                    read_records.extend(records);
                }

                for (l, r) in batch_data.into_iter().zip(read_records.iter()) {
                    assert_eq!(l.tags, r.tags);
                    assert_eq!(l.key, r.key);
                    assert_eq!(l.data, r.data);
                }

                let tag = format!("task-{tid}");
                let tag_records = storage_adapter
                    .read_by_tag(
                        &namespace,
                        &shard_name,
                        0,
                        &tag,
                        &ReadConfig {
                            max_record_num: u64::MAX,
                            max_size: u64::MAX,
                        },
                    )
                    .await
                    .unwrap();

                assert_eq!(tag_records.len(), 100);

                for (l, r) in read_records.into_iter().zip(tag_records) {
                    assert_eq!(l.offset, r.offset);
                    assert_eq!(l.tags, r.tags);
                    assert_eq!(l.key, r.key);
                    assert_eq!(l.data, r.data);
                }
            });

            tasks.push(task);
        }

        future::join_all(tasks).await;

        for shard in shards.iter() {
            let len = storage_adapter
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
                .unwrap()
                .len();

            assert_eq!(len, (10000 / shards.len()) * 100);
        }

        for shard in shards.iter() {
            storage_adapter
                .delete_shard(&namespace, shard)
                .await
                .unwrap();
        }

        let list_res = storage_adapter.list_shard(&namespace, "").await.unwrap();

        assert_eq!(list_res.len(), 0);

        storage_adapter.close().await.unwrap();

        let _ = std::fs::remove_dir_all(&db_path);
    }
}
