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

use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::core::shard::ShardState;
use dashmap::DashMap;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_BROKER;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IndexInfo {
    pub shard_name: String,
    pub offset: u64,
    pub create_time: u64,
}

#[derive(Clone)]
pub struct RocksDBStorageEngine {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub cache_manager: Arc<StorageCacheManager>,
    pub shard_write_locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
    pub shard_state: DashMap<String, ShardState>,
}

impl RocksDBStorageEngine {
    pub fn new(cache_manager: Arc<StorageCacheManager>, db: Arc<RocksDBEngine>) -> Self {
        RocksDBStorageEngine {
            rocksdb_engine_handler: db,
            cache_manager,
            shard_write_locks: DashMap::with_capacity(8),
            shard_state: DashMap::with_capacity(8),
        }
    }

    pub fn get_cf(&self) -> Result<Arc<rocksdb::BoundColumnFamily<'_>>, StorageEngineError> {
        self.rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_BROKER)
            .ok_or_else(|| {
                StorageEngineError::CommonErrorStr(format!(
                    "Column family '{}' not found",
                    DB_COLUMN_FAMILY_BROKER
                ))
            })
    }
}
