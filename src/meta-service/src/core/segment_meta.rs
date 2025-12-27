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
    controller::call_broker::{call::BrokerCallManager, storage::update_cache_by_set_segment_meta},
    core::{cache::CacheManager, error::MetaServiceError},
    raft::{
        manager::MultiRaftManager,
        route::data::{StorageData, StorageDataType},
    },
};
use bytes::Bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::{segment::EngineSegment, segment_meta::EngineSegmentMetadata};
use std::sync::Arc;
use tracing::warn;

const UNINITIALIZED_OFFSET: i64 = -1;
const UNINITIALIZED_TIMESTAMP: i64 = -1;

async fn update_segment_metadata<F>(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    segment: u32,
    update_fn: F,
) -> Result<(), MetaServiceError>
where
    F: FnOnce(&mut EngineSegmentMetadata),
{
    let meta_list = cache_manager
        .segment_meta_list
        .get(shard_name)
        .ok_or_else(|| {
            warn!("Shard '{}' not found in cache", shard_name);
            MetaServiceError::ShardDoesNotExist(shard_name.to_string())
        })?;

    let mut meta = meta_list.get_mut(&segment).ok_or_else(|| {
        warn!(
            "Segment {} not found for shard '{}' in cache",
            segment, shard_name
        );
        MetaServiceError::SegmentMetaDoesNotExist(format!("{},{}", shard_name, segment))
    })?;

    update_fn(&mut meta);
    sync_save_segment_metadata_info(raft_manager, &meta).await?;
    update_cache_by_set_segment_meta(call_manager, client_pool, (*meta).clone()).await?;

    Ok(())
}

pub async fn update_last_offset_by_segment_metadata(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    segment: u32,
    last_offset: i64,
) -> Result<(), MetaServiceError> {
    update_segment_metadata(
        cache_manager,
        raft_manager,
        call_manager,
        client_pool,
        shard_name,
        segment,
        |meta| meta.end_offset = last_offset,
    )
    .await
}

pub async fn update_start_timestamp_by_segment_metadata(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    segment: u32,
    start_timestamp: u64,
) -> Result<(), MetaServiceError> {
    let timestamp_i64 = start_timestamp
        .try_into()
        .map_err(|_| MetaServiceError::CommonError("Timestamp overflow".to_string()))?;

    update_segment_metadata(
        cache_manager,
        raft_manager,
        call_manager,
        client_pool,
        shard_name,
        segment,
        |meta| meta.start_timestamp = timestamp_i64,
    )
    .await
}

pub async fn update_end_timestamp_by_segment_metadata(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    segment: u32,
    end_timestamp: u64,
) -> Result<(), MetaServiceError> {
    let timestamp_i64 = end_timestamp
        .try_into()
        .map_err(|_| MetaServiceError::CommonError("Timestamp overflow".to_string()))?;

    update_segment_metadata(
        cache_manager,
        raft_manager,
        call_manager,
        client_pool,
        shard_name,
        segment,
        |meta| meta.end_timestamp = timestamp_i64,
    )
    .await
}

pub async fn create_segment_metadata(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    segment: &EngineSegment,
    start_offset: i64,
) -> Result<(), MetaServiceError> {
    if cache_manager
        .get_segment_meta(&segment.shard_name, segment.segment_seq)
        .is_some()
    {
        return Ok(());
    }

    let metadata = EngineSegmentMetadata {
        shard_name: segment.shard_name.clone(),
        segment_seq: segment.segment_seq,
        start_offset,
        end_offset: UNINITIALIZED_OFFSET,
        start_timestamp: UNINITIALIZED_TIMESTAMP,
        end_timestamp: UNINITIALIZED_TIMESTAMP,
    };

    sync_save_segment_metadata_info(raft_manager, &metadata).await?;
    update_cache_by_set_segment_meta(call_manager, client_pool, metadata).await?;

    Ok(())
}

async fn sync_save_segment_metadata_info(
    raft_manager: &Arc<MultiRaftManager>,
    meta: &EngineSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineSetSegmentMetadata,
        Bytes::copy_from_slice(&meta.encode()?),
    );

    raft_manager
        .write_metadata(data)
        .await?
        .ok_or(MetaServiceError::ExecutionResultIsEmpty)?;

    Ok(())
}

pub async fn sync_delete_segment_metadata_info(
    raft_manager: &Arc<MultiRaftManager>,
    meta: &EngineSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineDeleteSegmentMetadata,
        Bytes::copy_from_slice(&meta.encode()?),
    );

    raft_manager
        .write_metadata(data)
        .await?
        .ok_or(MetaServiceError::ExecutionResultIsEmpty)?;

    Ok(())
}
