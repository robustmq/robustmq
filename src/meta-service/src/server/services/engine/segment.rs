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
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::core::segment::{create_segment, seal_up_segment, update_segment_status};
use crate::core::segment_meta::{
    update_last_offset_by_segment_metadata, update_start_timestamp_by_segment_metadata,
};
use crate::core::shard::update_last_segment_by_shard;
use crate::raft::manager::MultiRaftManager;
use crate::storage::journal::segment::SegmentStorage;
use crate::storage::journal::segment_meta::SegmentMetadataStorage;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::SegmentStatus;
use protocol::meta::meta_service_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, DeleteSegmentReply, DeleteSegmentRequest,
    ListSegmentMetaReply, ListSegmentMetaRequest, ListSegmentReply, ListSegmentRequest,
    SealUpSegmentReply, SealUpSegmentRequest, UpdateStartTimeBySegmentMetaReply,
    UpdateStartTimeBySegmentMetaRequest,
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
    let shard = if let Some(shard) = cache_manager.shard_list.get(&req.shard_name) {
        shard.clone()
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

    create_segment(
        cache_manager,
        raft_manager,
        call_manager,
        client_pool,
        &shard,
        next_segment,
        (req.current_segment_end_offset + 1) as u64,
    )
    .await?;

    update_last_segment_by_shard(
        raft_manager,
        cache_manager,
        call_manager,
        client_pool,
        &shard.shard_name,
        next_segment,
    )
    .await?;

    update_last_offset_by_segment_metadata(
        cache_manager,
        raft_manager,
        call_manager,
        client_pool,
        &req.shard_name,
        req.current_segment as u32,
        req.current_segment_end_offset,
    )
    .await?;

    Ok(CreateNextSegmentReply {})
}

pub async fn delete_segment_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &DeleteSegmentRequest,
) -> Result<DeleteSegmentReply, MetaServiceError> {
    if !cache_manager.shard_list.contains_key(&req.shard_name) {
        return Err(MetaServiceError::ShardDoesNotExist(req.shard_name.clone()));
    };

    let segment = if let Some(segment) = cache_manager.get_segment(&req.shard_name, req.segment_seq)
    {
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
        call_manager,
        raft_manager,
        client_pool,
        &segment.shard_name,
        segment.segment_seq,
        SegmentStatus::PreDelete,
    )
    .await?;

    cache_manager.add_wait_delete_segment(&segment);
    Ok(DeleteSegmentReply::default())
}

pub async fn seal_up_segment_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &SealUpSegmentRequest,
) -> Result<SealUpSegmentReply, MetaServiceError> {
    if !cache_manager.shard_list.contains_key(&req.shard_name) {
        return Err(MetaServiceError::ShardDoesNotExist(req.shard_name.clone()));
    };

    let segment = if let Some(segment) = cache_manager.get_segment(&req.shard_name, req.segment_seq)
    {
        segment
    } else {
        return Err(MetaServiceError::SegmentDoesNotExist(format!(
            "{}-{}",
            req.shard_name, req.segment_seq
        )));
    };

    if segment.status != SegmentStatus::Write {
        return Err(MetaServiceError::CommonError("".to_string()));
    }

    seal_up_segment(
        cache_manager,
        raft_manager,
        call_manager,
        client_pool,
        &segment,
        req.end_timestamp,
    )
    .await?;

    Ok(SealUpSegmentReply::default())
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

pub async fn update_start_time_by_segment_meta_by_req(
    cache_manager: &Arc<CacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &UpdateStartTimeBySegmentMetaRequest,
) -> Result<UpdateStartTimeBySegmentMetaReply, MetaServiceError> {
    if !cache_manager.shard_list.contains_key(&req.shard_name) {
        return Err(MetaServiceError::ShardDoesNotExist(req.shard_name.clone()));
    };

    if cache_manager
        .get_segment(&req.shard_name, req.segment_no)
        .is_none()
    {
        return Err(MetaServiceError::SegmentDoesNotExist(format!(
            "{}_{}",
            req.shard_name, req.segment_no
        )));
    };

    update_start_timestamp_by_segment_metadata(
        cache_manager,
        raft_manager,
        call_manager,
        client_pool,
        &req.shard_name,
        req.segment_no,
        req.start_timestamp,
    )
    .await?;
    Ok(UpdateStartTimeBySegmentMetaReply::default())
}
