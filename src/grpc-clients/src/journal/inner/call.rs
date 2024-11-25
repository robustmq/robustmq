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

use common_base::error::common::CommonError;
use prost::Message as _;
use protocol::journal_server::journal_inner::{
    DeleteSegmentFileReply, DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest, UpdateJournalCacheReply, UpdateJournalCacheRequest,
};

use crate::journal::{call_once, JournalEngineInterface, JournalEngineReply, JournalEngineRequest, JournalEngineService};
use crate::pool::ClientPool;
use crate::utils::retry_call;

pub async fn journal_inner_update_cache(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: UpdateJournalCacheRequest,
) -> Result<UpdateJournalCacheReply, CommonError> {
    let request = JournalEngineRequest::UpdateCache(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        JournalEngineReply::UpdateCache(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn journal_inner_delete_shard_file(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: DeleteShardFileRequest,
) -> Result<DeleteShardFileReply, CommonError> {
    let request = JournalEngineRequest::DeleteShardFile(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        JournalEngineReply::DeleteShardFile(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn journal_inner_get_shard_delete_status(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: GetShardDeleteStatusRequest,
) -> Result<GetShardDeleteStatusReply, CommonError> {
    let request = JournalEngineRequest::GetShardDeleteStatus(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        JournalEngineReply::GetShardDeleteStatus(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn journal_inner_delete_segment_file(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: DeleteSegmentFileRequest,
) -> Result<DeleteSegmentFileReply, CommonError> {
    let request = JournalEngineRequest::DeleteSegmentFileRequest(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        JournalEngineReply::DeleteSegmentFile(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn journal_inner_get_segment_delete_status(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: GetSegmentDeleteStatusRequest,
) -> Result<GetSegmentDeleteStatusReply, CommonError> {
    let request = JournalEngineRequest::GetSegmentDeleteStatus(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        JournalEngineReply::GetSegmentDeleteStatus(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}
