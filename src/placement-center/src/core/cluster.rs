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

use common_base::config::placement_center::placement_center_conf;
use common_base::tools::now_mills;
use dashmap::DashMap;
use grpc_clients::pool::ClientPool;
use metadata_struct::placement::cluster::ClusterInfo;
use metadata_struct::placement::node::BrokerNode;
use prost::Message as _;
use protocol::placement_center::placement_center_inner::{
    ClusterType, RegisterNodeRequest, UnRegisterNodeRequest,
};
use raft::StateRole;
use serde::{Deserialize, Serialize};

use super::cache::PlacementCacheManager;
use super::error::PlacementCenterError;
use super::raft_node::RaftNode;
use crate::journal::controller::call_node::{
    update_cache_by_add_journal_node, update_cache_by_delete_journal_node, JournalInnerCallManager,
};
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ClusterMetadata {
    pub local: RaftNode,
    pub leader: Option<RaftNode>,
    pub raft_role: String,
    pub votes: DashMap<u64, RaftNode>,
    pub members: DashMap<u64, RaftNode>,
}

impl ClusterMetadata {
    pub fn new() -> ClusterMetadata {
        let config = placement_center_conf();
        if config.node.node_id == 0 {
            panic!("node ids can range from 0 to 65536");
        }

        let node_addr = if let Some(addr) =
            config.node.nodes.get(&format!("{}", config.node.node_id))
        {
            addr.to_string()
        } else {
            panic!("node id {} There is no corresponding service address, check the nodes configuration",config.node.node_id);
        };

        let local = RaftNode {
            node_id: config.node.node_id,
            node_addr,
        };

        let votes = DashMap::with_capacity(2);
        for (node_id, addr) in config.node.nodes.clone() {
            let id: u64 = match node_id.to_string().trim().parse() {
                Ok(id) => id,
                Err(_) => {
                    panic!("Node id must be u64");
                }
            };

            if addr.to_string().is_empty() {
                panic!(
                    "Address corresponding to the node id {} cannot be empty",
                    id
                );
            }

            let node = RaftNode {
                node_id: id,
                node_addr: addr.to_string(),
            };

            votes.insert(id, node);
        }

        ClusterMetadata {
            local,
            leader: None,
            raft_role: role_to_string(StateRole::Candidate),
            votes,
            members: DashMap::with_capacity(2),
        }
    }
}

fn role_to_string(role: StateRole) -> String {
    match role {
        StateRole::Leader => "Leader".to_string(),
        StateRole::Follower => "Follower".to_string(),
        StateRole::Candidate => "Candidate".to_string(),
        StateRole::PreCandidate => "PreCandidate".to_string(),
    }
}

pub async fn register_node_by_req(
    cluster_cache: &Arc<PlacementCacheManager>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    client_pool: &Arc<ClientPool>,
    call_manager: &Arc<JournalInnerCallManager>,
    req: RegisterNodeRequest,
) -> Result<(), PlacementCenterError> {
    let cluster_type = req.cluster_type();
    let cluster_name = req.cluster_name;

    let node = BrokerNode {
        node_id: req.node_id,
        node_ip: req.node_ip,
        node_inner_addr: req.node_inner_addr,
        extend: req.extend_info,
        cluster_name: cluster_name.clone(),
        cluster_type: cluster_type.as_str_name().to_string(),
        create_time: now_mills(),
    };

    sync_save_node(raft_machine_apply, &node).await?;

    if !cluster_cache.cluster_list.contains_key(&cluster_name) {
        let cluster = ClusterInfo {
            cluster_name: cluster_name.clone(),
            cluster_type: cluster_type.as_str_name().to_string(),
            create_time: now_mills(),
        };
        sync_save_cluster(raft_machine_apply, &cluster).await?;
    }

    if cluster_type == ClusterType::JournalServer {
        update_cache_by_add_journal_node(&cluster_name, call_manager, client_pool, node.clone())
            .await?;
    }

    cluster_cache.report_broker_heart(&cluster_name, req.node_id);
    Ok(())
}

pub async fn un_register_node_by_req(
    cluster_cache: &Arc<PlacementCacheManager>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    client_pool: &Arc<ClientPool>,
    call_manager: &Arc<JournalInnerCallManager>,
    req: UnRegisterNodeRequest,
) -> Result<(), PlacementCenterError> {
    if let Some(node) = cluster_cache.get_broker_node(&req.cluster_name, req.node_id) {
        sync_delete_node(raft_machine_apply, &req).await?;
        if req.cluster_type() == ClusterType::JournalServer {
            update_cache_by_delete_journal_node(&req.cluster_name, call_manager, client_pool, node)
                .await?;
        }
    }
    Ok(())
}

async fn sync_save_node(
    raft_machine_apply: &Arc<RaftMachineApply>,
    node: &BrokerNode,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(StorageDataType::ClusterAddNode, serde_json::to_vec(&node)?);
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(PlacementCenterError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_node(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &UnRegisterNodeRequest,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::ClusterDeleteNode,
        UnRegisterNodeRequest::encode_to_vec(req),
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(PlacementCenterError::ExecutionResultIsEmpty)
}

async fn sync_save_cluster(
    raft_machine_apply: &Arc<RaftMachineApply>,
    node: &ClusterInfo,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::ClusterAddCluster,
        serde_json::to_vec(&node)?,
    );
    if (raft_machine_apply.client_write(data).await?).is_some() {
        return Ok(());
    }
    Err(PlacementCenterError::ExecutionResultIsEmpty)
}
