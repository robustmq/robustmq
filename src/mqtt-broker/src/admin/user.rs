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

use crate::handler::cache::CacheManager;
use crate::security::AuthDriver;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::user::MqttUser;
use protocol::broker_mqtt::broker_mqtt_admin::{
    CreateUserReply, CreateUserRequest, DeleteUserReply, DeleteUserRequest, ListUserReply,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub async fn create_user_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<CreateUserRequest>,
) -> Result<Response<CreateUserReply>, Status> {
    let req = request.into_inner();
    let mqtt_user = MqttUser {
        username: req.username,
        password: req.password,
        is_superuser: req.is_superuser,
    };

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.save_user(mqtt_user).await {
        Ok(_) => Ok(Response::new(CreateUserReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn delete_user_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<DeleteUserRequest>,
) -> Result<Response<DeleteUserReply>, Status> {
    let req = request.into_inner();

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.delete_user(req.username).await {
        Ok(_) => Ok(Response::new(DeleteUserReply::default())),
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}

pub async fn list_user_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
) -> Result<Response<ListUserReply>, Status> {
    let mut reply = ListUserReply::default();
    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    match auth_driver.read_all_user().await {
        Ok(data) => {
            let mut users = Vec::new();
            for ele in data {
                users.push(ele.1.encode());
            }
            reply.users = users;
            Ok(Response::new(reply))
        }
        Err(e) => Err(Status::cancelled(e.to_string())),
    }
}
