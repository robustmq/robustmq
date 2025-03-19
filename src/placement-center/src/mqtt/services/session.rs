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

use prost::Message;
use rocksdb_engine::RocksDBEngine;
use std::sync::Arc;

use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::session::MqttSession;
use protocol::placement_center::placement_center_mqtt::{
    CreateSessionRequest, DeleteSessionRequest, ListSessionRequest, UpdateSessionRequest,
};

use crate::{
    core::error::PlacementCenterError,
    mqtt::controller::call_broker::{
        update_cache_by_add_session, update_cache_by_delete_session, MQTTInnerCallManager,
    },
    route::{
        apply::RaftMachineApply,
        data::{StorageData, StorageDataType},
    },
    storage::mqtt::session::MqttSessionStorage,
};

pub async fn list_session_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: ListSessionRequest,
) -> Result<Vec<String>, PlacementCenterError> {
    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());

    if !req.client_id.is_empty() {
        if let Some(data) = storage.get(&req.cluster_name, &req.client_id)? {
            return Ok(vec![data.encode()]);
        }
    } else {
        let data = storage.list(&req.cluster_name)?;
        let mut result = Vec::new();
        for raw in data {
            result.push(raw.data);
        }
        return Ok(result);
    }
    Ok(Vec::new())
}

pub async fn save_session_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: CreateSessionRequest,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttSetSession,
        CreateSessionRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    let session = serde_json::from_str::<MqttSession>(&req.session)?;
    update_cache_by_add_session(&req.cluster_name, call_manager, client_pool, session).await?;
    Ok(())
}

pub async fn update_session_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: UpdateSessionRequest,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttUpdateSession,
        UpdateSessionRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    if let Some(session) = storage.get(&req.cluster_name, &req.client_id)? {
        update_cache_by_delete_session(&req.cluster_name, call_manager, client_pool, session)
            .await?;
    }
    Ok(())
}

pub async fn delete_session_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: DeleteSessionRequest,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttDeleteSession,
        DeleteSessionRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    let storage = MqttSessionStorage::new(rocksdb_engine_handler.clone());
    if let Some(session) = storage.get(&req.cluster_name, &req.client_id)? {
        update_cache_by_delete_session(&req.cluster_name, call_manager, client_pool, session)
            .await?;
    }

    Ok(())
}
