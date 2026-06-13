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
use crate::core::group_leader::group_leader_switch;
use crate::core::node_decommission::decommission_node_segments;
use crate::core::notify::{send_notify_by_add_node, send_notify_by_delete_node};
use crate::core::segment_leader::segment_leader_switch;
use crate::raft::manager::MultiRaftManager;
use crate::raft::route::data::{StorageData, StorageDataType};
use bytes::Bytes;
use metadata_struct::meta::node::BrokerNode;
use node_call::NodeCallManager;
use prost::Message as _;
use protocol::meta::meta_service_common::{
    RegisterNodeReply, RegisterNodeRequest, UnRegisterNodeReply, UnRegisterNodeRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tracing::error;

pub async fn register_node_by_req(
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    mqtt_call_manager: &Arc<NodeCallManager>,
    req: RegisterNodeRequest,
) -> Result<RegisterNodeReply, MetaServiceError> {
    let node = BrokerNode::decode(&req.node)?;
    meta_cache.report_broker_heart(node.node_id);
    let broker_epoch = sync_save_node(raft_manager, &node).await?;
    send_notify_by_add_node(mqtt_call_manager, node.clone()).await?;
    Ok(RegisterNodeReply { broker_epoch })
}

/// Explicit unregister (permanent decommission): delete the node, switch the
/// leaders it held, and migrate its replicas onto surviving nodes.
pub async fn un_register_node_by_req(
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    call_manager: &Arc<NodeCallManager>,
    req: UnRegisterNodeRequest,
) -> Result<UnRegisterNodeReply, MetaServiceError> {
    decommission_node(
        meta_cache,
        raft_manager,
        rocksdb_engine_handler,
        call_manager,
        req.node_id,
    )
    .await
}

/// Temporary offline (heartbeat timeout / restart): delete the node from the
/// membership and switch the leaders it held, but leave it in each segment's
/// replica set so it resumes as a follower and catches up when it re-registers.
pub async fn remove_node(
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    call_manager: &Arc<NodeCallManager>,
    node_id: u64,
) -> Result<UnRegisterNodeReply, MetaServiceError> {
    if let Some(node) = meta_cache.get_broker_node(node_id) {
        sync_delete_node(raft_manager, &UnRegisterNodeRequest { node_id }).await?;
        send_notify_by_delete_node(call_manager, node.clone()).await?;
        trigger_leader_switch(
            meta_cache.clone(),
            raft_manager.clone(),
            rocksdb_engine_handler.clone(),
            call_manager.clone(),
            node_id,
        )
        .await;
    }
    Ok(UnRegisterNodeReply::default())
}

/// Permanent decommission: delete the node, then (in the background) switch its
/// group/segment leaders and migrate the segments it replicated onto surviving
/// nodes. Replica migration is metadata-only — the new replica catches up from
/// the leader once one exists.
pub async fn decommission_node(
    meta_cache: &Arc<MetaCacheManager>,
    raft_manager: &Arc<MultiRaftManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    call_manager: &Arc<NodeCallManager>,
    node_id: u64,
) -> Result<UnRegisterNodeReply, MetaServiceError> {
    if let Some(node) = meta_cache.get_broker_node(node_id) {
        sync_delete_node(raft_manager, &UnRegisterNodeRequest { node_id }).await?;
        send_notify_by_delete_node(call_manager, node.clone()).await?;

        let meta_cache = meta_cache.clone();
        let raft_manager = raft_manager.clone();
        let rocksdb_engine_handler = rocksdb_engine_handler.clone();
        let call_manager = call_manager.clone();
        tokio::spawn(async move {
            if let Err(e) = group_leader_switch(
                &meta_cache,
                &raft_manager,
                &call_manager,
                &rocksdb_engine_handler,
                node_id,
            )
            .await
            {
                error!(
                    "group leader switch failed for decommissioned node {}: {}",
                    node_id, e
                );
            }
            if let Err(e) = decommission_node_segments(
                &meta_cache,
                &raft_manager,
                &call_manager,
                &rocksdb_engine_handler,
                node_id,
            )
            .await
            {
                error!("segment decommission failed for node {}: {}", node_id, e);
            }
        });
    }
    Ok(UnRegisterNodeReply::default())
}

pub async fn trigger_leader_switch(
    meta_cache: Arc<MetaCacheManager>,
    raft_manager: Arc<MultiRaftManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    mqtt_call_manager: Arc<NodeCallManager>,
    remove_id: u64,
) {
    tokio::spawn(async move {
        let result: Result<(), MetaServiceError> = async {
            group_leader_switch(
                &meta_cache,
                &raft_manager,
                &mqtt_call_manager,
                &rocksdb_engine_handler,
                remove_id,
            )
            .await?;
            segment_leader_switch(
                &meta_cache,
                &raft_manager,
                &mqtt_call_manager,
                &rocksdb_engine_handler,
                remove_id,
            )
            .await?;
            Ok(())
        }
        .await;
        if let Err(e) = result {
            error!("leader switch failed for removed node {}: {}", remove_id, e);
        }
    });
}

async fn sync_save_node(
    raft_manager: &Arc<MultiRaftManager>,
    node: &BrokerNode,
) -> Result<u64, MetaServiceError> {
    let request = RegisterNodeRequest {
        node: node.encode()?,
    };
    let data = StorageData::new(
        StorageDataType::ClusterAddNode,
        Bytes::copy_from_slice(&RegisterNodeRequest::encode_to_vec(&request)),
    );
    let response = raft_manager
        .write_metadata(data)
        .await?
        .ok_or(MetaServiceError::ExecutionResultIsEmpty)?;
    let epoch_bytes = response
        .data
        .value
        .ok_or(MetaServiceError::ExecutionResultIsEmpty)?;
    let bytes: [u8; 8] = epoch_bytes.as_ref().try_into().map_err(|_| {
        MetaServiceError::CommonError(format!(
            "register_node returned malformed broker_epoch ({} bytes, expected 8)",
            epoch_bytes.len()
        ))
    })?;
    Ok(u64::from_le_bytes(bytes))
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
