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

use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    sync::Arc,
};

use axum::async_trait;
use common_base::error::common::CommonError;
use metadata_struct::adapter::{read_config::ReadConfig, record::Record};
use rocksdb_engine::RocksDBEngine;

use crate::storage::{ShardConfig, ShardOffset, StorageAdapter};

const DB_COLUMN_FAMILY: &str = "db";

fn column_family_list() -> Vec<String> {
    vec![DB_COLUMN_FAMILY.to_string()]
}

pub struct RocksDBStorageAdapter {
    pub db: Arc<RocksDBEngine>,
}

impl RocksDBStorageAdapter {
    pub fn new(db_path: &str, max_open_files: i32) -> Self {
        RocksDBStorageAdapter {
            db: Arc::new(RocksDBEngine::new(
                db_path,
                max_open_files,
                column_family_list(),
            )),
        }
    }

    #[inline(always)]
    pub fn shard_record_key<S1: Display>(
        &self,
        namespace: S1,
        shard: S1,
        record_offset: u64,
    ) -> String {
        format!("/{}/{}/record/{}", namespace, shard, record_offset)
    }

    #[inline(always)]
    pub fn shard_offset_key<S1: Display>(&self, namespace: &S1, shard: &S1) -> String {
        format!("/offset/{}/{}", namespace, shard)
    }

    #[inline(always)]
    pub fn key_offset_key<S1: Display>(&self, namespace: &S1, shard: &S1, key: &S1) -> String {
        format!("/key/{}/{}/{}", namespace, shard, key)
    }

    #[inline(always)]
    pub fn tag_offsets_key<S1: Display>(&self, namespace: &S1, shard: &S1, tag: &S1) -> String {
        format!("/tag/{}/{}/{}", namespace, shard, tag)
    }

    #[inline(always)]
    pub fn group_record_offsets_key<S1: Display>(&self, group: &S1) -> String {
        format!("/group/{}", group)
    }
}

#[async_trait]
impl StorageAdapter for RocksDBStorageAdapter {
    /// create a shard by inserting an offset 0
    async fn create_shard(
        &self,
        namespace: String,
        shard_name: String,
        _: ShardConfig,
    ) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let shard_offset_key = self.shard_offset_key(&namespace, &shard_name);

        // check whether the shard exists
        if self
            .db
            .read::<u64>(cf.clone(), shard_offset_key.as_str())?
            .is_some()
        {
            return Err(CommonError::CommonError(format!(
                "shard {} under namespace {} already exists",
                shard_name, namespace
            )));
        }

        self.db.write(cf, shard_offset_key.as_str(), &0_u64)
    }

    async fn delete_shard(&self, namespace: String, shard_name: String) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let shard_offset_key = self.shard_offset_key(&namespace, &shard_name);

        // check whether the shard exists
        if self
            .db
            .read::<u64>(cf.clone(), shard_offset_key.as_str())?
            .is_none()
        {
            return Err(CommonError::CommonError(format!(
                "shard {} under namespace {} not exists",
                shard_name, namespace
            )));
        }

        self.db.delete(cf, &shard_offset_key)
    }

    async fn write(
        &self,
        namespace: String,
        shard_name: String,
        mut message: Record,
    ) -> Result<u64, CommonError> {
        // check whether the key exists
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let key_offset_key = self.key_offset_key(&namespace, &shard_name, &message.key);

        if self.db.read::<u64>(cf.clone(), &key_offset_key)?.is_some() {
            return Err(CommonError::CommonError(format!(
                "key {} already exists in shard {} under {}",
                &message.key, &shard_name, &namespace
            )));
        }

        // read the offset
        let shard_offset_key = self.shard_offset_key(&namespace, &shard_name);
        let offset = match self.db.read::<u64>(cf.clone(), shard_offset_key.as_str())? {
            Some(offset) => offset,
            None => {
                return Err(CommonError::CommonError(format!(
                    "shard {} under {} not exists",
                    shard_name, namespace
                )));
            }
        };

        // write the message
        message.offset = Some(offset);

        let shard_record_key = self.shard_record_key(namespace.clone(), shard_name.clone(), offset);
        self.db.write(cf.clone(), &shard_record_key, &message)?;

        // update the shard offset
        self.db
            .write(cf.clone(), shard_offset_key.as_str(), &(offset + 1))?;

        // update the key offset
        self.db.write(cf, key_offset_key.as_str(), &offset)?;

        Ok(offset)
    }

    async fn batch_write(
        &self,
        namespace: String,
        shard_name: String,
        messages: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        // check whether key exists
        for message in messages.iter() {
            if message.key.is_empty() {
                continue;
            }

            let key_offset_key = self.key_offset_key(&namespace, &shard_name, &message.key);

            if self.db.read::<u64>(cf.clone(), &key_offset_key)?.is_some() {
                return Err(CommonError::CommonError(format!(
                    "key {} already exists in shard {} under {}",
                    &message.key, &shard_name, &namespace
                )));
            }
        }

        // check duplicate key exists in messages
        let mut key_set = HashSet::new();
        for message in messages.iter() {
            if !&message.key.is_empty() && !key_set.insert(&message.key) {
                return Err(CommonError::CommonError(format!(
                    "duplicate key {} in messages",
                    message.key
                )));
            }
        }

        // get the starting shard offset
        let shard_offset_key = self.shard_offset_key(&namespace, &shard_name);
        let offset = match self.db.read::<u64>(cf.clone(), shard_offset_key.as_str())? {
            Some(offset) => offset,
            None => {
                return Err(CommonError::CommonError(format!(
                    "shard {} under {} not exists",
                    shard_name, namespace
                )));
            }
        };

        let mut start_offset = offset;

        let mut offset_res = Vec::new();

        for mut msg in messages {
            offset_res.push(start_offset);
            msg.offset = Some(start_offset);

            // write the shard record
            let shard_record_key = self.shard_record_key(&namespace, &shard_name, start_offset);
            self.db.write(cf.clone(), &shard_record_key, &msg)?;

            // write the key offset
            let key_offset_key = self.key_offset_key(&namespace, &shard_name, &msg.key);
            self.db
                .write(cf.clone(), key_offset_key.as_str(), &start_offset)?;

            start_offset += 1;
        }

        // update the shard offset
        self.db
            .write(cf, shard_offset_key.as_str(), &start_offset)?;

        Ok(offset_res)
    }

    async fn read_by_offset(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let mut start_offset = offset;
        let mut record = Vec::new();

        // read records with indices from offset to offset + max_record_num
        // stop when the record is not found
        for _ in 0..read_config.max_record_num {
            let key = self.shard_record_key(&namespace, &shard_name, start_offset);
            let value = self.db.read::<Record>(cf.clone(), &key)?;

            match value {
                Some(value) => {
                    record.push(value);
                    start_offset += 1;
                }
                None => {
                    break;
                }
            }
        }

        Ok(record)
    }

    async fn read_by_tag(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        tag: String,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        // find all record offsets with the given tag, the offsets are sorted in ascending order
        let tag_offsets_key = self.tag_offsets_key(&namespace, &shard_name, &tag);

        let tag_offsets = self
            .db
            .read::<Vec<u64>>(cf.clone(), &tag_offsets_key)?
            .unwrap_or(Vec::new());

        // only keep offsets >= offset and at most read_config.max_record_num
        let retained_offsets: Vec<u64> = tag_offsets
            .into_iter()
            .filter(|&x| x >= offset)
            .take(read_config.max_record_num as usize)
            .collect();

        let mut records = Vec::new();

        for offset in retained_offsets {
            let shard_record_key = self.shard_record_key(&namespace, &shard_name, offset);
            let record = self
                .db
                .read::<Record>(cf.clone(), &shard_record_key)?
                .unwrap();

            records.push(record);
        }

        Ok(records)
    }

    async fn read_by_key(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        key: String,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let key_offset_key = self.key_offset_key(&namespace, &shard_name, &key);

        match self.db.read::<u64>(cf.clone(), &key_offset_key)? {
            Some(key_offset) if key_offset >= offset && read_config.max_record_num >= 1 => {
                let shard_record_key = self.shard_record_key(&namespace, &shard_name, offset);
                let record = self
                    .db
                    .read::<Record>(cf.clone(), &shard_record_key)?
                    .unwrap();

                return Ok(vec![record]);
            }
            _ => return Ok(Vec::new()),
        };
    }

    async fn get_offset_by_timestamp(
        &self,
        namespace: String,
        shard_name: String,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let mut start_offset = 0;

        loop {
            let shard_record_key = self.shard_record_key(&namespace, &shard_name, start_offset);
            let value = self.db.read::<Record>(cf.clone(), &shard_record_key)?;

            match value {
                Some(value) if value.timestamp >= timestamp => {
                    return Ok(Some(ShardOffset {
                        offset: start_offset,
                        ..Default::default()
                    }));
                }
                Some(_) => {
                    start_offset += 1;
                }
                None => {
                    break;
                }
            }
        }

        Ok(None)
    }

    async fn get_offset_by_group(
        &self,
        group_name: String,
    ) -> Result<Vec<ShardOffset>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let group_record_offsets_key = self.group_record_offsets_key(&group_name);

        let offsets = self
            .db
            .read::<HashMap<String, u64>>(cf.clone(), &group_record_offsets_key)?
            .unwrap_or(HashMap::new())
            .into_values()
            .map(|offset| ShardOffset {
                offset,
                ..Default::default()
            })
            .collect::<Vec<_>>();

        Ok(offsets)
    }

    async fn commit_offset(
        &self,
        group_name: String,
        namespace: String,
        offsets: HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY).unwrap();

        let group_record_offsets_key = self.group_record_offsets_key(&group_name);

        let mut shard_offset_pairs = self
            .db
            .read::<HashMap<String, u64>>(cf.clone(), &group_record_offsets_key)?
            .unwrap_or(HashMap::new());

        for (shard_name, offset) in offsets {
            let key = format!("{}_{}", &namespace, &shard_name);
            shard_offset_pairs.insert(key, offset);
        }

        self.db
            .write(cf, group_record_offsets_key.as_str(), &shard_offset_pairs)?;

        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use common_base::tools::unique_id;
    use metadata_struct::adapter::{read_config::ReadConfig, record::Record};

    use crate::storage::{ShardConfig, StorageAdapter};

    use super::RocksDBStorageAdapter;
    #[tokio::test]
    async fn stream_read_write() {
        let db_path = format!("/tmp/robustmq_{}", unique_id());

        let storage_adapter = RocksDBStorageAdapter::new(db_path.as_str(), 100);
        let namespace = unique_id();
        let shard_name = "test-11".to_string();

        // step 1: create shard
        storage_adapter
            .create_shard(
                namespace.clone(),
                shard_name.clone(),
                ShardConfig::default(),
            )
            .await
            .unwrap();

        // insert two records (no key or tag) into the shard
        let ms1 = "test1".to_string();
        let ms2 = "test2".to_string();
        let data = vec![
            Record::build_byte(ms1.clone().as_bytes().to_vec()),
            Record::build_byte(ms2.clone().as_bytes().to_vec()),
        ];

        let result = storage_adapter
            .batch_write(namespace.clone(), shard_name.clone(), data)
            .await
            .unwrap();

        assert_eq!(result.first().unwrap().clone(), 0);
        assert_eq!(result.get(1).unwrap().clone(), 1);

        // read previous records
        assert_eq!(
            storage_adapter
                .read_by_offset(
                    namespace.clone(),
                    shard_name.clone(),
                    0,
                    ReadConfig {
                        max_record_num: 10,
                        max_size: 1024,
                    }
                )
                .await
                .unwrap()
                .len(),
            2
        );

        // insert two other records (no key or tag) into the shard
        let ms3 = "test3".to_string();
        let ms4 = "test4".to_string();
        let data = vec![
            Record::build_byte(ms3.clone().as_bytes().to_vec()),
            Record::build_byte(ms4.clone().as_bytes().to_vec()),
        ];

        let result = storage_adapter
            .batch_write(namespace.clone(), shard_name.clone(), data)
            .await
            .unwrap();

        // read from offset 2
        let result_read = storage_adapter
            .read_by_offset(
                namespace.clone(),
                shard_name.clone(),
                2,
                ReadConfig {
                    max_record_num: 10,
                    max_size: 1024,
                },
            )
            .await
            .unwrap();

        assert_eq!(result.first().unwrap().clone(), 2);
        assert_eq!(result.get(1).unwrap().clone(), 3);
        assert_eq!(result_read.len(), 2);

        // test group functionalities
        let group_id = "test_group_id".to_string();
        let read_config = ReadConfig {
            max_record_num: 1,
            ..Default::default()
        };

        // read m1
        let offset = 0;
        let res = storage_adapter
            .read_by_offset(
                namespace.clone(),
                shard_name.clone(),
                offset,
                read_config.clone(),
            )
            .await
            .unwrap();

        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms1
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().clone().offset.unwrap(),
        );

        storage_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // read ms2
        let offset = storage_adapter
            .get_offset_by_group(group_id.clone())
            .await
            .unwrap();

        let res = storage_adapter
            .read_by_offset(
                namespace.clone(),
                shard_name.clone(),
                offset.first().unwrap().offset + 1,
                read_config.clone(),
            )
            .await
            .unwrap();

        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms2
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().clone().offset.unwrap(),
        );
        storage_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // read m3
        let offset: Vec<crate::storage::ShardOffset> = storage_adapter
            .get_offset_by_group(group_id.clone())
            .await
            .unwrap();

        let res = storage_adapter
            .read_by_offset(
                namespace.clone(),
                shard_name.clone(),
                offset.first().unwrap().offset + 1,
                read_config.clone(),
            )
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms3
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().clone().offset.unwrap(),
        );
        storage_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // read m4
        let offset = storage_adapter
            .get_offset_by_group(group_id.clone())
            .await
            .unwrap();

        let res = storage_adapter
            .read_by_offset(
                namespace.clone(),
                shard_name.clone(),
                offset.first().unwrap().offset + 1,
                read_config.clone(),
            )
            .await
            .unwrap();
        assert_eq!(
            String::from_utf8(res.first().unwrap().clone().data).unwrap(),
            ms4
        );

        let mut offset_data = HashMap::new();
        offset_data.insert(
            shard_name.clone(),
            res.first().unwrap().clone().offset.unwrap(),
        );
        storage_adapter
            .commit_offset(group_id.clone(), namespace.clone(), offset_data)
            .await
            .unwrap();

        // delete shard
        storage_adapter
            .delete_shard(namespace, shard_name)
            .await
            .unwrap();

        let _ = std::fs::remove_dir_all(&db_path);
    }
}
