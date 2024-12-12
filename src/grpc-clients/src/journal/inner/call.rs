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
use protocol::journal_server::journal_inner::{
    DeleteSegmentFileReply, DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest, UpdateJournalCacheReply, UpdateJournalCacheRequest,
};

use crate::pool::ClientPool;
use crate::utils::retry_call;

pub async fn journal_inner_update_cache(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: UpdateJournalCacheRequest,
) -> Result<UpdateJournalCacheReply, CommonError> {
    retry_call(&client_pool, addrs, request).await
}

pub async fn journal_inner_delete_shard_file(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: DeleteShardFileRequest,
) -> Result<DeleteShardFileReply, CommonError> {
    retry_call(&client_pool, addrs, request).await
}

pub async fn journal_inner_get_shard_delete_status(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: GetShardDeleteStatusRequest,
) -> Result<GetShardDeleteStatusReply, CommonError> {
    retry_call(&client_pool, addrs, request).await
}

pub async fn journal_inner_delete_segment_file(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: DeleteSegmentFileRequest,
) -> Result<DeleteSegmentFileReply, CommonError> {
    retry_call(&client_pool, addrs, request).await
}

pub async fn journal_inner_get_segment_delete_status(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: GetSegmentDeleteStatusRequest,
) -> Result<GetSegmentDeleteStatusReply, CommonError> {
    retry_call(&client_pool, addrs, request).await
}
