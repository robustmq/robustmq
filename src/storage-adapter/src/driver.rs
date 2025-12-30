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
    memory::MemoryStorageAdapter, rocksdb::RocksDBStorageAdapter, storage::ArcStorageAdapter,
};
use common_base::error::common::CommonError;
use common_config::storage::{
    memory::StorageDriverMemoryConfig, StorageAdapterConfig, StorageAdapterType,
};
use std::sync::Arc;
use storage_engine::{memory::engine::MemoryStorageEngine, rocksdb::engine::RocksDBStorageEngine};

pub async fn build_message_storage_driver(
    rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
    config: StorageAdapterConfig,
) -> Result<ArcStorageAdapter, CommonError> {
    let storage: ArcStorageAdapter = match config.storage_type {
        StorageAdapterType::Memory => {
            let engine = MemoryStorageEngine::create_full(StorageDriverMemoryConfig::default());
            Arc::new(MemoryStorageAdapter::new(Arc::new(engine)))
        }

        // StorageAdapterType::Journal => Arc::new(
        //     JournalStorageAdapter::new(offset_manager, config.journal_config.unwrap_or_default())
        //         .await?,
        // ),

        // StorageAdapterType::Mysql => Arc::new(MySQLStorageAdapter::new(
        //     config.mysql_config.unwrap_or_default(),
        // )?),
        StorageAdapterType::RocksDB => {
            Arc::new(RocksDBStorageAdapter::new(rocksdb_storage_engine.clone()))
        }

        StorageAdapterType::S3 => {
            // Arc::new(S3StorageAdapter::new(config.s3_config.unwrap_or_default()))
            return Err(CommonError::UnavailableStorageType);
        }

        StorageAdapterType::MinIO => {
            // Arc::new(MinIoStorageAdapter::new(
            // config.minio_config.unwrap_or_default(),
            // )?)
            return Err(CommonError::UnavailableStorageType);
        }
    };

    Ok(storage)
}
