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

use axum::async_trait;
use common_base::error::common::CommonError;
use metadata_struct::adapter::record::Record;
use rocksdb_engine::RocksDBEngine;
use std::{fmt::Display, sync::Arc};

use crate::storage::{ShardConfig, StorageAdapter};

const DB_COLUMN_FAMILY_KV: &str = "kv";
const DB_COLUMN_FAMILY_RECORD: &str = "record";

fn column_family_list() -> Vec<String> {
    return vec![
        DB_COLUMN_FAMILY_KV.to_string(),
        DB_COLUMN_FAMILY_RECORD.to_string(),
    ];
}

#[derive(Clone)]
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
    pub fn record_key<S1: Display>(&self, shard_name: S1, index: u128) -> String {
        format!("{}_record_{}", shard_name, index)
    }

    #[inline(always)]
    pub fn offset_shard_key<S1: Display>(&self, shard_name: S1) -> String {
        return format!("{}_{}", shard_name, "shard_offset");
    }

    #[inline(always)]
    pub fn offset_key<S1: Display, S2: Display>(&self, shard_name: S1, group_id: S2) -> String {
        return format!("{}_{}", shard_name, group_id);
    }

    pub fn get_offset<S1: Display, S2: Display>(
        &self,
        shard_name: S1,
        group_id: S2,
    ) -> Option<u128> {
        let key = self.offset_key(shard_name, group_id);
        self.db
            .cf_handle(DB_COLUMN_FAMILY_KV)
            .and_then(|cf| self.db.read(cf, &key).ok()?)
    }
}

impl RocksDBStorageAdapter {}

#[async_trait]
impl StorageAdapter for RocksDBStorageAdapter {
    async fn create_shard(&self, shard_name: String, _: ShardConfig) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
        let key = self.offset_shard_key(shard_name);
        self.db
            .write(cf, key.as_str(), &0_u128)
            .map_err(CommonError::CommmonError)
    }

    async fn delete_shard(&self, shard_name: String) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
        self.db.delete_prefix(cf, shard_name.as_str())
    }

    async fn set(&self, key: String, value: Record) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY_KV).unwrap();
        self.db
            .write(cf, key.as_str(), &value)
            .map_err(CommonError::CommmonError)
    }
    async fn get(&self, key: String) -> Result<Option<Record>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY_KV).unwrap();
        self.db.read(cf, &key).map_err(CommonError::CommmonError)
    }
    async fn delete(&self, key: String) -> Result<(), CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY_KV).unwrap();
        self.db.delete(cf, key.as_str())
    }
    async fn exists(&self, key: String) -> Result<bool, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY_KV).unwrap();
        Ok(self.db.exist(cf, key.as_str()))
    }

    async fn stream_write(
        &self,
        shard_name: String,
        message: Vec<Record>,
    ) -> Result<Vec<usize>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
        let key_shard_offset = self.offset_shard_key(&shard_name);
        let offset = self
            .db
            .read::<u128>(cf, key_shard_offset.as_str())
            .map_err(CommonError::CommmonError)?
            .unwrap_or(0);

        let mut start_offset = offset;

        let mut offset_res = Vec::new();
        for mut msg in message {
            offset_res.push(start_offset as usize);
            msg.offset = start_offset;

            self.db
                .write(
                    cf,
                    format!("{}_record_{}", shard_name, start_offset).as_str(),
                    &msg,
                )
                .map_err(CommonError::CommmonError)?;
            start_offset += 1;
        }

        self.db
            .write(cf, key_shard_offset.as_str(), &start_offset)
            .map_err(CommonError::CommmonError)?;

        return Ok(offset_res);
    }

    async fn stream_read(
        &self,
        shard_name: String,
        group_id: String,
        record_num: Option<u128>,
        _: Option<usize>,
    ) -> Result<Option<Vec<Record>>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
        let group_offset_key = self.offset_key(shard_name.clone(), group_id);
        let offset = self
            .db
            .read::<u128>(cf, group_offset_key.as_str())
            .map_err(CommonError::CommmonError)?
            .unwrap_or(0);

        let num = if let Some(num) = record_num { num } else { 10 };

        let mut cur_offset = 0;
        let mut result = Vec::new();
        for i in offset..(offset + num) {
            let value = self
                .db
                .read::<Record>(cf, self.record_key(&shard_name, i).as_str())
                .map_err(CommonError::CommmonError)?;

            if let Some(value) = value {
                result.push(value.clone());
                cur_offset += 1;
            } else {
                break;
            }
        }

        if cur_offset > 0 {
            self.db
                .write(cf, group_offset_key.as_str(), &(offset + cur_offset))
                .map_err(CommonError::CommmonError)?;
        }
        return Ok(Some(result));
    }

    async fn stream_commit_offset(
        &self,
        shard_name: String,
        group_id: String,
        offset: u128,
    ) -> Result<bool, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
        let key = self.offset_key(group_id, shard_name);
        self.db
            .write(cf, key.as_str(), &offset)
            .map_err(CommonError::CommmonError)?;
        return Ok(true);
    }

    async fn stream_read_by_offset(
        &self,
        shard_name: String,
        offset: usize,
    ) -> Result<Option<Record>, CommonError> {
        let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
        self.db
            .read::<Record>(cf, self.record_key(shard_name, offset as u128).as_str())
            .map_err(CommonError::CommmonError)
    }

    async fn stream_read_by_timestamp(
        &self,
        _: String,
        _: u128,
        _: u128,
        _: Option<usize>,
        _: Option<usize>,
    ) -> Result<Option<Vec<Record>>, CommonError> {
        return Ok(None);
    }

    async fn stream_read_by_key(
        &self,
        _: String,
        _: String,
    ) -> Result<Option<Record>, CommonError> {
        return Ok(None);
    }
}

#[cfg(test)]
mod tests {
    use super::RocksDBStorageAdapter;
    use crate::storage::StorageAdapter;
    use common_base::tools::unique_id;
    use metadata_struct::adapter::record::Record;
    #[tokio::test]
    async fn stream_read_write() {
        let db_path = format!("/tmp/robustmq_{}", unique_id());

        let storage_adapter = RocksDBStorageAdapter::new(db_path.as_str(), 100);
        let shard_name = "test-11".to_string();
        let ms1 = "test1".to_string();
        let ms2 = "test2".to_string();
        let data = vec![
            Record::build_b(ms1.clone().as_bytes().to_vec()),
            Record::build_b(ms2.clone().as_bytes().to_vec()),
        ];

        let result = storage_adapter
            .stream_write(shard_name.clone(), data)
            .await
            .unwrap();

        assert_eq!(result.get(0).unwrap().clone(), 0);
        assert_eq!(result.get(1).unwrap().clone(), 1);
        assert_eq!(
            storage_adapter
                .stream_read(shard_name.clone(), "test_".to_string(), Some(10), None)
                .await
                .unwrap()
                .unwrap()
                .len(),
            2
        );

        let ms3 = "test3".to_string();
        let ms4 = "test4".to_string();
        let data = vec![
            Record::build_b(ms3.clone().as_bytes().to_vec()),
            Record::build_b(ms4.clone().as_bytes().to_vec()),
        ];

        let result = storage_adapter
            .stream_write(shard_name.clone(), data)
            .await
            .unwrap();
        let result_read = storage_adapter
            .stream_read(shard_name.clone(), "test_".to_string(), Some(10), None)
            .await
            .unwrap();
        println!("{:?}", result_read);

        assert_eq!(result.get(0).unwrap().clone(), 2);
        assert_eq!(result.get(1).unwrap().clone(), 3);
        assert_eq!(result_read.unwrap().len(), 2);

        let group_id = "test_group_id".to_string();
        let record_num = Some(1);
        let record_size = None;
        let res = storage_adapter
            .stream_read(
                shard_name.clone(),
                group_id.clone(),
                record_num,
                record_size,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            String::from_utf8(res.get(0).unwrap().clone().data).unwrap(),
            ms1
        );
        storage_adapter
            .stream_commit_offset(
                shard_name.clone(),
                group_id.clone(),
                res.get(0).unwrap().clone().offset,
            )
            .await
            .unwrap();

        let res = storage_adapter
            .stream_read(
                shard_name.clone(),
                group_id.clone(),
                record_num,
                record_size,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            String::from_utf8(res.get(0).unwrap().clone().data).unwrap(),
            ms2
        );
        storage_adapter
            .stream_commit_offset(
                shard_name.clone(),
                group_id.clone(),
                res.get(0).unwrap().clone().offset,
            )
            .await
            .unwrap();

        let res = storage_adapter
            .stream_read(
                shard_name.clone(),
                group_id.clone(),
                record_num,
                record_size,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            String::from_utf8(res.get(0).unwrap().clone().data).unwrap(),
            ms3
        );
        storage_adapter
            .stream_commit_offset(
                shard_name.clone(),
                group_id.clone(),
                res.get(0).unwrap().clone().offset,
            )
            .await
            .unwrap();

        let res = storage_adapter
            .stream_read(
                shard_name.clone(),
                group_id.clone(),
                record_num,
                record_size,
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(
            String::from_utf8(res.get(0).unwrap().clone().data).unwrap(),
            ms4
        );
        storage_adapter
            .stream_commit_offset(
                shard_name.clone(),
                group_id.clone(),
                res.get(0).unwrap().clone().offset,
            )
            .await
            .unwrap();

        let _ = std::fs::remove_dir_all(&db_path);
    }
}
