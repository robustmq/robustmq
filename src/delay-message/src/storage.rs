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

use std::sync::Arc;

use common_base::error::common::CommonError;
use common_config::storage::StorageAdapterType;
use metadata_struct::storage::shard::EngineShardConfig;
use storage_adapter::driver::{ArcStorageAdapter, StorageDriverManager};

pub fn get_delay_message_storage_driver(
    storage_driver_manager: &Arc<StorageDriverManager>,
    engine_storage_type: &StorageAdapterType,
) -> Result<ArcStorageAdapter, CommonError> {
    match engine_storage_type {
        StorageAdapterType::Memory => Ok(storage_driver_manager.memory_storage.clone()),
        StorageAdapterType::RocksDB => Ok(storage_driver_manager.rocksdb_storage.clone()),
        StorageAdapterType::Engine => Ok(storage_driver_manager.engine_storage.clone()),
        _ => Err(CommonError::CommonError(format!(
            "Unsupported storage adapter type '{:?}' for delay message storage",
            engine_storage_type
        ))),
    }
}

pub fn build_delay_message_shard_config(
    engine_storage_type: &StorageAdapterType,
) -> Result<EngineShardConfig, CommonError> {
    match engine_storage_type {
        StorageAdapterType::Memory => Ok(EngineShardConfig {
            replica_num: 1,
            max_segment_size: 1073741824,
            retention_sec: 86400,
            storage_adapter_type: StorageAdapterType::Memory,
            engine_storage_type: None,
        }),
        StorageAdapterType::RocksDB => Ok(EngineShardConfig {
            replica_num: 1,
            max_segment_size: 1073741824,
            retention_sec: 86400,
            storage_adapter_type: StorageAdapterType::RocksDB,
            engine_storage_type: None,
        }),
        StorageAdapterType::Engine => Ok(EngineShardConfig {
            replica_num: 1,
            max_segment_size: 1073741824,
            retention_sec: 86400,
            storage_adapter_type: StorageAdapterType::RocksDB,
            engine_storage_type: None,
        }),
        _ => Err(CommonError::CommonError(format!(
            "Unsupported storage adapter type '{:?}' for delay message shard config",
            engine_storage_type
        ))),
    }
}
