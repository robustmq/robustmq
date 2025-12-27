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

use crate::core::{cache::StorageCacheManager, error::StorageEngineError};
use grpc_clients::pool::ClientPool;
use metadata_struct::{adapter::record::Record, storage::shard::EngineType};
use std::sync::Arc;

pub async fn _batch_write(
    _client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
    shard_name: &str,
    _records: &[Record],
) -> Result<Vec<u64>, StorageEngineError> {
    let Some(shard) = cache_manager.shards.get(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
    };

    let Some(_active_segment) = cache_manager.get_active_segment(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
    };

    let _offsets = match shard.engine_type {
        EngineType::Memory => write_memory().await?,
        EngineType::RocksDB => write_rocksdb().await?,
        EngineType::Segment => write_segment().await?,
    };
    Ok(Vec::new())
}

async fn write_memory() -> Result<Vec<u64>, StorageEngineError> {
    Ok(Vec::new())
}

async fn write_rocksdb() -> Result<Vec<u64>, StorageEngineError> {
    Ok(Vec::new())
}

async fn write_segment() -> Result<Vec<u64>, StorageEngineError> {
    Ok(Vec::new())
}
