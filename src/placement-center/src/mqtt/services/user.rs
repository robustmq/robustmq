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

use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::user::MqttUser;
use protocol::placement_center::placement_center_mqtt::{
    CreateUserRequest, DeleteUserRequest, ListUserRequest,
};
use rocksdb_engine::RocksDBEngine;

use crate::{
    core::error::PlacementCenterError,
    mqtt::controller::call_broker::{
        update_cache_by_add_user, update_cache_by_delete_user, MQTTInnerCallManager,
    },
    route::{
        apply::RaftMachineApply,
        data::{StorageData, StorageDataType},
    },
    storage::mqtt::user::MqttUserStorage,
};
use prost::Message;

pub async fn list_user_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: ListUserRequest,
) -> Result<Vec<Vec<u8>>, PlacementCenterError> {
    let storage = MqttUserStorage::new(rocksdb_engine_handler.clone());

    if !req.user_name.is_empty() {
        if let Some(data) = storage.get(&req.cluster_name, &req.user_name)? {
            return Ok(vec![data.encode()]);
        }
    } else {
        let data = storage.list(&req.cluster_name)?;
        let mut result = Vec::new();
        for raw in data {
            result.push(raw.encode());
        }
        return Ok(result);
    }
    Ok(Vec::new())
}

pub async fn save_user_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    req: CreateUserRequest,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttSetUser,
        CreateUserRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;

    let user = serde_json::from_slice::<MqttUser>(&req.content)?;
    update_cache_by_add_user(&req.cluster_name, call_manager, client_pool, user).await?;
    Ok(())
}

pub async fn delete_user_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: DeleteUserRequest,
) -> Result<(), PlacementCenterError> {
    let data = StorageData::new(
        StorageDataType::MqttDeleteUser,
        DeleteUserRequest::encode_to_vec(&req),
    );
    raft_machine_apply.client_write(data).await?;
    let storage = MqttUserStorage::new(rocksdb_engine_handler.clone());
    if let Some(user) = storage.get(&req.cluster_name, &req.user_name)? {
        update_cache_by_delete_user(&req.cluster_name, call_manager, client_pool, user).await?;
    }
    Ok(())
}
