use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use bytes::Bytes;
use metadata_struct::storage::shard::{EngineShard, EngineShardStatus};
use std::sync::Arc;

pub async fn update_start_segment_by_shard(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<CacheManager>,
    shard: &mut EngineShard,
    segment_no: u32,
) -> Result<(), MetaServiceError> {
    shard.start_segment_seq = segment_no;
    sync_save_shard_info(raft_manager, shard).await?;
    cache_manager.set_shard(shard);
    Ok(())
}

pub async fn update_last_segment_by_shard(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<CacheManager>,
    shard: &mut EngineShard,
    segment_no: u32,
) -> Result<(), MetaServiceError> {
    shard.last_segment_seq = segment_no;
    sync_save_shard_info(raft_manager, shard).await?;
    cache_manager.set_shard(shard);
    Ok(())
}

pub async fn sync_save_shard_info(
    raft_manager: &Arc<MultiRaftManager>,
    shard: &EngineShard,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineSetShard,
        Bytes::copy_from_slice(&shard.encode()?),
    );
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_shard_info(
    raft_manager: &Arc<MultiRaftManager>,
    shard: &EngineShard,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::StorageEngineDeleteShard,
        Bytes::copy_from_slice(&shard.encode()?),
    );
    if (raft_manager.write_metadata(data).await?).is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn update_shard_status(
    raft_manager: &Arc<MultiRaftManager>,
    cache_manager: &Arc<CacheManager>,
    shard: &EngineShard,
    status: EngineShardStatus,
) -> Result<(), MetaServiceError> {
    let mut new_shard = shard.clone();
    new_shard.status = status;
    sync_save_shard_info(raft_manager, &new_shard).await?;
    cache_manager.set_shard(&new_shard);
    Ok(())
}
