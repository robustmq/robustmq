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
    CreateUserReply, CreateUserRequest, DeleteUserReply, DeleteUserRequest, ListUserReply,
    ListUserRequest,
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

pub fn list_user_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &ListUserRequest,
) -> Result<ListUserReply, PlacementCenterError> {
    let storage = MqttUserStorage::new(rocksdb_engine_handler.clone());
    let mut users = Vec::new();

    if !req.cluster_name.is_empty() && !req.user_name.is_empty() {
        if let Some(data) = storage.get(&req.cluster_name, &req.user_name)? {
            users.push(data.encode());
        }
    }

    if !req.cluster_name.is_empty() && req.user_name.is_empty() {
        let user_list = storage.list_by_cluster(&req.cluster_name)?;
        users = user_list.into_iter().map(|user| user.encode()).collect();
    }

    Ok(ListUserReply { users })
}

pub async fn create_user_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &CreateUserRequest,
) -> Result<CreateUserReply, PlacementCenterError> {
    let storage = MqttUserStorage::new(rocksdb_engine_handler.clone());
    if storage.get(&req.cluster_name, &req.user_name)?.is_some() {
        return Err(PlacementCenterError::UserAlreadyExist(
            req.user_name.clone(),
        ));
    }

    let data = StorageData::new(
        StorageDataType::MqttSetUser,
        CreateUserRequest::encode_to_vec(req),
    );

    raft_machine_apply.client_write(data).await?;
    let user = serde_json::from_slice::<MqttUser>(&req.content)?;
    update_cache_by_add_user(&req.cluster_name, call_manager, client_pool, user).await?;

    Ok(CreateUserReply {})
}

pub async fn delete_user_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &DeleteUserRequest,
) -> Result<DeleteUserReply, PlacementCenterError> {
    let storage = MqttUserStorage::new(rocksdb_engine_handler.clone());
    let user = if let Some(user) = storage.get(&req.cluster_name, &req.user_name)? {
        user
    } else {
        return Err(PlacementCenterError::UserDoesNotExist(
            req.user_name.clone(),
        ));
    };

    let data = StorageData::new(
        StorageDataType::MqttDeleteUser,
        DeleteUserRequest::encode_to_vec(req),
    );
    raft_machine_apply.client_write(data).await?;
    update_cache_by_delete_user(&req.cluster_name, call_manager, client_pool, user).await?;

    Ok(DeleteUserReply {})
}
