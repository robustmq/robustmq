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

use std::sync::Arc;

use common_base::error::common::CommonError;
use rocksdb::{
    BoundColumnFamily, ColumnFamilyDescriptor, DBCompactionStyle, Options, SliceTransform, DB,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
pub mod engine;
pub mod warp;

#[derive(Debug)]
pub struct RocksDBEngine {
    pub db: Arc<DB>,
}

impl RocksDBEngine {
    /// Create a rocksdb instance
    pub fn new(data_path: &str, max_open_files: i32, cf_list: Vec<String>) -> Self {
        let opts: Options = Self::open_db_opts(max_open_files);
        let path = data_path.to_string();
        let mut cf_column_family = Vec::new();
        for cf in cf_list {
            cf_column_family.push(ColumnFamilyDescriptor::new(cf, Options::default()));
        }
        let instance = match DB::open_cf_descriptors(&opts, path, cf_column_family) {
            Ok(instance) => instance,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };

        RocksDBEngine {
            db: Arc::new(instance),
        }
    }

    /// Write the data serialization to RocksDB
    pub fn write<T: Serialize + std::fmt::Debug>(
        &self,
        cf: Arc<BoundColumnFamily>,
        key: &str,
        value: &T,
    ) -> Result<(), CommonError> {
        match serde_json::to_string(&value) {
            Ok(serialized) => {
                if let Err(e) = self.db.put_cf(&cf.clone(), key, serialized) {
                    return Err(CommonError::CommonError(format!(
                        "Failed to put to ColumnFamily:{:?}",
                        e
                    )));
                }
            }
            Err(err) => {
                return Err(CommonError::CommonError(format!(
                    "Failed to serialize to String. T: {:?}, err: {:?}",
                    value, err
                )))
            }
        }
        Ok(())
    }

    pub fn write_str(
        &self,
        cf: Arc<BoundColumnFamily>,
        key: &str,
        value: String,
    ) -> Result<(), CommonError> {
        self.write(cf, key, &value)
    }

    // Read data from the RocksDB
    pub fn read<T: DeserializeOwned>(
        &self,
        cf: Arc<BoundColumnFamily>,
        key: &str,
    ) -> Result<Option<T>, CommonError> {
        match self.db.get_cf(&cf, key) {
            Ok(Some(found)) => {
                if found.is_empty() {
                    return Ok(None);
                }

                match serde_json::from_slice::<T>(&found) {
                    Ok(t) => Ok(Some(t)),
                    Err(err) => Err(CommonError::CommonError(format!(
                        "Failed to deserialize: {:?}",
                        err
                    ))),
                }
            }
            Ok(None) => Ok(None),
            Err(err) => Err(CommonError::CommonError(format!(
                "Failed to get from ColumnFamily: {:?}",
                err
            ))),
        }
    }

    // Search data by prefix
    pub fn read_prefix(
        &self,
        cf: Arc<BoundColumnFamily>,
        search_key: &str,
    ) -> Result<Vec<(String, Vec<u8>)>, CommonError> {
        let mut iter = self.db.raw_iterator_cf(&cf);
        iter.seek(search_key);

        let mut result = Vec::new();
        while iter.valid() {
            if let Some(key) = iter.key() {
                if let Some(val) = iter.value() {
                    let key = String::from_utf8(key.to_vec())?;
                    if !key.starts_with(search_key) {
                        break;
                    }
                    result.push((key, val.to_vec()));
                }
            }

            iter.next();
        }
        Ok(result)
    }

    // Read all data in a ColumnFamily
    pub fn read_all_by_cf(
        &self,
        cf: Arc<BoundColumnFamily>,
    ) -> Result<Vec<(String, Vec<u8>)>, CommonError> {
        let mut iter = self.db.raw_iterator_cf(&cf);
        iter.seek_to_first();

        let mut result: Vec<(String, Vec<u8>)> = Vec::new();
        while iter.valid() {
            if let Some(key) = iter.key() {
                if let Some(val) = iter.value() {
                    let key = String::from_utf8(key.to_vec())?;
                    result.push((key, val.to_vec()));
                }
            }
            iter.next();
        }
        Ok(result)
    }

    pub fn delete(&self, cf: Arc<BoundColumnFamily>, key: &str) -> Result<(), CommonError> {
        Ok(self.db.delete_cf(&cf, key)?)
    }

    pub fn delete_prefix(
        &self,
        cf: Arc<BoundColumnFamily>,
        search_key: &str,
    ) -> Result<(), CommonError> {
        let mut iter = self.db.raw_iterator_cf(&cf);
        iter.seek(search_key);

        while iter.valid() {
            if let Some(key) = iter.key() {
                self.db.delete_cf(&cf, key)?
            }
            iter.next();
        }
        Ok(())
    }

    pub fn exist(&self, cf: Arc<BoundColumnFamily>, key: &str) -> bool {
        self.db.key_may_exist_cf(&cf, key)
    }

    pub fn cf_handle(&self, name: &str) -> Option<Arc<BoundColumnFamily>> {
        if let Some(cf) = self.db.cf_handle(name) {
            return Some(cf);
        }
        None
    }

    fn open_db_opts(max_open_files: i32) -> Options {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(max_open_files);
        opts.set_use_fsync(false);
        opts.set_bytes_per_sync(8388608);
        opts.optimize_for_point_lookup(1024);
        opts.set_table_cache_num_shard_bits(6);
        opts.set_max_write_buffer_number(32);
        opts.set_write_buffer_size(536870912);
        opts.set_target_file_size_base(1073741824);
        opts.set_min_write_buffer_number_to_merge(4);
        opts.set_level_zero_stop_writes_trigger(2000);
        opts.set_level_zero_slowdown_writes_trigger(0);
        opts.set_compaction_style(DBCompactionStyle::Universal);
        opts.set_disable_auto_compactions(true);

        let transform = SliceTransform::create_fixed_prefix(10);
        opts.set_prefix_extractor(transform);
        opts.set_memtable_prefix_bloom_ratio(0.2);

        opts
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use futures::future;
    use serde::{Deserialize, Serialize};
    use tokio::fs::{remove_dir, remove_dir_all};

    use super::RocksDBEngine;

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct User {
        pub name: String,
        pub age: u32,
    }

    fn cf_name() -> String {
        "cluster".to_string()
    }

    #[tokio::test]
    async fn multi_rocksdb_instance() {
        let config = placement_center_test_conf();

        let rs_handler = Arc::new(RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            vec![cf_name()],
        ));
        let mut tasks = Vec::new();
        for i in 1..100 {
            let rs = rs_handler.clone();
            let task = tokio::spawn(async move {
                let key = format!("name2{}", i);
                let name = format!("lobo{}", i);
                let user = User {
                    name: name.clone(),
                    age: 18,
                };
                let cf: std::sync::Arc<rocksdb::BoundColumnFamily<'_>> =
                    rs.cf_handle("cluster").unwrap();
                let res4 = rs.write(cf.clone(), &key, &user);
                assert!(res4.is_ok());
                let res1 = rs.read::<User>(cf.clone(), &key);
                let r = res1.unwrap();
                assert!(r.is_some());
                assert_eq!(r.unwrap().name, name);
                println!("spawn {}, key:{}", i, key);
                // sleep(Duration::from_secs(5)).await;
            });
            tasks.push(task);
        }

        // 等待所有任务完成
        let _ = future::join_all(tasks).await;

        remove_dir_all(config.rocksdb.data_path).await.unwrap();
    }

    #[tokio::test]
    async fn base_rw() {
        let config = placement_center_test_conf();

        let rs = RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            vec!["cluster".to_string()],
        );
        let cf = rs.cf_handle(&cf_name()).unwrap();

        let key = "name2";
        let res1 = rs.read::<User>(cf.clone(), key);
        assert!(res1.unwrap().is_none());

        let user = User {
            name: "lobo".to_string(),
            age: 18,
        };
        let res4 = rs.write(cf.clone(), key, &user);
        assert!(res4.is_ok());

        let res5 = rs.read::<User>(cf.clone(), key);
        let res5_rs = res5.unwrap().unwrap();
        assert!(res5_rs.name == user.name);
        assert!(res5_rs.age == user.age);

        let res6 = rs.delete(cf.clone(), key);
        assert!(res6.is_ok());

        match remove_dir(config.rocksdb.data_path).await {
            Ok(_) => {}
            Err(err) => {
                println!("{:?}", err)
            }
        }
    }

    #[tokio::test]
    async fn read_all() {
        let config = placement_center_test_conf();

        let rs = RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            vec!["cluster".to_string()],
        );

        let index = 66u64;
        let cf = rs.cf_handle(&cf_name()).unwrap();

        let key = "/raft/last_index".to_string();
        let _ = rs.write(cf.clone(), &key, &index);
        let res1 = rs.read::<u64>(cf.clone(), &key).unwrap().unwrap();
        assert_eq!(index, res1);

        let result = rs.read_all_by_cf(cf.clone()).unwrap();
        assert!(!result.is_empty());
        for raw in result.clone() {
            let raw_key = raw.0;
            let raw_val = raw.1;
            assert_eq!(raw_key, key);

            let val = String::from_utf8(raw_val).unwrap();
            assert_eq!(index.to_string(), val);

            let _ = rs.write_str(cf.clone(), &key, val);
            let res1 = rs.read::<String>(cf.clone(), &key).unwrap().unwrap();
            assert_eq!(index.to_string(), res1);
        }

        remove_dir_all(config.rocksdb.data_path).await.unwrap();
    }

    #[tokio::test]
    async fn read_prefix() {
        let config = placement_center_test_conf();

        let rs = RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            vec!["cluster".to_string()],
        );

        let cf = rs.cf_handle(&cf_name()).unwrap();

        rs.write_str(cf.clone(), "/v1/v1", "v11".to_string())
            .unwrap();
        rs.write_str(cf.clone(), "/v1/v2", "v12".to_string())
            .unwrap();
        rs.write_str(cf.clone(), "/v1/v3", "v13".to_string())
            .unwrap();
        rs.write_str(cf.clone(), "/v2/tmp_test/s1", "1".to_string())
            .unwrap();
        rs.write_str(cf.clone(), "/v2/tmp_test/s3", "2".to_string())
            .unwrap();
        rs.write_str(cf.clone(), "/v2/tmp_test/s2", "3".to_string())
            .unwrap();
        rs.write_str(cf.clone(), "/v3/tmp_test/s1", "1".to_string())
            .unwrap();
        rs.write_str(cf.clone().clone(), "/v3/tmp_test/s3", "2".to_string())
            .unwrap();
        rs.write_str(cf.clone(), "/v4/tmp_test/s2", "3".to_string())
            .unwrap();

        let result = rs.read_prefix(cf.clone(), "/v1").unwrap();
        assert_eq!(result.len(), 3);

        let result = rs.read_prefix(cf.clone(), "/v2").unwrap();
        assert_eq!(result.len(), 3);

        let result = rs.read_prefix(cf.clone(), "/v3").unwrap();
        assert_eq!(result.len(), 2);

        let result = rs.read_prefix(cf.clone(), "/v4").unwrap();
        assert_eq!(result.len(), 1);

        remove_dir_all(config.rocksdb.data_path).await.unwrap();
    }
}
