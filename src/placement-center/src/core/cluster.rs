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

use common_base::tools::now_mills;
use grpc_clients::pool::ClientPool;
use metadata_struct::placement::cluster::ClusterInfo;
use metadata_struct::placement::node::BrokerNode;
use prost::Message as _;
use protocol::placement_center::placement_center_inner::{
    ClusterType, RegisterNodeRequest, UnRegisterNodeRequest,
};

use super::cache::PlacementCacheManager;
use super::error::PlacementCenterError;
use crate::journal::controller::call_node::{
    update_cache_by_add_journal_node, update_cache_by_delete_journal_node, JournalInnerCallManager,
};
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};

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
    if raft_machine_apply.client_write(data).await?.is_some() {
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
    if raft_machine_apply.client_write(data).await?.is_some() {
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
    if raft_machine_apply.client_write(data).await?.is_some() {
        return Ok(());
    }
    Err(PlacementCenterError::ExecutionResultIsEmpty)
}
