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

use crate::controller::call_broker::call::BrokerCallManager;
use crate::controller::call_broker::storage::update_cache_by_set_shard;
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::core::segment::delete_segment_by_real;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use bytes::Bytes;
use common_base::tools::{now_second, unique_id};
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::shard::{
    EngineShard, EngineShardConfig, EngineShardStatus, EngineType,
};
use std::sync::Arc;
use tracing::info;

pub async fn create_shard(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    shard_config: EngineShardConfig,
) -> Result<EngineShard, MetaServiceError> {
    if let Some(existing_shard) = cache_manager.shard_list.get(shard_name) {
        return Ok(existing_shard.clone());
    }

    info!(
        "Creating shard: name={}, replica_num={}, max_segment_size={}",
        shard_name, shard_config.replica_num, shard_config.max_segment_size
    );

    let new_shard = EngineShard {
        shard_uid: unique_id(),
        shard_name: shard_name.to_string(),
        start_segment_seq: 0,
        active_segment_seq: 0,
        last_segment_seq: 0,
        status: EngineShardStatus::Run,
        config: shard_config.clone(),
        replica_num: shard_config.replica_num,
        engine_type: EngineType::Segment,
        create_time: now_second(),
    };

    sync_save_shard_info(raft_manager, &new_shard).await?;
    update_cache_by_set_shard(call_manager, client_pool, new_shard.clone()).await?;

    Ok(new_shard)
}

pub async fn delete_shard_by_real(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    shard_name: &str,
) -> Result<(), MetaServiceError> {
    let shard = match cache_manager.shard_list.get(shard_name) {
        Some(s) => s.clone(),
        None => return Ok(()),
    };

    info!("Deleting shard: name={}", shard_name);

    for segment in cache_manager.get_segment_list_by_shard(shard_name) {
        delete_segment_by_real(cache_manager, raft_manager, &segment).await?;
    }

    sync_delete_shard_info(raft_manager, &shard).await?;
    cache_manager.remove_shard(shard_name);

    Ok(())
}

async fn update_shard<F>(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<CacheManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    update_fn: F,
) -> Result<(), MetaServiceError>
where
    F: FnOnce(&mut EngineShard),
{
    if let Some(mut shard) = cache_manager.shard_list.get_mut(shard_name) {
        update_fn(&mut shard);
        let shard_clone = shard.clone();
        drop(shard);

        sync_save_shard_info(raft_manager, &shard_clone).await?;
        update_cache_by_set_shard(call_manager, client_pool, shard_clone).await?;
    }

    Ok(())
}

pub async fn update_start_segment_by_shard(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<CacheManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    segment_no: u32,
) -> Result<(), MetaServiceError> {
    info!(
        "Updating shard start segment: name={}, segment={}",
        shard_name, segment_no
    );

    update_shard(
        raft_manager,
        cache_manager,
        call_manager,
        client_pool,
        shard_name,
        |shard| shard.start_segment_seq = segment_no,
    )
    .await
}

pub async fn update_last_segment_by_shard(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<CacheManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    segment_no: u32,
) -> Result<(), MetaServiceError> {
    info!(
        "Updating shard last segment: name={}, segment={}",
        shard_name, segment_no
    );

    update_shard(
        raft_manager,
        cache_manager,
        call_manager,
        client_pool,
        shard_name,
        |shard| shard.last_segment_seq = segment_no,
    )
    .await
}

pub async fn update_shard_status(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<CacheManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    status: EngineShardStatus,
) -> Result<(), MetaServiceError> {
    info!(
        "Updating shard status: name={}, new_status={:?}",
        shard_name, status
    );

    update_shard(
        raft_manager,
        cache_manager,
        call_manager,
        client_pool,
        shard_name,
        |shard| shard.status = status,
    )
    .await
}

async fn sync_save_shard_info(
    raft_manager: &Arc<MultiRaftManager>,
    shard: &EngineShard,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineSetShard,
        Bytes::from(shard.encode()?),
    );

    raft_manager
        .write_metadata(data)
        .await?
        .ok_or(MetaServiceError::ExecutionResultIsEmpty)?;

    Ok(())
}

async fn sync_delete_shard_info(
    raft_manager: &Arc<MultiRaftManager>,
    shard: &EngineShard,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineDeleteShard,
        Bytes::from(shard.encode()?),
    );

    raft_manager
        .write_metadata(data)
        .await?
        .ok_or(MetaServiceError::ExecutionResultIsEmpty)?;

    Ok(())
}
