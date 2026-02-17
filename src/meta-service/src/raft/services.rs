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
use std::time::Instant;

use crate::raft::manager::MultiRaftManager;
use crate::{core::error::MetaServiceError, raft::type_config::Node};
use bincode::{deserialize, serialize};
use protocol::meta::meta_service_common::{
    AddLearnerReply, AddLearnerRequest, AppendReply, AppendRequest, ChangeMembershipReply,
    ChangeMembershipRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest,
};
use tracing::warn;

const SLOW_RAFT_HANDLER_THRESHOLD_MS: f64 = 500.0;

fn deserialize_from_slice<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
) -> Result<T, MetaServiceError> {
    deserialize(bytes).map_err(|e| MetaServiceError::CommonError(e.to_string()))
}

pub async fn vote_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &VoteRequest,
) -> Result<VoteReply, MetaServiceError> {
    let start = Instant::now();
    let vote_data = deserialize_from_slice(&req.value)?;
    let raft_node = raft_manager.get_raft_node(&req.machine)?;
    let result = raft_node
        .vote(vote_data)
        .await
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))
        .and_then(|res| {
            serialize(&res)
                .map_err(|e| MetaServiceError::CommonError(e.to_string()))
                .map(|value| VoteReply { value })
        });
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    if duration_ms > SLOW_RAFT_HANDLER_THRESHOLD_MS {
        warn!(
            "Raft server handler is slow. machine={}, op=vote, duration_ms={:.2}",
            req.machine, duration_ms
        );
    }
    result
}

pub async fn append_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &AppendRequest,
) -> Result<AppendReply, MetaServiceError> {
    let start = Instant::now();
    let append_data = deserialize_from_slice(&req.value)?;
    let raft_node = raft_manager.get_raft_node(&req.machine)?;
    let result = raft_node
        .append_entries(append_data)
        .await
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))
        .and_then(|res| {
            serialize(&res)
                .map_err(|e| MetaServiceError::CommonError(e.to_string()))
                .map(|value| AppendReply { value })
        });
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    if duration_ms > SLOW_RAFT_HANDLER_THRESHOLD_MS {
        warn!(
            "Raft server handler is slow. machine={}, op=append_entries, duration_ms={:.2}",
            req.machine, duration_ms
        );
    }
    result
}

pub async fn snapshot_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &SnapshotRequest,
) -> Result<SnapshotReply, MetaServiceError> {
    let start = Instant::now();
    let snapshot_data = deserialize_from_slice(&req.value)?;
    let raft_node = raft_manager.get_raft_node(&req.machine)?;
    let result = raft_node
        .install_snapshot(snapshot_data)
        .await
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))
        .and_then(|res| {
            serialize(&res)
                .map_err(|e| MetaServiceError::CommonError(e.to_string()))
                .map(|value| SnapshotReply { value })
        });
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    if duration_ms > SLOW_RAFT_HANDLER_THRESHOLD_MS {
        warn!(
            "Raft server handler is slow. machine={}, op=install_snapshot, duration_ms={:.2}",
            req.machine, duration_ms
        );
    }
    result
}

pub async fn add_learner_by_req(
    raft_manager: &Arc<MultiRaftManager>,
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
    let raft_node = raft_manager.get_raft_node(&req.machine)?;
    let res = raft_node
        .add_learner(node_id, raft_node_data, blocking)
        .await?;
    let value = serialize(&res)?;

    Ok(AddLearnerReply { value })
}

pub async fn change_membership_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &ChangeMembershipRequest,
) -> Result<ChangeMembershipReply, MetaServiceError> {
    let members = req.members.clone();
    let retain = req.retain;
    let raft_node = raft_manager.get_raft_node(&req.machine)?;
    let res = raft_node.change_membership(members, retain).await?;
    let value = serialize(&res)?;

    Ok(ChangeMembershipReply { value })
}
