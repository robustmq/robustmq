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

use crate::handler::cache::MQTTCacheManager;
use crate::handler::error::MqttBrokerError;
use crate::security::AuthDriver;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::user::MqttUser;
use protocol::broker::broker_mqtt_admin::{
    CreateUserReply, CreateUserRequest, DeleteUserReply, DeleteUserRequest, ListUserReply,
    ListUserRequest, UserRaw,
};
use std::sync::Arc;

// List all users by request
pub async fn list_user_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    _request: &ListUserRequest,
) -> Result<ListUserReply, MqttBrokerError> {
    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());

    let data = auth_driver.read_all_user().await?;

    let mut users = Vec::new();
    for ele in data {
        let user_raw = UserRaw {
            username: ele.1.username,
            is_superuser: ele.1.is_superuser,
        };
        users.push(user_raw);
    }
    Ok(ListUserReply { users })
}

// Create a new user
pub async fn create_user_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    request: &CreateUserRequest,
) -> Result<CreateUserReply, MqttBrokerError> {
    let mqtt_user = MqttUser {
        username: request.username.clone(),
        password: request.password.clone(),
        is_superuser: request.is_superuser,
    };

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    auth_driver.save_user(mqtt_user).await?;

    Ok(CreateUserReply {})
}

// Delete an existing user
pub async fn delete_user_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    client_pool: &Arc<ClientPool>,
    request: &DeleteUserRequest,
) -> Result<DeleteUserReply, MqttBrokerError> {
    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());

    auth_driver.delete_user(request.username.clone()).await?;

    Ok(DeleteUserReply {})
}
