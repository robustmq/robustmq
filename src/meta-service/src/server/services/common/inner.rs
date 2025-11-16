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

use crate::controller::mqtt::call_broker::{
    update_cache_by_set_resource_config, MQTTInnerCallManager,
};
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::placement::config::ResourceConfigStorage;
use crate::storage::placement::offset::OffsetStorage;
use common_base::tools::now_second;
use common_base::utils::serialize::encode_to_bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::resource_config::ClusterResourceConfig;
use protocol::meta::meta_service_common::{
    ClusterStatusReply, DeleteResourceConfigReply, DeleteResourceConfigRequest, GetOffsetDataReply,
    GetOffsetDataReplyOffset, GetOffsetDataRequest, GetResourceConfigReply,
    GetResourceConfigRequest, HeartbeatReply, HeartbeatRequest, NodeListReply, NodeListRequest,
    SaveOffsetDataReply, SaveOffsetDataRequest, SetResourceConfigReply, SetResourceConfigRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tracing::debug;

// Cluster Status
pub async fn cluster_status_by_req(
    raft_manager: &Arc<MultiRaftManager>,
) -> Result<ClusterStatusReply, MetaServiceError> {
    let status: openraft::RaftMetrics<crate::raft::type_config::TypeConfig> =
        raft_manager.metadata_raft_node.metrics().borrow().clone();

    let content = serde_json::to_string(&status).map_err(MetaServiceError::SerdeJsonError)?;

    Ok(ClusterStatusReply { content })
}

// Node Management
pub async fn node_list_by_req(
    cluster_cache: &Arc<CacheManager>,
    req: &NodeListRequest,
) -> Result<NodeListReply, MetaServiceError> {
    let nodes = cluster_cache
        .get_broker_node_by_cluster(&req.cluster_name)
        .into_iter()
        .map(|broker_node| broker_node.encode())
        .collect::<Result<Vec<_>, _>>()?;

    Ok(NodeListReply { nodes })
}

// Heartbeat
pub async fn heartbeat_by_req(
    cluster_cache: &Arc<CacheManager>,
    req: &HeartbeatRequest,
) -> Result<HeartbeatReply, MetaServiceError> {
    // Check if node exists
    if cluster_cache
        .get_broker_node(&req.cluster_name, req.node_id)
        .is_none()
    {
        return Err(MetaServiceError::NodeDoesNotExist(req.node_id));
    }

    debug!(
        "Received heartbeat from node {} at {}",
        req.node_id,
        now_second()
    );

    cluster_cache.report_broker_heart(&req.cluster_name, req.node_id);

    Ok(HeartbeatReply::default())
}

// Resource Config
pub async fn set_resource_config_by_req(
    raft_manager: &Arc<MultiRaftManager>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &SetResourceConfigRequest,
) -> Result<SetResourceConfigReply, MetaServiceError> {
    let data = StorageData::new(StorageDataType::ResourceConfigSet, encode_to_bytes(req));

    raft_manager.write_metadata(data).await?;

    let config = ClusterResourceConfig {
        cluster_name: req.cluster_name.clone(),
        resource: req.resources.join("/"),
        config: req.config.clone(),
    };

    update_cache_by_set_resource_config(&req.cluster_name, call_manager, client_pool, config)
        .await?;

    Ok(SetResourceConfigReply::default())
}

pub async fn get_resource_config_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: GetResourceConfigRequest,
) -> Result<GetResourceConfigReply, MetaServiceError> {
    let storage = ResourceConfigStorage::new(rocksdb_engine_handler.clone());

    let config = storage
        .get(req.cluster_name, req.resources)
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
    let data = StorageData::new(StorageDataType::OffsetSet, encode_to_bytes(req));

    raft_manager
        .write_offset(data)
        .await
        .map(|_| SaveOffsetDataReply::default())
}

pub async fn get_offset_data_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetOffsetDataRequest,
) -> Result<GetOffsetDataReply, MetaServiceError> {
    let offset_storage = OffsetStorage::new(rocksdb_engine_handler.clone());

    let offset_data = offset_storage
        .group_offset(&req.cluster_name, &req.group)
        .map_err(|e| MetaServiceError::CommonError(e.to_string()))?;

    let offsets = offset_data
        .into_iter()
        .map(|offset| GetOffsetDataReplyOffset {
            namespace: offset.namespace,
            shard_name: offset.shard_name,
            offset: offset.offset,
        })
        .collect();

    Ok(GetOffsetDataReply { offsets })
}
