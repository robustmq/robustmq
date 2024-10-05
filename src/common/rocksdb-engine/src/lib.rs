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

use common_base::error::common::CommonError;
use log::error;
use rocksdb::{ColumnFamily, DBCompactionStyle, Options, DB};
use rocksdb::{ColumnFamilyDescriptor, SliceTransform};
use serde::{de::DeserializeOwned, Serialize};
use serde_json;
use std::collections::HashMap;

#[derive(Debug)]
pub struct RocksDBEngine {
    pub db: DB,
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

        return RocksDBEngine { db: instance };
    }

    /// Write the data serialization to RocksDB
    pub fn write<T: Serialize + std::fmt::Debug>(
        &self,
        cf: &ColumnFamily,
        key: &str,
        value: &T,
    ) -> Result<(), String> {
        match serde_json::to_string(&value) {
            Ok(serialized) => self
                .db
                .put_cf(cf, key, serialized.into_bytes())
                .map_err(|err| format!("Failed to put to ColumnFamily:{:?}", err)),
            Err(err) => Err(format!(
                "Failed to serialize to String. T: {:?}, err: {:?}",
                value, err
            )),
        }
    }

    pub fn write_str(&self, cf: &ColumnFamily, key: &str, value: String) -> Result<(), String> {
        self.db
            .put_cf(cf, key, value.into_bytes())
            .map_err(|err| format!("Failed to put to ColumnFamily:{:?}", err))
    }

    // Read data from the RocksDB
    pub fn read<T: DeserializeOwned>(
        &self,
        cf: &ColumnFamily,
        key: &str,
    ) -> Result<Option<T>, String> {
        match self.db.get_cf(cf, key) {
            Ok(opt) => match opt {
                Some(found) => match String::from_utf8(found) {
                    Ok(s) => match serde_json::from_str::<T>(&s) {
                        Ok(t) => Ok(Some(t)),
                        Err(err) => Err(format!("Failed to deserialize: {:?}", err)),
                    },
                    Err(err) => Err(format!("Failed to deserialize: {:?}", err)),
                },
                None => Ok(None),
            },
            Err(err) => Err(format!("Failed to get from ColumnFamily: {:?}", err)),
        }
    }

    // Search data by prefix
    pub fn read_prefix(
        &self,
        cf: &ColumnFamily,
        search_key: &str,
    ) -> Vec<HashMap<String, Vec<u8>>> {
        let mut iter = self.db.raw_iterator_cf(cf);
        iter.seek(search_key);

        let mut result = Vec::new();
        while iter.valid() {
            let key = iter.key();
            let value = iter.value();

            let mut raw = HashMap::new();
            if key == None || value == None {
                continue;
            }
            let result_key = match String::from_utf8(key.unwrap().to_vec()) {
                Ok(s) => s,
                Err(_) => continue,
            };

            if !result_key.starts_with(search_key) {
                break;
            }
            raw.insert(result_key, value.unwrap().to_vec());
            result.push(raw);
            iter.next();
        }
        return result;
    }

    // Read all data in a ColumnFamily
    pub fn read_all_by_cf(&self, cf: &ColumnFamily) -> Vec<HashMap<String, String>> {
        let mut iter = self.db.raw_iterator_cf(cf);
        iter.seek_to_first();

        let mut result: Vec<HashMap<String, String>> = Vec::new();
        while iter.valid() {
            if let Some(key) = iter.key() {
                if let Some(val) = iter.value() {
                    match String::from_utf8(key.to_vec()) {
                        Ok(key) => match String::from_utf8(val.to_vec()) {
                            Ok(da) => {
                                let mut raw: HashMap<String, String> = HashMap::new();
                                raw.insert(key, da);
                                result.push(raw);
                            }
                            Err(e) => {
                                error!("{}", e);
                            }
                        },
                        Err(e) => {
                            error!("{}", e);
                        }
                    }
                }
            }
            iter.next();
        }
        return result;
    }

    pub fn delete(&self, cf: &ColumnFamily, key: &str) -> Result<(), CommonError> {
        return Ok(self.db.delete_cf(cf, key)?);
    }

    pub fn delete_prefix(&self, cf: &ColumnFamily, search_key: &str) -> Result<(), CommonError> {
        let mut iter = self.db.raw_iterator_cf(cf);
        iter.seek(search_key);

        while iter.valid() {
            if let Some(key) = iter.key() {
                self.db.delete_cf(cf, key)?
            }
            iter.next();
        }
        Ok(())
    }

    pub fn exist(&self, cf: &ColumnFamily, key: &str) -> bool {
        self.db.key_may_exist_cf(cf, key)
    }

    pub fn cf_handle(&self, name: &str) -> Option<&ColumnFamily> {
        if let Some(cf) = self.db.cf_handle(name) {
            return Some(cf);
        }
        return None;
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

        return opts;
    }
}

#[cfg(test)]
mod tests {
    use super::RocksDBEngine;
    use common_base::config::placement_center::placement_center_test_conf;
    use serde::{Deserialize, Serialize};
    use std::{sync::Arc, time::Duration};
    use tokio::{
        fs::{remove_dir, remove_dir_all},
        time::sleep,
    };

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct User {
        pub name: String,
        pub age: u32,
    }

    fn cf_name() -> String {
        return "cluster".to_string();
    }

    #[tokio::test]
    async fn multi_rocksdb_instance() {
        let config = placement_center_test_conf();

        let rs_handler = Arc::new(RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            vec![cf_name()],
        ));

        for i in 1..100 {
            let rs = rs_handler.clone();
            tokio::spawn(async move {
                let key = format!("name2{}", i);

                let name = format!("lobo{}", i);
                let user = User {
                    name: name.clone(),
                    age: 18,
                };
                let cf = rs.cf_handle(&cf_name()).unwrap();
                let res4 = rs.write(cf, &key, &user);
                assert!(res4.is_ok());

                let res1 = rs.read::<User>(cf, &key);
                let r = res1.unwrap();
                assert!(!r.is_none());
                assert_eq!(r.unwrap().name, name);
                println!("spawn {}, key:{}", i, key);
                sleep(Duration::from_secs(5)).await;
            });
        }

        sleep(Duration::from_secs(10)).await;

        remove_dir_all(config.rocksdb.data_path).await.unwrap();
    }

    #[tokio::test]
    async fn init_family() {
        let config = placement_center_test_conf();

        let rs = RocksDBEngine::new(
            &config.rocksdb.data_path,
            config.rocksdb.max_open_files.unwrap(),
            vec!["cluster".to_string()],
        );
        let cf = rs.cf_handle(&cf_name()).unwrap();

        let key = "name2";
        let res1 = rs.read::<User>(cf, key);
        assert!(res1.unwrap().is_none());

        let user = User {
            name: "lobo".to_string(),
            age: 18,
        };
        let res4 = rs.write(cf, key, &user);
        assert!(res4.is_ok());

        let res5 = rs.read::<User>(cf, key);
        let res5_rs = res5.unwrap().unwrap();
        assert!(res5_rs.name == user.name);
        assert!(res5_rs.age == user.age);

        let res6 = rs.delete(cf, key);
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
        let _ = rs.write(cf, &key, &index);
        let res1 = rs.read::<u64>(cf, &key).unwrap().unwrap();
        assert_eq!(index, res1);

        let result = rs.read_all_by_cf(cf);
        assert!(result.len() > 0);
        for raw in result.clone() {
            assert!(raw.contains_key(&key));
            let v = raw.get(&key);
            assert_eq!(index.to_string(), v.unwrap().to_string());

            let _ = rs.write_str(cf, &key, v.unwrap().to_string());
            let res1 = rs.read::<u64>(cf, &key).unwrap().unwrap();
            assert_eq!(index, res1);
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

        rs.write_str(cf, "/v1/v1", "v11".to_string()).unwrap();
        rs.write_str(cf, "/v1/v2", "v12".to_string()).unwrap();
        rs.write_str(cf, "/v1/v3", "v13".to_string()).unwrap();
        rs.write_str(cf, "/v2/tmp_test/s1", "1".to_string())
            .unwrap();
        rs.write_str(cf, "/v2/tmp_test/s3", "2".to_string())
            .unwrap();
        rs.write_str(cf, "/v2/tmp_test/s2", "3".to_string())
            .unwrap();
        rs.write_str(cf, "/v3/tmp_test/s1", "1".to_string())
            .unwrap();
        rs.write_str(cf, "/v3/tmp_test/s3", "2".to_string())
            .unwrap();
        rs.write_str(cf, "/v4/tmp_test/s2", "3".to_string())
            .unwrap();

        let result = rs.read_prefix(cf, "/v1");
        assert_eq!(result.len(), 3);

        let result = rs.read_prefix(cf, "/v2");
        assert_eq!(result.len(), 3);

        let result = rs.read_prefix(cf, "/v3");
        assert_eq!(result.len(), 2);

        let result = rs.read_prefix(cf, "/v4");
        assert_eq!(result.len(), 1);

        remove_dir_all(config.rocksdb.data_path).await.unwrap();
    }
}
