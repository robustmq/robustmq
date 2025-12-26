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
use crate::controller::call_broker::storage::{
    update_cache_by_set_segment, update_cache_by_set_segment_meta, update_cache_by_set_shard,
};
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::core::segment::{
    create_segment, sync_save_segment_info, sync_save_segment_metadata_info, update_segment_status,
};
use crate::core::shard::update_last_segment_by_shard;
use crate::raft::manager::MultiRaftManager;
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::segment_meta::SegmentMetadataStorage;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::{str_to_segment_status, SegmentStatus};
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use protocol::meta::meta_service_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, DeleteSegmentReply, DeleteSegmentRequest,
    ListSegmentMetaReply, ListSegmentMetaRequest, ListSegmentReply, ListSegmentRequest,
    UpdateSegmentMetaReply, UpdateSegmentMetaRequest, UpdateSegmentStatusReply,
    UpdateSegmentStatusRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn list_segment_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSegmentRequest,
) -> Result<ListSegmentReply, MetaServiceError> {
    let segment_storage = SegmentStorage::new(rocksdb_engine_handler.clone());
    let binary_segments = if req.shard_name.is_empty() && req.segment_no == -1 {
        segment_storage.all_segment()?
    } else if !req.shard_name.is_empty() && req.segment_no == -1 {
        segment_storage.list_by_shard(&req.shard_name)?
    } else {
        match segment_storage.get(&req.shard_name, req.segment_no as u32)? {
            Some(segment) => vec![segment],
            None => Vec::new(),
        }
    };

    let segments_data = binary_segments
        .into_iter()
        .map(|segment| segment.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListSegmentReply {
        segments: segments_data,
    })
}

pub async fn create_segment_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &CreateNextSegmentRequest,
) -> Result<CreateNextSegmentReply, MetaServiceError> {
    let mut shard = if let Some(shard) = cache_manager.get_shard(&req.shard_name) {
        shard
    } else {
        return Err(MetaServiceError::ShardDoesNotExist(req.shard_name.clone()));
    };

    if req.current_segment as u32 != shard.active_segment_seq {
        return Err(MetaServiceError::CommonError("".to_string()));
    }

    let next_segment = (req.current_segment + 1) as u32;
    if shard.last_segment_seq >= next_segment {
        return Ok(CreateNextSegmentReply {});
    }

    // If the next Segment hasn't already been created, it triggers the creation of the next Segment
    if cache_manager
        .get_segment(&req.shard_name, next_segment)
        .is_none()
    {
        // create next segment
        let segment = create_segment(&shard, cache_manager, next_segment).await?;
        sync_save_segment_info(raft_manager, &segment).await?;

        // save next segment metadata
        let metadata = EngineSegmentMetadata {
            shard_name: segment.shard_name.clone(),
            segment_seq: segment.segment_seq,
            start_offset: req.current_segment_end_offset + 1,
            end_offset: -1,
            start_timestamp: -1,
            end_timestamp: -1,
        };
        sync_save_segment_metadata_info(raft_manager, &metadata).await?;

        // update last segment by shard
        update_last_segment_by_shard(raft_manager, cache_manager, &mut shard, next_segment).await?;

        // update last offset by current segment
        

        // update call cache
        update_cache_by_set_shard(call_manager, client_pool, shard).await?;
        update_cache_by_set_segment(call_manager, client_pool, segment.clone()).await?;
        update_cache_by_set_segment_meta(call_manager, client_pool, metadata).await?;
    }

    Ok(CreateNextSegmentReply {})
}

pub async fn delete_segment_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteSegmentRequest,
) -> Result<DeleteSegmentReply, MetaServiceError> {
    if cache_manager.get_shard(&req.shard_name).is_none() {
        return Err(MetaServiceError::ShardDoesNotExist(req.shard_name.clone()));
    };

    let mut segment =
        if let Some(segment) = cache_manager.get_segment(&req.shard_name, req.segment_seq) {
            segment
        } else {
            return Err(MetaServiceError::SegmentDoesNotExist(format!(
                "{}-{}",
                req.shard_name, req.segment_seq
            )));
        };

    if segment.status != SegmentStatus::SealUp {
        return Err(MetaServiceError::NoAllowDeleteSegment(
            segment.name(),
            segment.status.to_string(),
        ));
    }

    update_segment_status(
        cache_manager,
        raft_manager,
        &segment,
        SegmentStatus::PreDelete,
    )
    .await?;

    segment.status = SegmentStatus::PreDelete;
    cache_manager.add_wait_delete_segment(&segment);

    update_cache_by_set_segment(call_manager, client_pool, segment.clone()).await?;

    Ok(DeleteSegmentReply::default())
}

pub async fn update_segment_status_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &UpdateSegmentStatusRequest,
) -> Result<UpdateSegmentStatusReply, MetaServiceError> {
    let mut segment =
        if let Some(segment) = cache_manager.get_segment(&req.shard_name, req.segment_seq) {
            segment
        } else {
            return Err(MetaServiceError::SegmentDoesNotExist(format!(
                "{}_{}",
                req.shard_name, req.segment_seq
            )));
        };

    if segment.status.to_string() != req.cur_status {
        return Err(MetaServiceError::SegmentStateError(
            segment.name(),
            segment.status.to_string(),
            req.cur_status.clone(),
        ));
    }

    let new_status = str_to_segment_status(&req.next_status)?;
    segment.status = new_status;

    sync_save_segment_info(raft_manager, &segment).await?;
    update_cache_by_set_segment(call_manager, client_pool, segment.clone()).await?;
    Ok(UpdateSegmentStatusReply::default())
}

pub async fn list_segment_meta_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListSegmentMetaRequest,
) -> Result<ListSegmentMetaReply, MetaServiceError> {
    let storage = SegmentMetadataStorage::new(rocksdb_engine_handler.clone());
    let binary_segments = if req.shard_name.is_empty() && req.segment_no == -1 {
        storage.all_segment()?
    } else if !req.shard_name.is_empty() && req.segment_no == -1 {
        storage.list_by_shard(&req.shard_name)?
    } else {
        match storage.get(&req.shard_name, req.segment_no as u32)? {
            Some(segment_meta) => vec![segment_meta],
            None => Vec::new(),
        }
    };

    let segments_data = binary_segments
        .into_iter()
        .map(|segment| segment.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(ListSegmentMetaReply {
        segments: segments_data,
    })
}

pub async fn update_segment_meta_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &UpdateSegmentMetaRequest,
) -> Result<UpdateSegmentMetaReply, MetaServiceError> {
    if req.shard_name.is_empty() {
        return Err(MetaServiceError::RequestParamsNotEmpty(
            " shard_name".to_string(),
        ));
    }

    if cache_manager
        .get_segment(&req.shard_name, req.segment_no)
        .is_none()
    {
        return Err(MetaServiceError::SegmentDoesNotExist(format!(
            "{}_{}",
            req.shard_name, req.segment_no
        )));
    };

    let mut segment_meta =
        if let Some(meta) = cache_manager.get_segment_meta(&req.shard_name, req.segment_no) {
            meta
        } else {
            return Err(MetaServiceError::SegmentMetaDoesNotExist(format!(
                "{}_{}",
                req.shard_name, req.segment_no
            )));
        };

    if req.start_offset > 0 {
        segment_meta.start_offset = req.start_offset;
    }

    if req.end_offset > 0 {
        segment_meta.end_offset = req.end_offset;
    }

    if req.start_timestamp > 0 {
        segment_meta.start_timestamp = req.start_timestamp;
    }

    if req.end_timestamp > 0 {
        segment_meta.end_timestamp = req.end_timestamp;
    }

    sync_save_segment_metadata_info(raft_manager, &segment_meta).await?;

    update_cache_by_set_segment_meta(call_manager, client_pool, segment_meta).await?;

    Ok(UpdateSegmentMetaReply::default())
}
