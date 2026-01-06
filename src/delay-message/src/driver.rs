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
use common_config::storage::StorageType;
use metadata_struct::storage::shard::{EngineShardConfig, EngineStorageType};

pub fn build_delay_message_shard_config(
    engine_storage_type: &StorageType,
) -> Result<EngineShardConfig, CommonError> {
    match engine_storage_type {
        StorageType::EngineMemory => Ok(EngineShardConfig {
            replica_num: 1,
            max_segment_size: 1073741824,
            retention_sec: 86400,
            engine_storage_type: None,
        }),
        StorageType::EngineRocksDB => Ok(EngineShardConfig {
            replica_num: 1,
            max_segment_size: 1073741824,
            retention_sec: 86400,
            engine_storage_type: None,
        }),
        StorageType::EngineSegment => Ok(EngineShardConfig {
            replica_num: 1,
            max_segment_size: 1073741824,
            retention_sec: 86400,
            engine_storage_type: Some(EngineStorageType::EngineRocksDB),
        }),
        _ => Err(CommonError::CommonError(format!(
            "Unsupported storage adapter type '{:?}' for delay message shard config",
            engine_storage_type
        ))),
    }
}
