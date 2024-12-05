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

// use std::fmt::Display;
// use std::sync::Arc;

// use axum::async_trait;
// use common_base::error::common::CommonError;
// use metadata_struct::adapter::record::Record;
// use rocksdb_engine::RocksDBEngine;

// use crate::storage::{ShardConfig, StorageAdapter};

// const DB_COLUMN_FAMILY_KV: &str = "kv";
// const DB_COLUMN_FAMILY_RECORD: &str = "record";

// fn column_family_list() -> Vec<String> {
//     vec![
//         DB_COLUMN_FAMILY_KV.to_string(),
//         DB_COLUMN_FAMILY_RECORD.to_string(),
//     ]
// }

// #[derive(Clone)]
// pub struct RocksDBStorageAdapter {
//     pub db: Arc<RocksDBEngine>,
// }

// impl RocksDBStorageAdapter {
//     pub fn new(db_path: &str, max_open_files: i32) -> Self {
//         RocksDBStorageAdapter {
//             db: Arc::new(RocksDBEngine::new(
//                 db_path,
//                 max_open_files,
//                 column_family_list(),
//             )),
//         }
//     }

//     #[inline(always)]
//     pub fn record_key<S1: Display>(&self, namespace: S1, shard_name: S1, index: u128) -> String {
//         format!("{}_{}_record_{}", namespace, shard_name, index)
//     }

//     #[inline(always)]
//     pub fn offset_shard_key<S1: Display>(&self, namespace: S1, shard_name: S1) -> String {
//         format!("{}_{}_{}", namespace, shard_name, "shard_offset")
//     }

//     #[inline(always)]
//     pub fn offset_key<S1: Display, S2: Display>(
//         &self,
//         namespace: S1,
//         shard_name: S1,
//         group_id: S2,
//     ) -> String {
//         format!("{}_{}_{}", namespace, shard_name, group_id)
//     }

//     pub fn get_offset<S1: Display, S2: Display>(
//         &self,
//         namespace: S1,
//         shard_name: S1,
//         group_id: S2,
//     ) -> Option<u128> {
//         let key = self.offset_key(namespace, shard_name, group_id);
//         self.db
//             .cf_handle(DB_COLUMN_FAMILY_KV)
//             .and_then(|cf| self.db.read(cf, &key).ok()?)
//     }
// }

// impl RocksDBStorageAdapter {}

// #[async_trait]
// impl StorageAdapter for RocksDBStorageAdapter {
//     async fn create_shard(
//         &self,
//         namespace: String,
//         shard_name: String,
//         _: ShardConfig,
//     ) -> Result<(), CommonError> {
//         let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
//         let key = self.offset_shard_key(namespace, shard_name);
//         self.db.write(cf, key.as_str(), &0_u128)
//     }

//     async fn delete_shard(&self, namespace: String, shard_name: String) -> Result<(), CommonError> {
//         let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
//         let key = self.offset_shard_key(namespace, shard_name);
//         self.db.delete_prefix(cf, &key)
//     }

//     async fn write(
//         &self,
//         namespace: String,
//         shard_name: String,
//         message: Vec<Record>,
//     ) -> Result<Vec<usize>, CommonError> {
//         let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
//         let key_shard_offset = self.offset_shard_key(namespace.clone(), shard_name.clone());
//         let offset = self
//             .db
//             .read::<u128>(cf, key_shard_offset.as_str())?
//             .unwrap_or(0);

//         let mut start_offset = offset;

//         let mut offset_res = Vec::new();
//         for mut msg in message {
//             offset_res.push(start_offset as usize);
//             msg.offset = start_offset;

//             let record_key = self.record_key(namespace.clone(), shard_name.clone(), start_offset);
//             self.db.write(cf, &record_key, &msg)?;
//             start_offset += 1;
//         }

//         self.db
//             .write(cf, key_shard_offset.as_str(), &start_offset)?;

//         Ok(offset_res)
//     }

//     async fn stream_read(
//         &self,
//         namespace: String,
//         shard_name: String,
//         group_id: String,
//         record_num: Option<u128>,
//         _: Option<usize>,
//     ) -> Result<Option<Vec<Record>>, CommonError> {
//         let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
//         let group_offset_key = self.offset_key(namespace.clone(), shard_name.clone(), group_id);
//         let offset = self
//             .db
//             .read::<u128>(cf, group_offset_key.as_str())?
//             .unwrap_or(0);

//         let num = record_num.unwrap_or(10);

//         let mut cur_offset = 0;
//         let mut result = Vec::new();
//         for i in offset..(offset + num) {
//             let record_key = self.record_key(namespace.clone(), shard_name.clone(), i);
//             let value = self.db.read::<Record>(cf, &record_key)?;

//             if let Some(value) = value {
//                 result.push(value.clone());
//                 cur_offset += 1;
//             } else {
//                 break;
//             }
//         }

//         if cur_offset > 0 {
//             self.db
//                 .write(cf, group_offset_key.as_str(), &(offset + cur_offset))?;
//         }
//         Ok(Some(result))
//     }

//     async fn stream_commit_offset(
//         &self,
//         namespace: String,
//         shard_name: String,
//         group_id: String,
//         offset: u128,
//     ) -> Result<bool, CommonError> {
//         let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
//         let key = self.offset_key(namespace, group_id, shard_name);
//         self.db.write(cf, key.as_str(), &offset)?;
//         Ok(true)
//     }

//     async fn stream_read_by_offset(
//         &self,
//         namespace: String,
//         shard_name: String,
//         offset: usize,
//     ) -> Result<Option<Record>, CommonError> {
//         let cf = self.db.cf_handle(DB_COLUMN_FAMILY_RECORD).unwrap();
//         self.db.read::<Record>(
//             cf,
//             self.record_key(namespace, shard_name, offset as u128)
//                 .as_str(),
//         )
//     }

//     async fn stream_read_by_timestamp(
//         &self,
//         _: String,
//         _: String,
//         _: u128,
//         _: u128,
//         _: Option<usize>,
//         _: Option<usize>,
//     ) -> Result<Option<Vec<Record>>, CommonError> {
//         Ok(None)
//     }

//     async fn stream_read_by_key(
//         &self,
//         _: String,
//         _: String,
//         _: String,
//     ) -> Result<Option<Record>, CommonError> {
//         Ok(None)
//     }
//     async fn close(&self) -> Result<(), CommonError> {
//         Ok(())
//     }
// }

// #[cfg(test)]
// mod tests {
//     use common_base::tools::unique_id;
//     use metadata_struct::adapter::record::Record;

//     use super::RocksDBStorageAdapter;
//     use crate::storage::StorageAdapter;
//     #[tokio::test]
//     async fn stream_read_write() {
//         let db_path = format!("/tmp/robustmq_{}", unique_id());

//         let storage_adapter = RocksDBStorageAdapter::new(db_path.as_str(), 100);
//         let namespace = unique_id();
//         let shard_name = "test-11".to_string();
//         let ms1 = "test1".to_string();
//         let ms2 = "test2".to_string();
//         let data = vec![
//             Record::build_b(ms1.clone().as_bytes().to_vec()),
//             Record::build_b(ms2.clone().as_bytes().to_vec()),
//         ];

//         let result = storage_adapter
//             .stream_write(namespace.clone(), shard_name.clone(), data)
//             .await
//             .unwrap();

//         assert_eq!(result.first().unwrap().clone(), 0);
//         assert_eq!(result.get(1).unwrap().clone(), 1);
//         assert_eq!(
//             storage_adapter
//                 .stream_read(
//                     namespace.clone(),
//                     shard_name.clone(),
//                     "test_".to_string(),
//                     Some(10),
//                     None
//                 )
//                 .await
//                 .unwrap()
//                 .unwrap()
//                 .len(),
//             2
//         );

//         let ms3 = "test3".to_string();
//         let ms4 = "test4".to_string();
//         let data = vec![
//             Record::build_b(ms3.clone().as_bytes().to_vec()),
//             Record::build_b(ms4.clone().as_bytes().to_vec()),
//         ];

//         let result = storage_adapter
//             .stream_write(namespace.clone(), shard_name.clone(), data)
//             .await
//             .unwrap();
//         let result_read = storage_adapter
//             .stream_read(
//                 namespace.clone(),
//                 shard_name.clone(),
//                 "test_".to_string(),
//                 Some(10),
//                 None,
//             )
//             .await
//             .unwrap();
//         println!("{:?}", result_read);

//         assert_eq!(result.first().unwrap().clone(), 2);
//         assert_eq!(result.get(1).unwrap().clone(), 3);
//         assert_eq!(result_read.unwrap().len(), 2);

//         let group_id = "test_group_id".to_string();
//         let record_num = Some(1);
//         let record_size = None;
//         let res = storage_adapter
//             .stream_read(
//                 namespace.clone(),
//                 shard_name.clone(),
//                 group_id.clone(),
//                 record_num,
//                 record_size,
//             )
//             .await
//             .unwrap()
//             .unwrap();
//         assert_eq!(
//             String::from_utf8(res.first().unwrap().clone().data).unwrap(),
//             ms1
//         );
//         storage_adapter
//             .stream_commit_offset(
//                 namespace.clone(),
//                 shard_name.clone(),
//                 group_id.clone(),
//                 res.first().unwrap().clone().offset,
//             )
//             .await
//             .unwrap();

//         let res = storage_adapter
//             .stream_read(
//                 namespace.clone(),
//                 shard_name.clone(),
//                 group_id.clone(),
//                 record_num,
//                 record_size,
//             )
//             .await
//             .unwrap()
//             .unwrap();
//         assert_eq!(
//             String::from_utf8(res.first().unwrap().clone().data).unwrap(),
//             ms2
//         );
//         storage_adapter
//             .stream_commit_offset(
//                 namespace.clone(),
//                 shard_name.clone(),
//                 group_id.clone(),
//                 res.first().unwrap().clone().offset,
//             )
//             .await
//             .unwrap();

//         let res = storage_adapter
//             .stream_read(
//                 namespace.clone(),
//                 shard_name.clone(),
//                 group_id.clone(),
//                 record_num,
//                 record_size,
//             )
//             .await
//             .unwrap()
//             .unwrap();
//         assert_eq!(
//             String::from_utf8(res.first().unwrap().clone().data).unwrap(),
//             ms3
//         );
//         storage_adapter
//             .stream_commit_offset(
//                 namespace.clone(),
//                 shard_name.clone(),
//                 group_id.clone(),
//                 res.first().unwrap().clone().offset,
//             )
//             .await
//             .unwrap();

//         let res = storage_adapter
//             .stream_read(
//                 namespace.clone(),
//                 shard_name.clone(),
//                 group_id.clone(),
//                 record_num,
//                 record_size,
//             )
//             .await
//             .unwrap()
//             .unwrap();
//         assert_eq!(
//             String::from_utf8(res.first().unwrap().clone().data).unwrap(),
//             ms4
//         );
//         storage_adapter
//             .stream_commit_offset(
//                 namespace.clone(),
//                 shard_name.clone(),
//                 group_id.clone(),
//                 res.first().unwrap().clone().offset,
//             )
//             .await
//             .unwrap();

//         let _ = std::fs::remove_dir_all(&db_path);
//     }
// }
