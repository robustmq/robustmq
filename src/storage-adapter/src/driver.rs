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
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    storage_cache_manager: Arc<StorageCacheManager>,
    engine_adapter_handler: Arc<StorageEngineHandler>,

    memory_storage: Option<ArcStorageAdapter>,
    rocksdb_storage: Option<ArcStorageAdapter>,
    engine_storage: Option<ArcStorageAdapter>,
}

impl StorageDriverManager {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        storage_cache_manager: Arc<StorageCacheManager>,
        engine_adapter_handler: Arc<StorageEngineHandler>,
    ) -> Self {
        StorageDriverManager {
            rocksdb_engine_handler,
            storage_cache_manager,
            engine_adapter_handler,
            memory_storage: None,
            rocksdb_storage: None,
            engine_storage: None,
        }
    }

    pub async fn init_driver(&mut self) -> Result<(), CommonError> {
        self.memory_storage = Some(
            self.build_message_storage_driver(StorageAdapterType::Memory)
                .await?,
        );
        self.rocksdb_storage = Some(
            self.build_message_storage_driver(StorageAdapterType::RocksDB)
                .await?,
        );
        self.engine_storage = Some(
            self.build_message_storage_driver(StorageAdapterType::Engine)
                .await?,
        );
        Ok(())
    }

    pub async fn get_memory_storage_driver(&self) -> Result<ArcStorageAdapter, CommonError> {
        if let Some(driver) = self.memory_storage.clone() {
            return Ok(driver);
        }

        Err(CommonError::CommonError("".to_string()))
    }

    pub async fn get_rocksdb_storage_driver(&self) -> Result<ArcStorageAdapter, CommonError> {
        if let Some(driver) = self.rocksdb_storage.clone() {
            return Ok(driver);
        }

        Err(CommonError::CommonError("".to_string()))
    }

    pub async fn get_engine_storage_driver(&self) -> Result<ArcStorageAdapter, CommonError> {
        if let Some(driver) = self.engine_storage.clone() {
            return Ok(driver);
        }

        Err(CommonError::CommonError("".to_string()))
    }

    async fn build_message_storage_driver(
        &self,
        storage_type: StorageAdapterType,
    ) -> Result<ArcStorageAdapter, CommonError> {
        let storage: ArcStorageAdapter = match storage_type {
            StorageAdapterType::Memory => {
                let engine = MemoryStorageEngine::create_standalone(
                    self.rocksdb_engine_handler.clone(),
                    self.storage_cache_manager.clone(),
                    StorageDriverMemoryConfig::default(),
                );
                Arc::new(MemoryStorageAdapter::new(Arc::new(engine)))
            }

            StorageAdapterType::Engine => {
                Arc::new(StorageEngineAdapter::new(self.engine_adapter_handler.clone()).await)
            }

            StorageAdapterType::RocksDB => {
                let engine = Arc::new(RocksDBStorageEngine::create_standalone(
                    self.storage_cache_manager.clone(),
                    self.rocksdb_engine_handler.clone(),
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
