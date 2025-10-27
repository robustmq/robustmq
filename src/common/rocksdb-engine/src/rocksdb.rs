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
use common_base::error::common::CommonError;
use rocksdb::{
    BlockBasedOptions, BoundColumnFamily, Cache, ColumnFamilyDescriptor, DBCompactionStyle,
    DBCompressionType, Options, SliceTransform, DB,
};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug)]
pub struct RocksDBEngine {
    pub db: Arc<DB>,
}

impl RocksDBEngine {
    /// Create a rocksdb instance
    pub fn new(data_path: &str, max_open_files: i32, cf_list: Vec<String>) -> Self {
        let opts: Options = Self::open_db_opts(max_open_files);
        let path = data_path.to_string();

        // Create shared cache for all column families (512MB total)
        // This is more memory-efficient than per-CF caches
        let shared_cache = Cache::new_lru_cache(512 * 1024 * 1024);

        let mut cf_column_family = Vec::new();
        for cf in cf_list {
            // Use optimized column family options with shared cache
            let cf_opts = Self::open_cf_opts(max_open_files, &shared_cache);
            cf_column_family.push(ColumnFamilyDescriptor::new(cf, cf_opts));
        }
        let instance = match DB::open_cf_descriptors(&opts, path, cf_column_family) {
            Ok(instance) => instance,
            Err(e) => {
                panic!("Open RocksDB Fail,{e}");
            }
        };

        RocksDBEngine {
            db: Arc::new(instance),
        }
    }

    /// Write the data serialization to RocksDB
    pub fn write<T: Serialize + std::fmt::Debug>(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        key: &str,
        value: &T,
    ) -> Result<(), CommonError> {
        match serde_json::to_string(&value) {
            Ok(serialized) => {
                if let Err(e) = self.db.put_cf(&cf.clone(), key, serialized) {
                    return Err(CommonError::CommonError(format!(
                        "Failed to put to ColumnFamily:{e:?}"
                    )));
                }
            }
            Err(err) => {
                return Err(CommonError::CommonError(format!(
                    "Failed to serialize to String. T: {value:?}, err: {err:?}"
                )))
            }
        }
        Ok(())
    }

    pub fn write_str(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        key: &str,
        value: String,
    ) -> Result<(), CommonError> {
        self.write(cf, key, &value)
    }

    pub fn write_raw(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        key: &str,
        value: &[u8],
    ) -> Result<(), CommonError> {
        if let Err(e) = self.db.put_cf(&cf.clone(), key, value) {
            return Err(CommonError::CommonError(format!(
                "Failed to put to ColumnFamily:{e:?}"
            )));
        }
        Ok(())
    }

    // Read data from the RocksDB
    pub fn read<T: DeserializeOwned>(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
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
                        "Failed to deserialize: {err:?}"
                    ))),
                }
            }
            Ok(None) => Ok(None),
            Err(err) => Err(CommonError::CommonError(format!(
                "Failed to get from ColumnFamily: {err:?}"
            ))),
        }
    }

    // Search data by prefix
    pub fn read_prefix(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
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

    // Search data by prefix
    pub fn read_list_by_model(
        &self,
        cf: Arc<BoundColumnFamily<'_>>,
        mode: &rocksdb::IteratorMode,
    ) -> Result<Vec<(String, Vec<u8>)>, CommonError> {
        let mut result = Vec::new();
        let iter = self.db.iterator_cf(&cf, *mode);
        for raw in iter {
            let (k, value) = raw?;
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

    pub fn exist(&self, cf: Arc<BoundColumnFamily<'_>>, key: &str) -> bool {
        self.db.key_may_exist_cf(&cf, key)
    }

    pub fn cf_handle(&self, name: &str) -> Option<Arc<BoundColumnFamily<'_>>> {
        if let Some(cf) = self.db.cf_handle(name) {
            return Some(cf);
        }
        None
    }

    fn open_cf_opts(_max_open_files: i32, shared_cache: &Cache) -> Options {
        let mut opts = Options::default();

        // ========== Write Optimization ==========
        opts.set_write_buffer_size(128 * 1024 * 1024); // 128MB
        opts.set_max_write_buffer_number(4);
        opts.set_min_write_buffer_number_to_merge(2);

        // ========== Compaction Optimization ==========
        opts.set_compaction_style(DBCompactionStyle::Level);
        opts.set_level_compaction_dynamic_level_bytes(true);
        opts.set_level_zero_file_num_compaction_trigger(8);
        opts.set_level_zero_stop_writes_trigger(32);
        opts.set_level_zero_slowdown_writes_trigger(16);
        opts.set_target_file_size_base(128 * 1024 * 1024);
        opts.set_target_file_size_multiplier(2);

        // ========== Compression ==========
        opts.set_compression_type(DBCompressionType::Lz4);
        opts.set_compression_per_level(&[
            DBCompressionType::None,
            DBCompressionType::None,
            DBCompressionType::Lz4,
            DBCompressionType::Lz4,
            DBCompressionType::Zstd,
        ]);

        // ========== Read Optimization ==========
        let transform = SliceTransform::create_fixed_prefix(10);
        opts.set_prefix_extractor(transform);
        opts.set_memtable_prefix_bloom_ratio(0.2);

        // Block-based table options with shared cache
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_bloom_filter(10.0, false);
        block_opts.set_block_size(4 * 1024);

        // Use shared cache across all column families
        block_opts.set_block_cache(shared_cache);
        block_opts.set_cache_index_and_filter_blocks(true);
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        block_opts.set_index_type(rocksdb::BlockBasedIndexType::TwoLevelIndexSearch);
        block_opts.set_partition_filters(true);
        block_opts.set_whole_key_filtering(true);

        opts.set_block_based_table_factory(&block_opts);

        // ========== Level Compaction ==========
        opts.set_max_bytes_for_level_base(256 * 1024 * 1024);
        opts.set_max_bytes_for_level_multiplier(10.0);

        opts
    }

    fn open_db_opts(max_open_files: i32) -> Options {
        let mut opts = Options::default();

        // ========== Basic Configuration ==========
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_max_open_files(max_open_files);

        // ========== Write Optimization (for high-frequency writes) ==========
        // Reduce write buffer size to increase flush frequency (512MB -> 128MB)
        // Balance between memory usage and write performance
        opts.set_write_buffer_size(128 * 1024 * 1024); // 128MB

        // Reduce write buffer count (32 -> 4)
        // Lower memory usage while maintaining sufficient concurrent write capacity
        opts.set_max_write_buffer_number(4);

        // Write buffer merge threshold (4 -> 2)
        // Flush data to SST files faster
        opts.set_min_write_buffer_number_to_merge(2);

        // Enable pipelined write for better throughput
        opts.set_enable_pipelined_write(true);

        // WAL configuration: no fsync for better write performance
        opts.set_use_fsync(false);

        // ========== Compaction Optimization (MQTT workload characteristics) ==========
        // Use Level compaction style (better for mixed read/write workloads)
        opts.set_compaction_style(DBCompactionStyle::Level);

        // Enable auto compaction (essential for maintaining read performance)
        opts.set_disable_auto_compactions(false);

        // Dynamic level size adjustment (adapts to changing data volumes)
        opts.set_level_compaction_dynamic_level_bytes(true);

        // Number of Level 0 files to trigger compaction (2000 -> 8)
        // More aggressive compaction to avoid read amplification
        opts.set_level_zero_file_num_compaction_trigger(8);

        // Number of Level 0 files to stop writes (2000 -> 32)
        // Prevent writes from significantly outpacing compaction
        opts.set_level_zero_stop_writes_trigger(32);

        // Number of Level 0 files to slow down writes (0 -> 16)
        // Give compaction time before stopping writes
        opts.set_level_zero_slowdown_writes_trigger(16);

        // SST file size (1GB -> 128MB)
        // Smaller files facilitate compaction and range deletions
        opts.set_target_file_size_base(128 * 1024 * 1024);

        // File size multiplier for different levels
        opts.set_target_file_size_multiplier(2);

        // Maximum background jobs (compaction + flush)
        // Balance between background work and foreground performance
        opts.set_max_background_jobs(6);

        // Maximum subcompactions for parallel compaction
        opts.set_max_subcompactions(2);

        // ========== Data Compression (save disk space and I/O) ==========
        // Use LZ4 compression (fast with moderate compression ratio)
        opts.set_compression_type(DBCompressionType::Lz4);

        // Different compression algorithms for different levels
        // Hot data (L0-L1): no compression for faster access
        // Warm data (L2): LZ4 for balanced performance
        // Cold data (L3+): Zstd for better compression ratio
        opts.set_compression_per_level(&[
            DBCompressionType::None, // Level 0: no compression (hot data)
            DBCompressionType::None, // Level 1: no compression
            DBCompressionType::Lz4,  // Level 2: LZ4 (warm data)
            DBCompressionType::Lz4,  // Level 3: LZ4
            DBCompressionType::Zstd, // Level 4+: Zstd (cold data, better ratio)
        ]);

        // Compression options for Zstd
        opts.set_zstd_max_train_bytes(100 * 1024 * 1024); // 100MB dictionary training

        // ========== Read Optimization (prefix query + BloomFilter) ==========
        // Prefix extractor (fixed length 10, adapted to key design)
        let transform = SliceTransform::create_fixed_prefix(10);
        opts.set_prefix_extractor(transform);

        // Memtable prefix BloomFilter (reduce unnecessary SST file reads)
        opts.set_memtable_prefix_bloom_ratio(0.2);

        // Memtable configuration
        // Use SkipList for balanced read/write performance
        // opts.set_allow_concurrent_memtable_write(true); // Default is true

        // Block-based Table configuration (BloomFilter + Cache)
        let mut block_opts = BlockBasedOptions::default();

        // Enable BloomFilter (10 bits per key, ~1% false positive rate)
        // Significantly accelerates point queries (exist operations)
        block_opts.set_bloom_filter(10.0, false);

        // Use ribbon filter for better performance (requires RocksDB 6.15+)
        // block_opts.set_ribbon_filter(10.0);

        // Block size (default 4KB, suitable for small objects)
        // MQTT messages are typically small, so keep default
        block_opts.set_block_size(4 * 1024);

        // Block cache (256MB for DB-level operations)
        // Note: Column families use a separate shared 512MB cache
        let cache = Cache::new_lru_cache(256 * 1024 * 1024);
        block_opts.set_block_cache(&cache);

        // Enable index and filter caching in block cache
        block_opts.set_cache_index_and_filter_blocks(true);

        // Pin L0 filter and index blocks in cache (reduce read amplification)
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);

        // Enable partitioned index/filters for large databases
        block_opts.set_index_type(rocksdb::BlockBasedIndexType::TwoLevelIndexSearch);
        block_opts.set_partition_filters(true);

        // Enable whole key filtering (optimize point lookups)
        block_opts.set_whole_key_filtering(true);

        // Apply block-based table options
        opts.set_block_based_table_factory(&block_opts);

        // ========== I/O and Performance Optimization ==========
        // Background sync (8MB buffer, reduce write latency)
        opts.set_bytes_per_sync(8 * 1024 * 1024);

        // WAL sync (8MB buffer)
        opts.set_wal_bytes_per_sync(8 * 1024 * 1024);

        // Table cache shard count (reduce lock contention)
        opts.set_table_cache_num_shard_bits(6);

        // Increase parallelism for compaction
        opts.increase_parallelism(num_cpus::get() as i32);

        // Allow mmap reads for better read performance on large files
        opts.set_allow_mmap_reads(false); // Disable mmap for stability

        // Allow mmap writes
        opts.set_allow_mmap_writes(false); // Disable mmap for stability

        // ========== Memory Management ==========
        // Set max total WAL size (512MB)
        // Limit WAL size to avoid excessive memory usage
        opts.set_max_total_wal_size(512 * 1024 * 1024);

        // Recycle log files (reduce file system operations)
        opts.set_recycle_log_file_num(4);

        // ========== Statistics and Monitoring ==========
        // Enable statistics (for monitoring and tuning)
        opts.enable_statistics();

        // Stats dump period (every 10 minutes)
        opts.set_stats_dump_period_sec(600);

        // Statistics level (kExceptDetailedTimers)
        // opts.set_statistics_level(rocksdb::StatsLevel::ExceptDetailedTimers);

        // ========== Advanced Tuning ==========
        // Base level size (256MB)
        opts.set_max_bytes_for_level_base(256 * 1024 * 1024);

        // Level size multiplier
        opts.set_max_bytes_for_level_multiplier(10.0);

        // Compaction readahead size (2MB, for sequential reads during compaction)
        opts.set_compaction_readahead_size(2 * 1024 * 1024);

        // Enable adaptive readahead for iterators
        // opts.set_adaptive_readahead(true); // Requires newer RocksDB version

        // ========== Delete and TTL Optimization ==========
        // Enable delete range support (efficient range deletions)
        // This is useful for MQTT message expiration and cleanup
        // (Enabled by default in newer RocksDB versions)

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

    #[tokio::test]
    async fn read_all() {
        let rs = test_rocksdb_instance();

        let index = 66u64;
        let cf = rs.cf_handle(&default_rocksdb_family()).unwrap();

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
    }

    #[tokio::test]
    async fn read_prefix() {
        let rs = test_rocksdb_instance();

        let cf = rs.cf_handle(&default_rocksdb_family()).unwrap();

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
    }
}
