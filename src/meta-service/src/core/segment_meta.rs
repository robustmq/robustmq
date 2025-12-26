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

pub async fn update_last_offset_by_segment_metadata(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    segment: u32,
    last_offset: i64,
) -> Result<(), MetaServiceError> {
    if let Some(meta_list) = cache_manager.segment_meta_list.get(shard_name) {
        if let Some(mut meta) = meta_list.get_mut(&segment) {
            meta.end_offset = last_offset;
            sync_save_segment_metadata_info(raft_manager, &meta).await?;
            update_cache_by_set_segment_meta(call_manager, client_pool, meta.clone()).await?;
        }
    }
    Ok(())
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
    if let Some(meta_list) = cache_manager.segment_meta_list.get(shard_name) {
        if let Some(mut meta) = meta_list.get_mut(&segment) {
            meta.start_timestamp = start_timestamp as i64;
            sync_save_segment_metadata_info(raft_manager, &meta).await?;
            update_cache_by_set_segment_meta(call_manager, client_pool, meta.clone()).await?;
        }
    }
    Ok(())
}

pub async fn update_end_timestamp_by_segment_metadata(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
    segment: u32,
    start_timestamp: u64,
) -> Result<(), MetaServiceError> {
    if let Some(meta_list) = cache_manager.segment_meta_list.get(shard_name) {
        if let Some(mut meta) = meta_list.get_mut(&segment) {
            meta.end_timestamp = start_timestamp as i64;
            sync_save_segment_metadata_info(raft_manager, &meta).await?;
            update_cache_by_set_segment_meta(call_manager, client_pool, meta.clone()).await?;
        }
    }
    Ok(())
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
        end_offset: -1,
        start_timestamp: -1,
        end_timestamp: -1,
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
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

async fn sync_delete_segment_metadata_info(
    raft_manager: &Arc<MultiRaftManager>,
    meta: &EngineSegmentMetadata,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineDeleteSegmentMetadata,
        Bytes::copy_from_slice(&meta.encode()?),
    );
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}
