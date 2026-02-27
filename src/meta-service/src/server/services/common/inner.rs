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

use crate::core::cache::MetaCacheManager;
use crate::core::error::MetaServiceError;
use crate::core::notify::send_notify_by_set_resource_config;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::common::config::ResourceConfigStorage;
use crate::storage::common::offset::OffsetStorage;
use common_base::tools::now_second;
use common_base::utils::serialize::encode_to_bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::resource_config::ResourceConfig;
use node_call::NodeCallManager;
use protocol::meta::meta_service_common::{
    ClusterStatusReply, DeleteResourceConfigReply, DeleteResourceConfigRequest, GetOffsetDataReply,
    GetOffsetDataReplyOffset, GetOffsetDataRequest, GetResourceConfigReply,
    GetResourceConfigRequest, HeartbeatReply, HeartbeatRequest, NodeListReply, NodeListRequest,
    SaveOffsetData, SaveOffsetDataReply, SaveOffsetDataRequest, SetResourceConfigReply,
    SetResourceConfigRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tracing::debug;

// Cluster Status
pub async fn cluster_status_by_req(
    raft_manager: &Arc<MultiRaftManager>,
) -> Result<ClusterStatusReply, MetaServiceError> {
    let mut results = HashMap::new();
    for (name, node) in raft_manager.metadata.all_nodes() {
        results.insert(name.clone(), node.metrics().borrow().clone());
    }
    for (name, node) in raft_manager.offset.all_nodes() {
        results.insert(name.clone(), node.metrics().borrow().clone());
    }
    for (name, node) in raft_manager.data.all_nodes() {
        results.insert(name.clone(), node.metrics().borrow().clone());
    }
    let content = serde_json::to_string(&results).map_err(MetaServiceError::SerdeJsonError)?;
    Ok(ClusterStatusReply { content })
}

// Node Management
pub async fn node_list_by_req(
    cluster_cache: &Arc<MetaCacheManager>,
    _req: &NodeListRequest,
) -> Result<NodeListReply, MetaServiceError> {
    let nodes = cluster_cache
        .node_list
        .iter()
        .map(|broker_node| broker_node.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(NodeListReply { nodes })
}

// Heartbeat
pub async fn heartbeat_by_req(
    cluster_cache: &Arc<MetaCacheManager>,
    req: &HeartbeatRequest,
) -> Result<HeartbeatReply, MetaServiceError> {
    // Check if node exists
    if cluster_cache.get_broker_node(req.node_id).is_none() {
        return Err(MetaServiceError::NodeDoesNotExist(req.node_id));
    }

    debug!(
        "Received heartbeat from node {} at {}",
        req.node_id,
        now_second()
    );

    cluster_cache.report_broker_heart(req.node_id);

    Ok(HeartbeatReply::default())
}

// Resource Config
pub async fn set_resource_config_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<NodeCallManager>,
    _client_pool: &Arc<ClientPool>,
    req: &SetResourceConfigRequest,
) -> Result<SetResourceConfigReply, MetaServiceError> {
    let data = StorageData::new(StorageDataType::ResourceConfigSet, encode_to_bytes(req));

    raft_manager.write_metadata(data).await?;

    let config = ResourceConfig {
        resource: req.resources.join("/"),
        config: req.config.clone().into(),
    };

    send_notify_by_set_resource_config(call_manager, config).await?;

    Ok(SetResourceConfigReply::default())
}

pub async fn get_resource_config_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: GetResourceConfigRequest,
) -> Result<GetResourceConfigReply, MetaServiceError> {
    let storage = ResourceConfigStorage::new(rocksdb_engine_handler.clone());

    let config = storage
        .get(req.resources)
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))?
        .unwrap_or_default();

    Ok(GetResourceConfigReply { config })
}

pub async fn delete_resource_config_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &DeleteResourceConfigRequest,
) -> Result<DeleteResourceConfigReply, MetaServiceError> {
    let data = StorageData::new(StorageDataType::ResourceConfigDelete, encode_to_bytes(req));

    raft_manager
        .write_metadata(data)
        .await
        .map(|_| DeleteResourceConfigReply::default())
}

// Offset
pub async fn save_offset_data_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    req: &SaveOffsetDataRequest,
) -> Result<SaveOffsetDataReply, MetaServiceError> {
    let mut grouped_offsets: BTreeMap<String, Vec<SaveOffsetData>> = BTreeMap::new();
    for offset in &req.offsets {
        grouped_offsets
            .entry(offset.group.clone())
            .or_default()
            .push(offset.clone());
    }

    for (group, offsets) in grouped_offsets {
        let sub_req = SaveOffsetDataRequest { offsets };
        let data = StorageData::new(StorageDataType::OffsetSet, encode_to_bytes(&sub_req));
        raft_manager.write_offset(&group, data).await.map_err(|e| {
            MetaServiceError::CommonError(format!(
                "Failed to write offset group '{}': {}",
                group, e
            ))
        })?;
    }

    Ok(SaveOffsetDataReply::default())
}

pub async fn get_offset_data_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetOffsetDataRequest,
) -> Result<GetOffsetDataReply, MetaServiceError> {
    let offset_storage = OffsetStorage::new(rocksdb_engine_handler.clone());

    let offset_data = offset_storage
        .group_offset(&req.group)
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))?;

    let offsets = offset_data
        .into_iter()
        .map(|offset| GetOffsetDataReplyOffset {
            shard_name: offset.shard_name,
            offset: offset.offset,
        })
        .collect();

    Ok(GetOffsetDataReply { offsets })
}
