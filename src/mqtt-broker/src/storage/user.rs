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

use common_config::broker::broker_config;
use dashmap::DashMap;
use grpc_clients::meta::mqtt::call::{
    placement_create_user, placement_delete_user, placement_list_user,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::user::MqttUser;
use protocol::meta::meta_service_mqtt::{CreateUserRequest, DeleteUserRequest, ListUserRequest};

use crate::handler::error::MqttBrokerError;
use crate::handler::tool::ResultMqttBrokerError;

pub struct UserStorage {
    client_pool: Arc<ClientPool>,
}
impl UserStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        UserStorage { client_pool }
    }

    pub async fn save_user(&self, user_info: MqttUser) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = CreateUserRequest {
            cluster_name: config.cluster_name.clone(),
            user_name: user_info.username.clone(),
            content: user_info.encode()?,
        };
        placement_create_user(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn delete_user(&self, user_name: String) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = DeleteUserRequest {
            cluster_name: config.cluster_name.clone(),
            user_name,
        };
        placement_delete_user(&self.client_pool, &config.get_meta_service_addr(), request).await?;
        Ok(())
    }

    pub async fn get_user(&self, username: String) -> Result<Option<MqttUser>, MqttBrokerError> {
        let config = broker_config();

        let request = ListUserRequest {
            cluster_name: config.cluster_name.clone(),
            user_name: username.clone(),
        };

        let reply =
            placement_list_user(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;

        if let Some(raw) = reply.users.first() {
            return Ok(Some(MqttUser::decode(raw)?));
        }

        Ok(None)
    }

    pub async fn user_list(&self) -> Result<DashMap<String, MqttUser>, MqttBrokerError> {
        let config = broker_config();
        let request = ListUserRequest {
            cluster_name: config.cluster_name.clone(),
            ..Default::default()
        };

        let reply =
            placement_list_user(&self.client_pool, &config.get_meta_service_addr(), request)
                .await?;

        let results = DashMap::with_capacity(2);
        for raw in reply.users {
            let data = MqttUser::decode(&raw)?;
            results.insert(data.username.clone(), data);
        }
        Ok(results)
    }
}
