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
use tonic::{Request, Response, Status};

pub fn list_user_by_req(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<ListUserRequest>,
) -> Result<Response<ListUserReply>, Status> {
    let req = request.into_inner();

    let storage = MqttUserStorage::new(rocksdb_engine_handler.clone());

    if !req.cluster_name.is_empty() && !req.user_name.is_empty() {
        if let Some(data) = storage.get(&req.cluster_name, &req.user_name)? {
            let users = vec![data.encode()];
            return Ok(Response::new(ListUserReply { users }));
        }
    }

    if !req.cluster_name.is_empty() && req.user_name.is_empty() {
        let data = storage.list(&req.cluster_name)?;
        let mut users = Vec::new();
        for raw in data {
            users.push(raw.encode());
        }
        return Ok(Response::new(ListUserReply { users }));
    }
    Ok(Response::new(ListUserReply { users: Vec::new() }))
}

pub async fn create_user_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<CreateUserRequest>,
) -> Result<Response<CreateUserReply>, Status> {
    let req = request.into_inner();

    let data = StorageData::new(
        StorageDataType::MqttSetUser,
        CreateUserRequest::encode_to_vec(&req),
    );
    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    };

    let user = match serde_json::from_slice::<MqttUser>(&req.content) {
        Ok(user) => user,
        Err(e) => {
            return Err(Status::cancelled(e.to_string()));
        }
    };
    if let Err(e) =
        update_cache_by_add_user(&req.cluster_name, call_manager, client_pool, user).await
    {
        return Err(Status::cancelled(e.to_string()));
    };
    Ok(Response::new(CreateUserReply::default()))
}

pub async fn delete_user_by_req(
    raft_machine_apply: &Arc<RaftMachineApply>,
    call_manager: &Arc<MQTTInnerCallManager>,
    client_pool: &Arc<ClientPool>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: Request<DeleteUserRequest>,
) -> Result<Response<DeleteUserReply>, Status> {
    let req = request.into_inner();
    let data = StorageData::new(
        StorageDataType::MqttDeleteUser,
        DeleteUserRequest::encode_to_vec(&req),
    );
    if let Err(e) = raft_machine_apply.client_write(data).await {
        return Err(Status::cancelled(e.to_string()));
    };
    let storage = MqttUserStorage::new(rocksdb_engine_handler.clone());

    if let Some(user) = storage.get(&req.cluster_name, &req.user_name)? {
        if let Err(e) =
            update_cache_by_delete_user(&req.cluster_name, call_manager, client_pool, user).await
        {
            return Err(Status::cancelled(e.to_string()));
        };
    }

    Ok(Response::new(DeleteUserReply::default()))
}
