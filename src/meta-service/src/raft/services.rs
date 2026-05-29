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
    AppendReply, AppendRequest, JoinClusterReply, JoinClusterRequest, LeaveClusterReply,
    LeaveClusterRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest,
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

/// Handle a join request from a new node.
///
/// For every Raft state machine, the joining node is first added as a learner
/// (which triggers log replication), then promoted to a full voter via
/// change_membership.  Both steps are blocking so the caller knows the cluster
/// has accepted the new member before this returns.
pub async fn join_cluster_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &JoinClusterRequest,
) -> Result<JoinClusterReply, MetaServiceError> {
    let node_id = req.node_id;
    let raft_node_data = Node {
        rpc_addr: req.rpc_addr.clone(),
        node_id,
    };

    let shards: Vec<(String, _)> = raft_manager
        .all_shards()
        .map(|(name, raft)| (name.clone(), raft))
        .collect();

    for (machine, raft_node) in &shards {
        // Step 1: add as learner (blocking = true waits until log is caught up)
        raft_node
            .add_learner(node_id, raft_node_data.clone(), true)
            .await
            .map_err(|e| {
                MetaServiceError::CommonError(format!(
                    "[{}] add_learner failed for node {}: {}",
                    machine, node_id, e
                ))
            })?;

        // Step 2: promote to voter
        let current_members: Vec<u64> = raft_node
            .metrics()
            .borrow()
            .membership_config
            .membership()
            .voter_ids()
            .collect();

        let mut new_members = current_members;
        if !new_members.contains(&node_id) {
            new_members.push(node_id);
        }

        raft_node
            .change_membership(new_members, true)
            .await
            .map_err(|e| {
                MetaServiceError::CommonError(format!(
                    "[{}] change_membership failed for node {}: {}",
                    machine, node_id, e
                ))
            })?;
    }

    tracing::info!(
        "Node {} ({}) successfully joined the cluster",
        node_id,
        req.rpc_addr
    );

    Ok(JoinClusterReply {})
}

/// Handle a leave request from a node that is shutting down.
/// Must be called on the Leader — removes the node from membership of all shards.
pub async fn leave_cluster_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &LeaveClusterRequest,
) -> Result<LeaveClusterReply, MetaServiceError> {
    let node_id = req.node_id;

    for (machine, raft_node) in raft_manager.all_shards() {
        let current_members: Vec<u64> = raft_node
            .metrics()
            .borrow()
            .membership_config
            .membership()
            .voter_ids()
            .collect();

        let new_members: Vec<u64> = current_members
            .into_iter()
            .filter(|&id| id != node_id)
            .collect();

        if new_members.is_empty() {
            tracing::info!(
                "[{}] Node {} is the last member, skipping leave",
                machine,
                node_id
            );
            continue;
        }

        raft_node
            .change_membership(new_members, false)
            .await
            .map_err(|e| {
                MetaServiceError::CommonError(format!(
                    "[{}] change_membership failed while removing node {}: {}",
                    machine, node_id, e
                ))
            })?;

        tracing::info!("[{}] Node {} removed from membership", machine, node_id);
    }

    tracing::info!("Node {} successfully left the cluster", node_id);
    Ok(LeaveClusterReply {})
}
