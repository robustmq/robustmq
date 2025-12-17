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

use std::sync::Arc;

use protocol::broker::broker_storage::{
    DeleteSegmentFileReply, DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest, UpdateJournalCacheReply, UpdateJournalCacheRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;

use crate::core::cache::CacheManager;
use crate::core::error::JournalServerError;
use crate::core::notification::parse_notification;
use crate::core::segment::{delete_local_segment, segment_already_delete};
use crate::core::shard::{delete_local_shard, is_delete_by_shard};
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;

/// Update journal cache based on the request
pub async fn update_cache_by_req(
    cache_manager: &Arc<CacheManager>,
    segment_file_manager: &Arc<SegmentFileManager>,
    request: &UpdateJournalCacheRequest,
) -> Result<UpdateJournalCacheReply, JournalServerError> {
    parse_notification(
        cache_manager,
        segment_file_manager,
        request.action_type(),
        request.resource_type(),
        &request.data,
    )
    .await;

    Ok(UpdateJournalCacheReply::default())
}

/// Delete shard file based on the request
pub async fn delete_shard_file_by_req(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    request: &DeleteShardFileRequest,
) -> Result<DeleteShardFileReply, JournalServerError> {
    delete_local_shard(
        cache_manager.clone(),
        rocksdb_engine_handler.clone(),
        segment_file_manager.clone(),
        request.clone(),
    );

    Ok(DeleteShardFileReply::default())
}

/// Get shard delete status based on the request
pub async fn get_shard_delete_status_by_req(
    request: &GetShardDeleteStatusRequest,
) -> Result<GetShardDeleteStatusReply, JournalServerError> {
    let flag = is_delete_by_shard(request)?;
    Ok(GetShardDeleteStatusReply { status: flag })
}

/// Delete segment file based on the request
pub async fn delete_segment_file_by_req(
    cache_manager: &Arc<CacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    request: &DeleteSegmentFileRequest,
) -> Result<DeleteSegmentFileReply, JournalServerError> {
    let segment_iden = SegmentIdentity::new(&request.shard_name, request.segment);
    delete_local_segment(
        cache_manager,
        rocksdb_engine_handler,
        segment_file_manager,
        &segment_iden,
    )
    .await?;

    Ok(DeleteSegmentFileReply::default())
}

/// Get segment delete status based on the request
pub async fn get_segment_delete_status_by_req(
    cache_manager: &Arc<CacheManager>,
    request: &GetSegmentDeleteStatusRequest,
) -> Result<GetSegmentDeleteStatusReply, JournalServerError> {
    let flag = segment_already_delete(cache_manager, request).await?;
    Ok(GetSegmentDeleteStatusReply { status: flag })
}
