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
use common_base::tools::{now_millis, now_second};
use common_config::broker::broker_config;
use grpc_clients::{meta::journal::call::create_next_segment, pool::ClientPool};
use metadata_struct::{adapter::record::Record, storage::segment::EngineSegment};
use protocol::meta::meta_service_journal::CreateNextSegmentRequest;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;

pub async fn batch_write(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
    shard: &str,
    records: &[Record],
) -> Result<Vec<u64>, StorageEngineError> {
    let shard_info = if let Some(shard_info) = cache_manager.shards.get(shard) {
        shard_info.clone()
    } else {
        return Err(StorageEngineError::ShardNotExist(shard.to_string()));
    };

    let segment = get_active_segment(client_pool, cache_manager, shard).await?;
    let config = broker_config();
    if segment.leader == config.broker_id {}
    Ok(Vec::new())
}

async fn write_to_local() {}

async fn write_to_leader() {}

async fn get_active_segment(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
    shard: &str,
) -> Result<EngineSegment, StorageEngineError> {
    if let Some(segment) = cache_manager.get_active_segment(shard) {
        return Ok(segment);
    }

    let config = broker_config();
    let request = CreateNextSegmentRequest {
        shard_name: shard.to_string(),
    };
    create_next_segment(client_pool, &config.get_meta_service_addr(), request).await?;
    let start_time = now_second();
    loop {
        if let Some(segment) = cache_manager.get_active_segment(shard) {
            return Ok(segment);
        }

        if now_second() - start_time >= 3 {
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }
    Err(StorageEngineError::NotActiveSegment(shard.to_string()))
}
