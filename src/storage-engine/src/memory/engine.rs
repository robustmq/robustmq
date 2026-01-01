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
use crate::core::shard::{ShardState, StorageEngineRunType};
use common_config::storage::memory::StorageDriverMemoryConfig;
use dashmap::DashMap;
use metadata_struct::storage::adapter_offset::AdapterShardInfo;
use metadata_struct::storage::storage_record::StorageRecord;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

#[derive(Clone)]
pub struct MemoryStorageEngine {
    // ====inner struct====
    //(shard, lock)
    pub shard_write_locks: DashMap<String, Arc<tokio::sync::Mutex<()>>>,
    pub config: StorageDriverMemoryConfig,
    pub engine_type: StorageEngineRunType,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub cache_manager: Arc<StorageCacheManager>,

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
    pub fn create_standalone(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<StorageCacheManager>,
        config: StorageDriverMemoryConfig,
    ) -> Self {
        MemoryStorageEngine::new(
            rocksdb_engine_handler,
            cache_manager,
            StorageEngineRunType::Standalone,
            config,
        )
    }

    pub fn create_storage(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<StorageCacheManager>,
        config: StorageDriverMemoryConfig,
    ) -> Self {
        MemoryStorageEngine::new(
            rocksdb_engine_handler,
            cache_manager,
            StorageEngineRunType::EngineStorage,
            config,
        )
    }

    fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<StorageCacheManager>,
        engine_type: StorageEngineRunType,
        config: StorageDriverMemoryConfig,
    ) -> Self {
        MemoryStorageEngine {
            cache_manager,
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
        if self.engine_type == StorageEngineRunType::EngineStorage {
            return Err(StorageEngineError::NotSupportMemoryStorageType(
                "create_shard".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_build_memory_engine;

    #[test]
    fn test_storage_type_check() {
        let standalone = test_build_memory_engine(StorageEngineRunType::Standalone);
        assert!(standalone.storage_type_check().is_ok());
        let engine_storage = test_build_memory_engine(StorageEngineRunType::EngineStorage);
        assert!(engine_storage.storage_type_check().is_err());
    }
}
