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

use crate::{
    engine::StorageEngineAdapter, memory::MemoryStorageAdapter, rocksdb::RocksDBStorageAdapter,
    storage::StorageAdapter,
};
use common_base::error::common::CommonError;
use common_config::storage::{memory::StorageDriverMemoryConfig, StorageAdapterType};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use storage_engine::{
    core::cache::StorageCacheManager, handler::adapter::StorageEngineHandler,
    memory::engine::MemoryStorageEngine, rocksdb::engine::RocksDBStorageEngine,
};

pub type ArcStorageAdapter = Arc<dyn StorageAdapter + Send + Sync>;

pub struct StorageDriverManager {
    pub memory_storage: ArcStorageAdapter,
    pub rocksdb_storage: ArcStorageAdapter,
    pub engine_storage: ArcStorageAdapter,
}

impl StorageDriverManager {
    pub async fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        storage_cache_manager: Arc<StorageCacheManager>,
        engine_adapter_handler: Arc<StorageEngineHandler>,
    ) -> Result<Self, CommonError> {
        let memory_storage = StorageDriverManager::build_message_storage_driver(
            &rocksdb_engine_handler,
            &storage_cache_manager,
            &engine_adapter_handler,
            StorageAdapterType::Memory,
        )
        .await?;
        let rocksdb_storage = StorageDriverManager::build_message_storage_driver(
            &rocksdb_engine_handler,
            &storage_cache_manager,
            &engine_adapter_handler,
            StorageAdapterType::RocksDB,
        )
        .await?;
        let engine_storage = StorageDriverManager::build_message_storage_driver(
            &rocksdb_engine_handler,
            &storage_cache_manager,
            &engine_adapter_handler,
            StorageAdapterType::Engine,
        )
        .await?;
        Ok(StorageDriverManager {
            memory_storage,
            rocksdb_storage,
            engine_storage,
        })
    }

    async fn build_message_storage_driver(
        rocksdb_engine_handler: &Arc<RocksDBEngine>,
        storage_cache_manager: &Arc<StorageCacheManager>,
        engine_adapter_handler: &Arc<StorageEngineHandler>,
        storage_type: StorageAdapterType,
    ) -> Result<ArcStorageAdapter, CommonError> {
        let storage: ArcStorageAdapter = match storage_type {
            StorageAdapterType::Memory => {
                let engine = MemoryStorageEngine::create_standalone(
                    rocksdb_engine_handler.clone(),
                    storage_cache_manager.clone(),
                    StorageDriverMemoryConfig::default(),
                );
                Arc::new(MemoryStorageAdapter::new(Arc::new(engine)))
            }

            StorageAdapterType::Engine => {
                Arc::new(StorageEngineAdapter::new(engine_adapter_handler.clone()).await)
            }

            StorageAdapterType::RocksDB => {
                let engine = Arc::new(RocksDBStorageEngine::create_standalone(
                    storage_cache_manager.clone(),
                    rocksdb_engine_handler.clone(),
                ));
                Arc::new(RocksDBStorageAdapter::new(engine.clone()))
            }

            StorageAdapterType::S3 => {
                return Err(CommonError::UnavailableStorageType);
            }

            StorageAdapterType::Mysql => {
                return Err(CommonError::UnavailableStorageType);
            }

            StorageAdapterType::MinIO => {
                return Err(CommonError::UnavailableStorageType);
            }
        };

        Ok(storage)
    }
}
