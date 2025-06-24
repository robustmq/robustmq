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
    ClusterType, RegisterNodeReply, RegisterNodeRequest, UnRegisterNodeReply, UnRegisterNodeRequest,
};

use super::cache::PlacementCacheManager;
use super::error::PlacementCenterError;
use crate::journal::controller::call_node::{
    update_cache_by_add_journal_node, update_cache_by_delete_journal_node, JournalInnerCallManager,
};
use crate::mqtt::controller::call_broker::{
    update_cache_by_add_node, update_cache_by_delete_node, MQTTInnerCallManager,
};
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};

pub async fn register_node_by_req(
    cluster_cache: &Arc<PlacementCacheManager>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    client_pool: &Arc<ClientPool>,
    journal_call_manager: &Arc<JournalInnerCallManager>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    req: RegisterNodeRequest,
) -> Result<RegisterNodeReply, PlacementCenterError> {
    let node = serde_json::from_slice::<BrokerNode>(&req.node)?;

    sync_save_node(raft_machine_apply, &node).await?;

    if cluster_cache.get_cluster(&node.cluster_name).is_none() {
        let cluster = ClusterInfo {
            cluster_name: node.cluster_name.clone(),
            cluster_type: node.cluster_type.clone(),
            create_time: now_mills(),
        };
        sync_save_cluster(raft_machine_apply, &cluster).await?;
    }

    if node.cluster_type == *ClusterType::JournalServer.as_str_name() {
        update_cache_by_add_journal_node(
            &node.cluster_name,
            journal_call_manager,
            client_pool,
            node.clone(),
        )
        .await?;
    }
    if node.cluster_type == *ClusterType::MqttBrokerServer.as_str_name() {
        update_cache_by_add_node(
            &node.cluster_name,
            mqtt_call_manager,
            client_pool,
            node.clone(),
        )
        .await?;
    }

    cluster_cache.report_broker_heart(&node.cluster_name, node.node_id);
    Ok(RegisterNodeReply::default())
}

pub async fn un_register_node_by_req(
    cluster_cache: &Arc<PlacementCacheManager>,
    raft_machine_apply: &Arc<RaftMachineApply>,
    client_pool: &Arc<ClientPool>,
    journal_call_manager: &Arc<JournalInnerCallManager>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    req: UnRegisterNodeRequest,
) -> Result<UnRegisterNodeReply, PlacementCenterError> {
    if let Some(node) = cluster_cache.get_broker_node(&req.cluster_name, req.node_id) {
        sync_delete_node(raft_machine_apply, &req).await?;
        if req.cluster_type() == ClusterType::JournalServer {
            update_cache_by_delete_journal_node(
                &req.cluster_name,
                journal_call_manager,
                client_pool,
                node.clone(),
            )
            .await?;
            journal_call_manager.remove_node(&req.cluster_name, req.node_id);
        }
        if req.cluster_type() == ClusterType::MqttBrokerServer {
            update_cache_by_delete_node(
                &req.cluster_name,
                mqtt_call_manager,
                client_pool,
                node.clone(),
            )
            .await?;
            mqtt_call_manager.remove_node(&req.cluster_name, req.node_id);
        }
    }
    Ok(UnRegisterNodeReply::default())
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
