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
use protocol::journal_server::journal_admin::{
    ListSegmentReply, ListSegmentRequest, ListShardReply, ListShardRequest,
};

use crate::journal::{retry_call, JournalEngineInterface, JournalEngineService};
use crate::pool::ClientPool;

pub async fn journal_admin_list_shard(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListShardRequest,
) -> Result<ListShardReply, CommonError> {
    let request_data = ListShardRequest::encode_to_vec(&request);
    match retry_call(
        JournalEngineService::Admin,
        JournalEngineInterface::ListShard,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ListShardReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn journal_admin_list_segment(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ListSegmentRequest,
) -> Result<ListSegmentReply, CommonError> {
    let request_data = ListSegmentRequest::encode_to_vec(&request);
    match retry_call(
        JournalEngineService::Admin,
        JournalEngineInterface::ListSegment,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ListSegmentReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}
