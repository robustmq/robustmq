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

use crate::core::{cache::StorageCacheManager, offset::ShardOffset};
use bytes::Bytes;
use common_config::storage::memory::StorageDriverMemoryConfig;
use dashmap::DashMap;
use metadata_struct::storage::record::StorageRecord;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub struct MemoryShardData {
    pub data: DashMap<u64, StorageRecord>,
    pub tag_index: DashMap<String, Vec<u64>>,
    pub key_index: DashMap<Bytes, u64>,
    pub timestamp_index: DashMap<u64, u64>,
    pub write_lock: Arc<tokio::sync::Mutex<()>>,
}

impl MemoryShardData {
    pub fn new(capacity: usize) -> Self {
        MemoryShardData {
            data: DashMap::with_capacity(capacity),
            tag_index: DashMap::with_capacity(8),
            key_index: DashMap::with_capacity(8),
            timestamp_index: DashMap::with_capacity(8),
            write_lock: Arc::new(tokio::sync::Mutex::new(())),
        }
    }
}

#[derive(Clone)]
pub struct MemoryStorageEngine {
    pub shards: DashMap<String, Arc<MemoryShardData>>,
    pub config: StorageDriverMemoryConfig,
    pub commit_log_offset: Arc<ShardOffset>,
    pub cache_manager: Arc<StorageCacheManager>,
}

impl MemoryStorageEngine {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<StorageCacheManager>,
        config: StorageDriverMemoryConfig,
    ) -> Self {
        MemoryStorageEngine {
            shards: DashMap::with_capacity(8),
            config,
            commit_log_offset: Arc::new(ShardOffset::new(
                cache_manager.clone(),
                rocksdb_engine_handler.clone(),
            )),
            cache_manager,
        }
    }

    pub fn get_or_create_shard(&self, shard_name: &str) -> Arc<MemoryShardData> {
        let capacity = self.config.max_shard_size_limit.min(1024);
        self.shards
            .entry(shard_name.to_string())
            .or_insert_with(|| Arc::new(MemoryShardData::new(capacity)))
            .clone()
    }
}
