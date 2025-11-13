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

use crate::core::error::MetaServiceError;
use crate::raft::raft_node::Node;
use crate::raft::type_config::TypeConfig;
use bincode::{deserialize, serialize};
use openraft::Raft;
use protocol::meta::meta_service_openraft::{
    AddLearnerReply, AddLearnerRequest, AppendReply, AppendRequest, ChangeMembershipReply,
    ChangeMembershipRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest,
};

// Deserialize directly from slice to avoid unnecessary copies
fn deserialize_from_slice<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, MetaServiceError> {
    deserialize(bytes).map_err(|e| MetaServiceError::CommonError(e.to_string()))
}

pub async fn vote_by_req(
    raft_node: &Raft<TypeConfig>,
    req: &VoteRequest,
) -> Result<VoteReply, MetaServiceError> {
    let vote_data = deserialize_from_slice(&req.value)?;
    raft_node
        .vote(vote_data)
        .await
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))
        .and_then(|res| {
            serialize(&res)
                .map_err(|e| MetaServiceError::CommonError(e.to_string()))
                .map(|value| VoteReply { value })
        })
}

pub async fn append_by_req(
    raft_node: &Raft<TypeConfig>,
    req: &AppendRequest,
) -> Result<AppendReply, MetaServiceError> {
    let append_data = deserialize_from_slice(&req.value)?;
    raft_node
        .append_entries(append_data)
        .await
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))
        .and_then(|res| {
            serialize(&res)
                .map_err(|e| MetaServiceError::CommonError(e.to_string()))
                .map(|value| AppendReply { value })
        })
}

pub async fn snapshot_by_req(
    raft_node: &Raft<TypeConfig>,
    req: &SnapshotRequest,
) -> Result<SnapshotReply, MetaServiceError> {
    let snapshot_data = deserialize_from_slice(&req.value)?;
    raft_node
        .install_snapshot(snapshot_data)
        .await
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))
        .and_then(|res| {
            serialize(&res)
                .map_err(|e| MetaServiceError::CommonError(e.to_string()))
                .map(|value| SnapshotReply { value })
        })
}

pub async fn add_learner_by_req(
    raft_node: &Raft<TypeConfig>,
    req: &AddLearnerRequest,
) -> Result<AddLearnerReply, MetaServiceError> {
    let node_id = req.node_id;
    let node = req
        .node
        .clone()
        .ok_or(MetaServiceError::RequestParamsNotEmpty("node".to_string()))?;

    let raft_node_data = Node {
        rpc_addr: node.rpc_addr,
        node_id: node.node_id,
    };

    let blocking = req.blocking;
    let res = raft_node
        .add_learner(node_id, raft_node_data, blocking)
        .await?;
    let value = serialize(&res)?;

    Ok(AddLearnerReply { value })
}

pub async fn change_membership_by_req(
    raft_node: &Raft<TypeConfig>,
    req: &ChangeMembershipRequest,
) -> Result<ChangeMembershipReply, MetaServiceError> {
    let members = req.members.clone();
    let retain = req.retain;

    let res = raft_node.change_membership(members, retain).await?;
    let value = serialize(&res)?;

    Ok(ChangeMembershipReply { value })
}
