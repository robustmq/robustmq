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

use common_base::error::common::CommonError;
use protocol::journal_server::journal_admin::{
    ListSegmentReply, ListSegmentRequest, ListShardReply, ListShardRequest,
};
use protocol::journal_server::journal_inner::{
    DeleteSegmentFileReply, DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest, UpdateJournalCacheReply, UpdateJournalCacheRequest,
};

use crate::pool::ClientPool;

pub mod admin;
pub mod inner;

#[derive(Clone)]
pub enum JournalEngineService {
    Admin,
    Inner,
}

#[derive(Clone, Debug)]
pub enum JournalEngineInterface {
    // inner
    UpdateCache,
    DeleteShardFile,
    GetShardDeleteStatus,
    DeleteSegmentFile,
    GetSegmentDeleteStatus,

    // admin
    ListShard,
    ListSegment,
}

#[derive(Debug, Clone)]
pub enum JournalEngineRequest {
    // inner
    UpdateCache(UpdateJournalCacheRequest),
    DeleteShardFile(DeleteShardFileRequest),
    GetShardDeleteStatus(GetShardDeleteStatusRequest),
    DeleteSegmentFileRequest(DeleteSegmentFileRequest),
    GetSegmentDeleteStatus(GetSegmentDeleteStatusRequest),

    // admin
    ListShard(ListShardRequest),
    ListSegment(ListSegmentRequest),
}

#[derive(Debug, Clone)]
pub enum JournalEngineReply {
    // inner
    UpdateCache(UpdateJournalCacheReply),
    DeleteShardFile(DeleteShardFileReply),
    GetShardDeleteStatus(GetShardDeleteStatusReply),
    DeleteSegmentFile(DeleteSegmentFileReply),
    GetSegmentDeleteStatus(GetSegmentDeleteStatusReply),

    // admin
    ListShard(ListShardReply),
    ListSegment(ListSegmentReply),
}

async fn call_once(
    client_pool: &ClientPool,
    addr: &str,
    request: JournalEngineRequest,
) -> Result<JournalEngineReply, CommonError> {
    use JournalEngineRequest::*;

    match request {
        UpdateCache(update_journal_cache_request) => {
            let mut client = client_pool.journal_inner_services_client(addr).await?;
            let reply = client.update_cache(update_journal_cache_request).await?;
            Ok(JournalEngineReply::UpdateCache(reply.into_inner()))
        }
        DeleteShardFile(delete_shard_file_request) => {
            let mut client = client_pool.journal_inner_services_client(addr).await?;
            let reply = client.delete_shard_file(delete_shard_file_request).await?;
            Ok(JournalEngineReply::DeleteShardFile(reply.into_inner()))
        }
        GetShardDeleteStatus(get_shard_delete_status_request) => {
            let mut client = client_pool.journal_inner_services_client(addr).await?;
            let reply = client
                .get_shard_delete_status(get_shard_delete_status_request)
                .await?;
            Ok(JournalEngineReply::GetShardDeleteStatus(reply.into_inner()))
        }
        DeleteSegmentFileRequest(delete_segment_file_request) => {
            let mut client = client_pool.journal_inner_services_client(addr).await?;
            let reply = client
                .delete_segment_file(delete_segment_file_request)
                .await?;
            Ok(JournalEngineReply::DeleteSegmentFile(reply.into_inner()))
        }
        GetSegmentDeleteStatus(get_segment_delete_status_request) => {
            let mut client = client_pool.journal_inner_services_client(addr).await?;
            let reply = client
                .get_segment_delete_status(get_segment_delete_status_request)
                .await?;
            Ok(JournalEngineReply::GetSegmentDeleteStatus(
                reply.into_inner(),
            ))
        }
        ListShard(list_shard_request) => {
            let mut client = client_pool.journal_admin_services_client(addr).await?;
            let reply = client.list_shard(list_shard_request).await?;
            Ok(JournalEngineReply::ListShard(reply.into_inner()))
        }
        ListSegment(list_segment_request) => {
            let mut client = client_pool.journal_admin_services_client(addr).await?;
            let reply = client.list_segment(list_segment_request).await?;
            Ok(JournalEngineReply::ListSegment(reply.into_inner()))
        }
    }
}
