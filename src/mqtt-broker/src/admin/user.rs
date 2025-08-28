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

use crate::admin::query::{apply_filters, apply_pagination, apply_sorting, Queryable};
use crate::handler::cache::CacheManager;
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
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: &ListUserRequest,
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

    let filtered = apply_filters(users, &request.options);
    let sorted = apply_sorting(filtered, &request.options);
    let pagination = apply_pagination(sorted, &request.options);

    Ok(ListUserReply {
        users: pagination.0,
        total_count: pagination.1 as u32,
    })
}

// Create a new user
pub async fn create_user_by_req(
    cache_manager: &Arc<CacheManager>,
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
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: &DeleteUserRequest,
) -> Result<DeleteUserReply, MqttBrokerError> {
    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());

    auth_driver.delete_user(request.username.clone()).await?;

    Ok(DeleteUserReply {})
}

impl Queryable for UserRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "username" => Some(self.username.clone()),
            "is_superuser" => Some(self.is_superuser.to_string()),
            _ => None,
        }
    }
}
