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
use protocol::placement_center::placement_center_openraft::{
    AddLearnerReply, AddLearnerRequest, AppendReply, AppendRequest, ChangeMembershipReply,
    ChangeMembershipRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest,
};

use super::PlacementCenterInterface;
use crate::placement::{retry_call, PlacementCenterService};
use crate::pool::ClientPool;

pub async fn placement_openraft_vote(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: VoteRequest,
) -> Result<VoteReply, CommonError> {
    let request_data = VoteRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::OpenRaft,
        PlacementCenterInterface::Vote,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match VoteReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn placement_openraft_append(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: AppendRequest,
) -> Result<AppendReply, CommonError> {
    let request_data = AppendRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::OpenRaft,
        PlacementCenterInterface::Append,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match AppendReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn placement_openraft_snapshot(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SnapshotRequest,
) -> Result<SnapshotReply, CommonError> {
    let request_data = SnapshotRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::OpenRaft,
        PlacementCenterInterface::Snapshot,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match SnapshotReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn placement_openraft_add_learner(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: AddLearnerRequest,
) -> Result<AddLearnerReply, CommonError> {
    let request_data = AddLearnerRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::OpenRaft,
        PlacementCenterInterface::AddLearner,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match AddLearnerReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn placement_openraft_change_membership(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: ChangeMembershipRequest,
) -> Result<ChangeMembershipReply, CommonError> {
    let request_data = ChangeMembershipRequest::encode_to_vec(&request);
    match retry_call(
        PlacementCenterService::OpenRaft,
        PlacementCenterInterface::ChangeMembership,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match ChangeMembershipReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}
