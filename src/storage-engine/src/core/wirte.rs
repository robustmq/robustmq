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
use metadata_struct::{
    adapter::record::Record,
    storage::{segment::EngineSegment, shard::EngineShard},
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn _batch_write(
    _client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
    shard: &str,
    _records: &[Record],
) -> Result<Vec<u64>, StorageEngineError> {
    let _shard_info = if let Some(shard_info) = cache_manager.shards.get(shard) {
        shard_info.clone()
    } else {
        return Err(StorageEngineError::ShardNotExist(shard.to_string()));
    };

    // let segment = get_active_segment(client_pool, cache_manager, shard).await?;
    // let config = broker_config();
    // if segment.leader == config.broker_id {}
    Ok(Vec::new())
}

async fn _write_to_local(
    _client_pool: &Arc<ClientPool>,
    _cache_manager: &Arc<StorageCacheManager>,
    _rocksdb_engine_handler: &Arc<RocksDBEngine>,
    _shard_info: &EngineShard,
) -> Result<Vec<u64>, StorageEngineError> {
    // let req_body = WriteReqBody {};
    // match shard_info.engine_type {
    //     EngineType::Memory => {}
    //     EngineType::Segment => {
    //         let resp = write_data_req(
    //             cache_manager,
    //             rocksdb_engine_handler,
    //             segment_file_manager,
    //             client_pool,
    //             &req_body,
    //         )
    //         .await?;
    //     }
    // }
    Ok(Vec::new())
}

async fn _write_to_leader() {}

async fn _get_active_segment(
    _client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
    shard: &str,
) -> Result<EngineSegment, StorageEngineError> {
    if let Some(segment) = cache_manager.get_active_segment(shard) {
        return Ok(segment);
    }

    Err(StorageEngineError::NotActiveSegment(shard.to_string()))
}
