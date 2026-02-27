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

use super::cache::MetaCacheManager;
use super::error::MetaServiceError;
use crate::core::notify::{send_notify_by_add_node, send_notify_by_delete_node};
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use bytes::Bytes;
use metadata_struct::meta::node::BrokerNode;
use node_call::NodeCallManager;
use prost::Message as _;
use protocol::meta::meta_service_common::{
    RegisterNodeReply, RegisterNodeRequest, UnRegisterNodeReply, UnRegisterNodeRequest,
};
use std::sync::Arc;

pub async fn register_node_by_req(
    cluster_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    _client_pool: &Arc<grpc_clients::pool::ClientPool>,
    mqtt_call_manager: &Arc<NodeCallManager>,
    req: RegisterNodeRequest,
) -> Result<RegisterNodeReply, MetaServiceError> {
    let node = BrokerNode::decode(&req.node)?;
    cluster_cache.report_broker_heart(node.node_id);
    sync_save_node(raft_manager, &node).await?;

    send_notify_by_add_node(mqtt_call_manager, node.clone()).await?;

    Ok(RegisterNodeReply::default())
}

pub async fn un_register_node_by_req(
    cluster_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    _client_pool: &Arc<grpc_clients::pool::ClientPool>,
    mqtt_call_manager: &Arc<NodeCallManager>,
    req: UnRegisterNodeRequest,
) -> Result<UnRegisterNodeReply, MetaServiceError> {
    if let Some(node) = cluster_cache.get_broker_node(req.node_id) {
        sync_delete_node(raft_manager, &req).await?;
        send_notify_by_delete_node(mqtt_call_manager, node.clone()).await?;
    }
    Ok(UnRegisterNodeReply::default())
}

async fn sync_save_node(
    raft_manager: &Arc<MultiRaftManager>,
    node: &BrokerNode,
) -> Result<(), MetaServiceError> {
    let request = RegisterNodeRequest {
        node: node.encode()?,
    };
    let data = StorageData::new(
        StorageDataType::ClusterAddNode,
        Bytes::copy_from_slice(&RegisterNodeRequest::encode_to_vec(&request)),
    );
    if raft_manager.write_metadata(data).await?.is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}

async fn sync_delete_node(
    raft_manager: &Arc<MultiRaftManager>,
    req: &UnRegisterNodeRequest,
) -> Result<(), MetaServiceError> {
    let data = StorageData::new(
        StorageDataType::ClusterDeleteNode,
        Bytes::copy_from_slice(&UnRegisterNodeRequest::encode_to_vec(req)),
    );
    if raft_manager.write_metadata(data).await?.is_some() {
        return Ok(());
    }
    Err(MetaServiceError::ExecutionResultIsEmpty)
}
