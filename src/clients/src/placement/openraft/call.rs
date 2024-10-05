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

use super::PlacementCenterInterface;
use crate::{
    placement::{retry_call, PlacementCenterService},
    poll::ClientPool,
};
use common_base::error::common::CommonError;
use prost::Message as _;
use protocol::placement_center::generate::openraft::{
    AppendReply, AppendRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest,
};
use std::sync::Arc;

pub async fn placement_openraft_vote(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: VoteRequest,
) -> Result<VoteReply, CommonError> {
    let request_data = VoteRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::OpenRaft,
        PlacementCenterInterface::Vote,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match VoteReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_openraft_append(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: AppendRequest,
) -> Result<AppendReply, CommonError> {
    let request_data = AppendRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::OpenRaft,
        PlacementCenterInterface::Append,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match AppendReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn placement_openraft_snapshot(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SnapshotRequest,
) -> Result<SnapshotReply, CommonError> {
    let request_data = SnapshotRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::OpenRaft,
        PlacementCenterInterface::Snapshot,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match SnapshotReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}
