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

use crate::controller::call_broker::call::BrokerCallManager;
use crate::controller::call_broker::mqtt::update_cache_by_set_resource_config;
use crate::core::cache::CacheManager;
use crate::core::error::MetaServiceError;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use crate::storage::common::config::ResourceConfigStorage;
use crate::storage::common::offset::OffsetStorage;
use common_base::tools::now_second;
use common_base::utils::serialize::encode_to_bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::resource_config::ResourceConfig;
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
    cluster_cache: &Arc<CacheManager>,
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
    call_manager: &Arc<BrokerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &SetResourceConfigRequest,
) -> Result<SetResourceConfigReply, MetaServiceError> {
    let data = StorageData::new(StorageDataType::ResourceConfigSet, encode_to_bytes(req));

    raft_manager.write_metadata(data).await?;

    let config = ResourceConfig {
        resource: req.resources.join("/"),
        config: req.config.clone().into(),
    };

    update_cache_by_set_resource_config(call_manager, client_pool, config).await?;

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
