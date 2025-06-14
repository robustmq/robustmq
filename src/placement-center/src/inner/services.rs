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

use crate::core::cache::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::mqtt::controller::call_broker::{
    update_cache_by_set_resource_config, MQTTInnerCallManager,
};
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};
use crate::storage::placement::config::ResourceConfigStorage;
use crate::storage::placement::idempotent::IdempotentStorage;
use crate::storage::placement::offset::OffsetStorage;
use common_base::tools::now_second;
use grpc_clients::pool::ClientPool;
use metadata_struct::resource_config::ClusterResourceConfig;
use prost::Message;
use protocol::placement_center::placement_center_inner::{
    ClusterStatusReply, DeleteIdempotentDataReply, DeleteIdempotentDataRequest,
    DeleteResourceConfigReply, DeleteResourceConfigRequest, ExistsIdempotentDataReply,
    ExistsIdempotentDataRequest, GetOffsetDataReply, GetOffsetDataReplyOffset,
    GetOffsetDataRequest, GetResourceConfigReply, GetResourceConfigRequest, HeartbeatReply,
    HeartbeatRequest, NodeListReply, NodeListRequest, SaveOffsetDataReply, SaveOffsetDataRequest,
    SetIdempotentDataReply, SetIdempotentDataRequest, SetResourceConfigReply,
    SetResourceConfigRequest,
};
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;
use tracing::debug;

pub async fn cluster_status_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
) -> Result<ClusterStatusReply, PlacementCenterError> {
    let mut reply = ClusterStatusReply::default();
    let status = raft_machine_apply.openraft_node.metrics().borrow().clone();

    reply.content = match serde_json::to_string(&status) {
        Ok(data) => data,
        Err(e) => {
            return Err(PlacementCenterError::SerdeJsonError(e));
        }
    };

    Ok(reply)
}

pub async fn node_list_by_req(
    cluster_cache: &Arc<PlacementCacheManager>,
    req: &NodeListRequest,
) -> Result<NodeListReply, PlacementCenterError> {
    let mut nodes = Vec::new();

    for broker_node in cluster_cache.get_broker_node_by_cluster(&req.cluster_name) {
        nodes.push(broker_node.encode());
    }

    Ok(NodeListReply { nodes })
}

pub async fn heartbeat_by_req(
    cluster_cache: &Arc<PlacementCacheManager>,
    req: &HeartbeatRequest,
) -> Result<HeartbeatReply, PlacementCenterError> {
    match cluster_cache.get_broker_node(&req.cluster_name, req.node_id) {
        Some(_) => {
            debug!(
                "receive heartbeat from node:{:?},time:{}",
                req.node_id,
                now_second()
            );

            cluster_cache.report_broker_heart(&req.cluster_name, req.node_id);

            Ok(HeartbeatReply::default())
        }
        None => Err(PlacementCenterError::NodeDoesNotExist(req.node_id)),
    }
}

pub async fn set_resource_config_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: &SetResourceConfigRequest,
) -> Result<SetResourceConfigReply, PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::ResourceConfigSet,
        SetResourceConfigRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    let config = ClusterResourceConfig {
        cluster_name: req.cluster_name.to_owned(),
        resouce: req.resources.to_owned().join("/"),
        config: req.config.clone(),
    };

    update_cache_by_set_resource_config(&req.cluster_name, call_manager, client_pool, config)
        .await?;
    Ok(SetResourceConfigReply::default())
}

pub async fn get_resource_config_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: GetResourceConfigRequest,
) -> Result<GetResourceConfigReply, PlacementCenterError> {
    let storage = ResourceConfigStorage::new(rocksdb_engine_handler.clone());

    match storage.get(req.cluster_name, req.resources) {
        Ok(data) => match data {
            Some(res) => Ok(GetResourceConfigReply { config: res }),
            None => Ok(GetResourceConfigReply { config: Vec::new() }),
        },
        Err(e) => Err(PlacementCenterError::CommonError(e.to_string())),
    }
}

pub async fn delete_resource_config_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &DeleteResourceConfigRequest,
) -> Result<DeleteResourceConfigReply, PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::ResourceConfigDelete,
        DeleteResourceConfigRequest::encode_to_vec(req),
    );

    raft_machine_apply
        .client_write(data)
        .await
        .map(|_| DeleteResourceConfigReply::default())
}

pub async fn set_idempotent_data_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &SetIdempotentDataRequest,
) -> Result<SetIdempotentDataReply, PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::IdempotentDataSet,
        SetIdempotentDataRequest::encode_to_vec(req),
    );

    raft_machine_apply
        .client_write(data)
        .await
        .map(|_| SetIdempotentDataReply::default())
}

pub async fn exists_idempotent_data_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ExistsIdempotentDataRequest,
) -> Result<ExistsIdempotentDataReply, PlacementCenterError> {
    let storage = IdempotentStorage::new(rocksdb_engine_handler.clone());

    storage
        .exists(&req.cluster_name, &req.producer_id, req.seq_num)
        .map_err(|e| PlacementCenterError::CommonError(e.to_string()))
        .map(|flag| ExistsIdempotentDataReply { exists: flag })
}

pub async fn delete_idempotent_data_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &DeleteIdempotentDataRequest,
) -> Result<DeleteIdempotentDataReply, PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::IdempotentDataDelete,
        DeleteIdempotentDataRequest::encode_to_vec(req),
    );

    raft_machine_apply
        .client_write(data)
        .await
        .map(|_| DeleteIdempotentDataReply::default())
}

pub async fn save_offset_data_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    req: &SaveOffsetDataRequest,
) -> Result<SaveOffsetDataReply, PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::OffsetSet,
        SaveOffsetDataRequest::encode_to_vec(req),
    );

    raft_machine_apply
        .client_write(data)
        .await
        .map(|_| SaveOffsetDataReply::default())
}

pub async fn get_offset_data_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &GetOffsetDataRequest,
) -> Result<GetOffsetDataReply, PlacementCenterError> {
    let offset_storage = OffsetStorage::new(rocksdb_engine_handler.clone());
    offset_storage
        .group_offset(&req.cluster_name, &req.group)
        .map_err(|e| PlacementCenterError::CommonError(e.to_string()))
        .map(|offset_data| GetOffsetDataReply {
            offsets: offset_data
                .into_iter()
                .map(|offset| GetOffsetDataReplyOffset {
                    namespace: offset.namespace,
                    shard_name: offset.shard_name,
                    offset: offset.offset,
                })
                .collect(),
        })
}
