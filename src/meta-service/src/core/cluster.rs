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

use super::cache::CacheManager;
use super::error::MetaServiceError;
use crate::controller::journal::call_node::JournalInnerCallManager;
use crate::controller::mqtt::call_broker::{
    update_cache_by_add_node, update_cache_by_delete_node, MQTTInnerCallManager,
};
use crate::raft::route::apply::StorageDriver;
use crate::raft::route::data::{StorageData, StorageDataType};
use common_base::tools::now_mills;
use grpc_clients::pool::ClientPool;
use metadata_struct::meta::cluster::ClusterInfo;
use metadata_struct::meta::node::BrokerNode;
use prost::Message as _;
use protocol::meta::meta_service_inner::{
    RegisterNodeReply, RegisterNodeRequest, UnRegisterNodeReply, UnRegisterNodeRequest,
};
use std::sync::Arc;

pub async fn register_node_by_req(
    cluster_cache: &Arc<CacheManager>,
    raft_machine_apply: &Arc<StorageDriver>,
    client_pool: &Arc<ClientPool>,
    _journal_call_manager: &Arc<JournalInnerCallManager>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    req: RegisterNodeRequest,
) -> Result<RegisterNodeReply, MetaServiceError> {
    let node = serde_json::from_slice::<BrokerNode>(&req.node)?;
    cluster_cache.report_broker_heart(&node.cluster_name, node.node_id);
    sync_save_node(raft_machine_apply, &node).await?;

    if cluster_cache.get_cluster(&node.cluster_name).is_none() {
        let cluster = ClusterInfo {
            cluster_name: node.cluster_name.clone(),
            create_time: now_mills(),
        };
        sync_save_cluster(raft_machine_apply, &cluster).await?;
    }

    update_cache_by_add_node(
        &node.cluster_name,
        mqtt_call_manager,
        client_pool,
        node.clone(),
    )
    .await?;

    Ok(RegisterNodeReply::default())
}

pub async fn un_register_node_by_req(
    cluster_cache: &Arc<CacheManager>,
    raft_machine_apply: &Arc<StorageDriver>,
    client_pool: &Arc<ClientPool>,
    _journal_call_manager: &Arc<JournalInnerCallManager>,
    mqtt_call_manager: &Arc<MQTTInnerCallManager>,
    req: UnRegisterNodeRequest,
) -> Result<UnRegisterNodeReply, MetaServiceError> {
    if let Some(node) = cluster_cache.get_broker_node(&req.cluster_name, req.node_id) {
        sync_delete_node(raft_machine_apply, &req).await?;

        update_cache_by_delete_node(
            &req.cluster_name,
            mqtt_call_manager,
            client_pool,
            node.clone(),
        )
        .await?;
        mqtt_call_manager.remove_node(&req.cluster_name, req.node_id);
    }
    Ok(UnRegisterNodeReply::default())
}

async fn sync_save_node(
    raft_machine_apply: &Arc<StorageDriver>,
    node: &BrokerNode,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(StorageDataType::ClusterAddNode, serde_json::to_vec(&node)?);
    if raft_machine_apply.client_write(data).await?.is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

pub async fn sync_delete_node(
    raft_machine_apply: &Arc<StorageDriver>,
    req: &UnRegisterNodeRequest,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::ClusterDeleteNode,
        UnRegisterNodeRequest::encode_to_vec(req),
    );
    if raft_machine_apply.client_write(data).await?.is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

async fn sync_save_cluster(
    raft_machine_apply: &Arc<StorageDriver>,
    node: &ClusterInfo,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::ClusterAddCluster,
        serde_json::to_vec(&node)?,
    );
    if raft_machine_apply.client_write(data).await?.is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}
