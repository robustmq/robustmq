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

#![allow(clippy::result_large_err)]
use common_base::{error::common::CommonError, utils::serialize};
use rocksdb::{
    BlockBasedOptions, BoundColumnFamily, Cache, ColumnFamilyDescriptor, DBCompactionStyle,
    DBCompressionType, Options, SliceTransform, DB,
};
use serde::{de::DeserializeOwned, Serialize};

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct RocksDBConfig {
    pub block_cache_size: usize,
    pub write_buffer_size: usize,
    pub max_write_buffer_number: i32,
}

impl Default for RocksDBConfig {
    fn default() -> Self {
        Self {
            block_cache_size: 512 * 1024 * 1024,
            write_buffer_size: 128 * 1024 * 1024,
            max_write_buffer_number: 4,
        }
    }
}

#[derive(Debug)]
pub struct RocksDBEngine {
    pub db: Arc<DB>,
}

impl RocksDBEngine {
    pub fn new(data_path: &str, max_open_files: i32, cf_list: Vec<String>) -> Self {
        Self::new_with_config(data_path, max_open_files, None, cf_list)
    }

    pub fn new_with_config(
        data_path: &str,
        max_open_files: i32,
        config: Option<&RocksDBConfig>,
        cf_list: Vec<String>,
    ) -> Self {
        let default_config = RocksDBConfig::default();
        let cfg = config.unwrap_or(&default_config);

        let opts = Self::open_db_opts_with_config(max_open_files, cfg);
        let shared_cache = Cache::new_lru_cache(cfg.block_cache_size);
        let cf_column_family: Vec<_> = cf_list
            .into_iter()
            .map(|cf| {
                let cf_opts = Self::open_cf_opts_with_config(max_open_files, cfg, &shared_cache);
                ColumnFamilyDescriptor::new(cf, cf_opts)
            })
            .collect();

        let instance = DB::open_cf_descriptors(&opts, data_path, cf_column_family)
            .unwrap_or_else(|e| panic!("Open RocksDB Fail: {e}"));

        RocksDBEngine {
            db: Arc::new(instance),
        }
    }

    /// Write the data serialization to RocksDB using bincode (high performance)
    pub fn write<T: Serialize>(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        key: &str,
        value: &T,
    ) -> Result<(), CommonError> {
        let serialized = serialize::serialize(value)?;

        self.db
            .put_cf(&cf, key, serialized)
            .map_err(|e| CommonError::CommonError(format!("Failed to put to CF: {e:?}")))
    }

    /// Write raw bytes directly to RocksDB without serialization
    /// This is useful for snapshot recovery where data is already serialized
    pub fn write_raw(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        key: &str,
        value: &[u8],
    ) -> Result<(), CommonError> {
        self.db
            .put_cf(&cf, key, value)
            .map_err(|e| CommonError::CommonError(format!("Failed to put to CF: {e:?}")))
    }

    /// Execute a write batch atomically
    /// This provides better performance for multiple writes
    pub fn write_batch(&self, batch: rocksdb::WriteBatch) -> Result<(), CommonError> {
        self.db
            .write(batch)
            .map_err(|e| CommonError::CommonError(format!("Failed to write batch: {e:?}")))?;
        Ok(())
    }

    /// Read data from RocksDB using bincode deserialization (high performance)
    pub fn read<T: DeserializeOwned>(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        key: &str,
    ) -> Result<Option<T>, CommonError> {
        match self.db.get_cf(&cf, key) {
            Ok(Some(data)) => {
                if data.is_empty() {
                    Ok(None)
                } else {
                    serialize::deserialize::<T>(&data).map(Some)
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(CommonError::CommonError(format!(
                "Failed to get from CF: {e:?}"
            ))),
        }
    }

    /// Batch read multiple keys at once (more efficient than individual reads)
    /// Returns a vector of results in the same order as the input keys
    pub fn multi_get<T: DeserializeOwned>(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        keys: &[impl AsRef<str>],
    ) -> Result<Vec<Option<T>>, CommonError> {
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        // Use RocksDB's multi_get_cf for efficient batch reading
        let key_bytes: Vec<Vec<u8>> = keys
            .iter()
            .map(|k| k.as_ref().as_bytes().to_vec())
            .collect();

        let keys_with_cf: Vec<(&Arc<BoundColumnFamily>, &[u8])> =
            key_bytes.iter().map(|k| (&cf, k.as_slice())).collect();

        let results = self.db.multi_get_cf(keys_with_cf);

        // Process results
        let mut output = Vec::with_capacity(results.len());
        for result in results {
            match result {
                Ok(Some(data)) if !data.is_empty() => {
                    let value = serialize::deserialize::<T>(&data)?;
                    output.push(Some(value));
                }
                Ok(_) => output.push(None),
                Err(e) => {
                    return Err(CommonError::CommonError(format!(
                        "Failed to multi_get: {e:?}"
                    )))
                }
            }
        }

        Ok(output)
    }

    // Search data by prefix
    pub fn read_prefix(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        search_key: &str,
    ) -> Result<Vec<(String, Vec<u8>)>, CommonError> {
        let mut iter = self.db.raw_iterator_cf(&cf);
        iter.seek(search_key);

        let mut result = Vec::with_capacity(64); // Pre-allocate capacity

        while iter.valid() {
            let Some(key_bytes) = iter.key() else {
                break;
            };

            let key = String::from_utf8(key_bytes.to_vec())?;
            if !key.starts_with(search_key) {
                break;
            }

            if let Some(val) = iter.value() {
                result.push((key, val.to_vec()));
            }

            iter.next();
        }
        Ok(result)
    }

    // Search data by prefix
    pub fn read_list_by_model(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        mode: &rocksdb::IteratorMode,
    ) -> Result<Vec<(String, Vec<u8>)>, CommonError> {
        let mut result = Vec::with_capacity(64); // Pre-allocate capacity

        for item in self.db.iterator_cf(&cf, *mode) {
            let (k, value) = item?;
            let key = String::from_utf8(k.to_vec())?;
            result.push((key, value.to_vec()));
        }
        Ok(result)
    }

    // Read all data in a ColumnFamily
    pub fn read_all_by_cf(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
    ) -> Result<Vec<(String, Vec<u8>)>, CommonError> {
        let mut iter = self.db.raw_iterator_cf(&cf);
        iter.seek_to_first();

        let mut result = Vec::with_capacity(128); // Pre-allocate for "all" data

        while iter.valid() {
            let Some(key_bytes) = iter.key() else {
                break;
            };

            let key = String::from_utf8(key_bytes.to_vec())?;

            if let Some(val) = iter.value() {
                result.push((key, val.to_vec()));
            }

            iter.next();
        }
        Ok(result)
    }

    pub fn delete(&self, cf: Arc<BoundColumnFamily<'_>>, key: &str) -> Result<(), CommonError> {
        Ok(self.db.delete_cf(&cf, key)?)
    }

    pub fn delete_range_cf(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        from: Vec<u8>,
        to: Vec<u8>,
    ) -> Result<(), CommonError> {
        Ok(self.db.delete_range_cf(&cf, from, to)?)
    }

    pub fn delete_prefix(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        prefix: &str,
    ) -> Result<(), CommonError> {
        let start = prefix.as_bytes().to_vec();
        let end = self.prefix_range_end(prefix);
        self.delete_range_cf(cf, start, end)
    }

    #[inline]
    fn prefix_range_end(&self, prefix: &str) -> Vec<u8> {
        let mut end = prefix.as_bytes().to_vec();

        for i in (0..end.len()).rev() {
            if end[i] < 255 {
                end[i] += 1;
                return end;
            }
        }

        end.push(0);
        end
    }

    pub fn exist(&self, cf: Arc<BoundColumnFamily<'_>>, key: &str) -> bool {
        self.db.key_may_exist_cf(&cf, key)
    }

    pub fn cf_handle(&self, name: &str) -> Option<Arc<BoundColumnFamily<'_>>> {
        self.db.cf_handle(name)
    }

    fn open_cf_opts_with_config(
        _max_open_files: i32,
        config: &RocksDBConfig,
        shared_cache: &Cache,
    ) -> Options {
        let mut opts = Options::default();

        opts.set_write_buffer_size(config.write_buffer_size);
        opts.set_max_write_buffer_number(config.max_write_buffer_number);
        opts.set_min_write_buffer_number_to_merge(2);

        opts.set_compaction_style(DBCompactionStyle::Level);
        opts.set_level_compaction_dynamic_level_bytes(true);
        opts.set_level_zero_file_num_compaction_trigger(8);
        opts.set_level_zero_stop_writes_trigger(32);
        opts.set_level_zero_slowdown_writes_trigger(16);
        opts.set_target_file_size_base(128 * 1024 * 1024);
        opts.set_target_file_size_multiplier(2);

        opts.set_compression_type(DBCompressionType::Lz4);
        opts.set_compression_per_level(&[
            DBCompressionType::None,
            DBCompressionType::None,
            DBCompressionType::Lz4,
            DBCompressionType::Lz4,
            DBCompressionType::Zstd,
        ]);

        let transform = SliceTransform::create_fixed_prefix(10);
        opts.set_prefix_extractor(transform);
        opts.set_memtable_prefix_bloom_ratio(0.2);

        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_bloom_filter(10.0, false);
        block_opts.set_block_size(4 * 1024);

        block_opts.set_block_cache(shared_cache);
        block_opts.set_cache_index_and_filter_blocks(true);
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        block_opts.set_index_type(rocksdb::BlockBasedIndexType::TwoLevelIndexSearch);
        block_opts.set_partition_filters(true);
        block_opts.set_whole_key_filtering(true);

        opts.set_block_based_table_factory(&block_opts);

        opts.set_max_bytes_for_level_base(256 * 1024 * 1024);
        opts.set_max_bytes_for_level_multiplier(10.0);

        opts
    }

    fn open_db_opts_with_config(max_open_files: i32, config: &RocksDBConfig) -> Options {
        let mut opts = Options::default();

        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(max_open_files);

        opts.set_write_buffer_size(config.write_buffer_size);
        opts.set_max_write_buffer_number(config.max_write_buffer_number);
        opts.set_min_write_buffer_number_to_merge(2);

        opts.set_enable_pipelined_write(true);
        opts.set_use_fsync(false);

        opts.set_compaction_style(DBCompactionStyle::Level);
        opts.set_disable_auto_compactions(false);
        opts.set_level_compaction_dynamic_level_bytes(true);

        opts.set_level_zero_file_num_compaction_trigger(8);
        opts.set_level_zero_stop_writes_trigger(32);
        opts.set_level_zero_slowdown_writes_trigger(16);

        opts.set_target_file_size_base(128 * 1024 * 1024);
        opts.set_target_file_size_multiplier(2);

        opts.set_max_background_jobs(4);
        opts.set_max_subcompactions(2);

        opts.set_compression_type(DBCompressionType::Lz4);
        opts.set_compression_per_level(&[
            DBCompressionType::None,
            DBCompressionType::None,
            DBCompressionType::Lz4,
            DBCompressionType::Lz4,
            DBCompressionType::Zstd,
        ]);
        opts.set_zstd_max_train_bytes(100 * 1024 * 1024);

        let transform = SliceTransform::create_fixed_prefix(10);
        opts.set_prefix_extractor(transform);
        opts.set_memtable_prefix_bloom_ratio(0.2);

        opts
    }
}

#[cfg(test)]
mod tests {
    use crate::test::test_rocksdb_instance;
    use common_config::broker::default_rocksdb_family;
    use futures::future;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
    struct User {
        pub name: String,
        pub age: u32,
    }

    #[tokio::test]
    async fn multi_rocksdb_instance() {
        let rs_handler = test_rocksdb_instance();
        let mut tasks = Vec::new();
        for i in 1..100 {
            let rs = rs_handler.clone();
            let task = tokio::spawn(async move {
                let key = format!("name2{i}");
                let name = format!("lobo{i}");
                let user = User {
                    name: name.clone(),
                    age: 18,
                };
                let cf = rs.cf_handle(&default_rocksdb_family()).unwrap();
                let res4 = rs.write(cf.clone(), &key, &user);
                assert!(res4.is_ok());
                let res1 = rs.read::<User>(cf.clone(), &key);
                let r = res1.unwrap();
                assert!(r.is_some());
                assert_eq!(r.unwrap().name, name);
                println!("spawn {i}, key:{key}");
            });
            tasks.push(task);
        }

        let _ = future::join_all(tasks).await;
    }

    #[tokio::test]
    async fn base_rw() {
        let rs = test_rocksdb_instance();
        let cf = rs.cf_handle(&default_rocksdb_family()).unwrap();

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
    }
}
