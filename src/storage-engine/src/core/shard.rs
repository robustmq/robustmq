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

use super::cache::StorageCacheManager;
use super::error::StorageEngineError;
use crate::filesegment::SegmentIdentity;
use common_config::{broker::broker_config, storage::StorageType};
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::adapter_offset::AdapterShardInfo;
use metadata_struct::storage::shard::EngineShard;
use protocol::meta::meta_service_journal::{
    CreateShardRequest, DeleteShardRequest, ListShardRequest,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{debug, info};

fn is_shard_ready(cache_manager: &Arc<StorageCacheManager>, shard: &AdapterShardInfo) -> bool {
    let shard_name = &shard.shard_name;
    if !cache_manager.shards.contains_key(shard_name) {
        return false;
    }
    let segment_iden = SegmentIdentity::new(shard_name, 0);
    if cache_manager.get_segment(&segment_iden).is_none() {
        return false;
    }
    if shard.config.storage_type == StorageType::EngineSegment {
        return cache_manager.get_segment_meta(&segment_iden).is_some();
    }
    true
}

pub async fn create_shard_to_place(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    shard: &AdapterShardInfo,
) -> Result<(), StorageEngineError> {
    is_support_storage_type(shard.config.storage_type)?;

    let shard_name = &shard.shard_name;

    if is_shard_ready(cache_manager, shard) {
        debug!(
            "Shard {} already provisioned, skipping creation",
            shard_name
        );
        return Ok(());
    }

    let conf: &common_config::config::BrokerConfig = broker_config();
    let request = CreateShardRequest {
        shard_name: shard_name.to_string(),
        topic_name: shard.topic_name.to_string(),
        shard_config: shard.config.encode()?,
        desc: shard.desc.to_string(),
    };

    grpc_clients::meta::storage::call::create_shard(
        client_pool,
        &conf.get_meta_service_addr(),
        request,
    )
    .await?;

    // Wait for the shard to be ready: shard and segment-0 populated in local cache.
    const SHARD_READY_TIMEOUT_SECS: u64 = 10;
    let wait_result = timeout(Duration::from_secs(SHARD_READY_TIMEOUT_SECS), async {
        loop {
            if is_shard_ready(cache_manager, shard) {
                return;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await;

    match wait_result {
        Ok(_) => {
            info!(
                "Shard {} created successfully, shard info:{:?}",
                shard_name, shard
            );
            Ok(())
        }
        Err(_) => Err(StorageEngineError::CommonErrorStr(format!(
            "Timeout waiting for shard '{}' (topic '{}') to be created in local cache after {}s",
            shard_name, shard.topic_name, SHARD_READY_TIMEOUT_SECS
        ))),
    }
}

pub async fn delete_shard_to_place(
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
) -> Result<(), StorageEngineError> {
    let conf = broker_config();
    let request = DeleteShardRequest {
        shard_name: shard_name.to_string(),
    };

    grpc_clients::meta::storage::call::delete_shard(
        client_pool,
        &conf.get_meta_service_addr(),
        request,
    )
    .await?;
    Ok(())
}

pub async fn list_shards(
    client_pool: &Arc<ClientPool>,
) -> Result<Vec<EngineShard>, StorageEngineError> {
    let conf = broker_config();
    let request = ListShardRequest {
        ..Default::default()
    };
    let mut stream = grpc_clients::meta::storage::call::list_shard(
        client_pool,
        &conf.get_meta_service_addr(),
        request,
    )
    .await?;
    let mut shards = Vec::new();
    while let Some(reply) = stream
        .message()
        .await
        .map_err(|e| StorageEngineError::CommonErrorStr(e.to_string()))?
    {
        shards.push(EngineShard::decode(&reply.shard)?);
    }
    Ok(shards)
}

pub fn is_support_storage_type(storage_type: StorageType) -> Result<(), StorageEngineError> {
    if storage_type == StorageType::EngineMemory
        || storage_type == StorageType::EngineRocksDB
        || storage_type == StorageType::EngineSegment
    {
        return Ok(());
    }

    Err(StorageEngineError::UnsupportedStorageType(format!(
        "{storage_type:?}"
    )))
}
