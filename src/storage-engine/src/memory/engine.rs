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

use crate::core::error::StorageEngineError;
use common_config::storage::memory::StorageDriverMemoryConfig;
use dashmap::DashMap;
use metadata_struct::storage::adapter_offset::AdapterShardInfo;
use metadata_struct::storage::storage_record::StorageRecord;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

#[derive(Clone, Debug, Default)]
pub struct ShardState {
    pub earliest_offset: u64,
    pub latest_offset: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum MemoryStorageType {
    #[default]
    Full,
    Storage,
}

#[derive(Clone)]
pub struct MemoryStorageEngine {
    // ====inner struct====
    //(shard, lock)
    pub shard_write_locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
    pub config: StorageDriverMemoryConfig,
    pub engine_type: MemoryStorageType,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,

    // ====Metadata data====
    //(shard, (ShardInfo))
    pub shard_info: DashMap<String, AdapterShardInfo>,
    //(group, (shard, offset))
    pub group_data: DashMap<String, DashMap<String, u64>>,
    //(shard, ShardState)
    pub shard_state: DashMap<String, ShardState>,

    // ====Message Data====
    //(shard, (offset,Record))
    pub shard_data: DashMap<String, DashMap<u64, StorageRecord>>,
    //(shard, (tag, (offset)))
    pub tag_index: DashMap<String, DashMap<String, Vec<u64>>>,
    //(shard, (key, offset))
    pub key_index: DashMap<String, DashMap<String, u64>>,
    //(shard, (timestamp, offset))
    pub timestamp_index: DashMap<String, DashMap<u64, u64>>,
}

impl MemoryStorageEngine {
    pub fn create_full(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        config: StorageDriverMemoryConfig,
    ) -> Self {
        MemoryStorageEngine::new(rocksdb_engine_handler, MemoryStorageType::Full, config)
    }

    pub fn create_storage(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        config: StorageDriverMemoryConfig,
    ) -> Self {
        MemoryStorageEngine::new(rocksdb_engine_handler, MemoryStorageType::Storage, config)
    }

    fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        engine_type: MemoryStorageType,
        config: StorageDriverMemoryConfig,
    ) -> Self {
        MemoryStorageEngine {
            rocksdb_engine_handler,
            engine_type,
            shard_info: DashMap::with_capacity(8),
            shard_data: DashMap::with_capacity(8),
            shard_state: DashMap::with_capacity(8),
            tag_index: DashMap::with_capacity(8),
            key_index: DashMap::with_capacity(8),
            timestamp_index: DashMap::with_capacity(8),
            group_data: DashMap::with_capacity(8),
            shard_write_locks: DashMap::with_capacity(8),
            config,
        }
    }

    pub fn storage_type_check(&self) -> Result<(), StorageEngineError> {
        if self.engine_type == MemoryStorageType::Storage {
            return Err(StorageEngineError::NotSupportMemoryStorageType(
                "create_shard".to_string(),
            ));
        }
        Ok(())
    }
}
