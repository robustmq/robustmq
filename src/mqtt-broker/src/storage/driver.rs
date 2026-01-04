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
use common_config::{broker::broker_config, storage::StorageAdapterType};
use storage_adapter::driver::{ArcStorageAdapter, StorageDriverManager};

pub fn get_driver_by_mqtt_topic_name(
    storage_driver_manager: &Arc<StorageDriverManager>,
    _topic_name: &str,
) -> Result<ArcStorageAdapter, CommonError> {
    let conf = broker_config();
    get_driver_by_type(storage_driver_manager, conf.message_storage.storage_type)
}

pub fn get_driver_by_system_topic_name(
    storage_driver_manager: &Arc<StorageDriverManager>,
) -> Result<ArcStorageAdapter, CommonError> {
    let conf = broker_config();
    get_driver_by_type(storage_driver_manager, conf.message_storage.storage_type)
}

fn get_driver_by_type(
    storage_driver_manager: &Arc<StorageDriverManager>,
    storage_type: StorageAdapterType,
) -> Result<ArcStorageAdapter, CommonError> {
    match storage_type {
        StorageAdapterType::Memory => Ok(storage_driver_manager.memory_storage.clone()),
        StorageAdapterType::RocksDB => Ok(storage_driver_manager.rocksdb_storage.clone()),
        StorageAdapterType::Engine => Ok(storage_driver_manager.engine_storage.clone()),
        _ => Err(CommonError::CommonError(format!(
            "Unsupported storage adapter type '{:?}' for delay message storage",
            storage_type
        ))),
    }
}
