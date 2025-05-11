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
use protocol::broker_mqtt::broker_mqtt_admin::{
    CreateUserRequest, DeleteUserRequest, ListUserRequest, UserRaw,
};
use std::sync::Arc;
use tonic::Request;

// List all users by request
pub async fn list_user_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<ListUserRequest>,
) -> Result<(Vec<UserRaw>, usize), MqttBrokerError> {
    let req = request.into_inner();
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

    let filtered = apply_filters(users, &req.options);
    let sorted = apply_sorting(filtered, &req.options);
    let pagination = apply_pagination(sorted, &req.options);
    Ok(pagination)
}

// Create a new user
pub async fn create_user_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<CreateUserRequest>,
) -> Result<(), MqttBrokerError> {
    let req = request.into_inner();
    let mqtt_user = MqttUser {
        username: req.username,
        password: req.password,
        is_superuser: req.is_superuser,
    };

    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());
    auth_driver.save_user(mqtt_user).await?;

    Ok(())
}

// Delete an existing user
pub async fn delete_user_by_req(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    request: Request<DeleteUserRequest>,
) -> Result<(), MqttBrokerError> {
    let req = request.into_inner();
    let auth_driver = AuthDriver::new(cache_manager.clone(), client_pool.clone());

    auth_driver.delete_user(req.username).await?;

    Ok(())
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
