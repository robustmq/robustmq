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

use grpc_clients::mqtt::admin::call::{
    cluster_status, mqtt_broker_create_user, mqtt_broker_delete_user, mqtt_broker_list_user,
};
use grpc_clients::poll::ClientPool;
use metadata_struct::mqtt::user::MqttUser;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusRequest, CreateUserRequest, DeleteUserRequest, ListUserRequest,
};

use crate::{error_info, grpc_addr};

#[derive(Clone)]
pub struct MqttCliCommandParam {
    pub server: String,
    pub action: MqttActionType,
}

#[derive(Clone)]
pub enum MqttActionType {
    Status,
    CreateUser(CreateUserCliRequest),
    DeleteUser(DeleteUserCliRequest),
    ListUser,
}

#[derive(Debug, Clone)]
pub struct CreateUserCliRequest {
    username: String,
    password: String,
    is_superuser: bool,
}

impl CreateUserCliRequest {
    pub fn new(username: String, password: String, is_superuser: bool) -> Self {
        CreateUserCliRequest {
            username,
            password,
            is_superuser,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeleteUserCliRequest {
    username: String,
}

impl DeleteUserCliRequest {
    pub fn new(username: String) -> Self {
        DeleteUserCliRequest { username }
    }
}

pub struct MqttBrokerCommand {}

impl Default for MqttBrokerCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl MqttBrokerCommand {
    pub fn new() -> Self {
        MqttBrokerCommand {}
    }

    pub async fn start(&self, params: MqttCliCommandParam) {
        // let action_type = MqttActionType::from(params.action.clone());
        let client_poll = Arc::new(ClientPool::new(100));
        match params.action {
            MqttActionType::Status => {
                self.status(client_poll.clone(), params.clone()).await;
            }
            MqttActionType::CreateUser(ref request) => {
                self.create_user(client_poll.clone(), params.clone(), request.clone())
                    .await;
            }
            MqttActionType::DeleteUser(ref request) => {
                self.delete_user(client_poll.clone(), params.clone(), request.clone())
                    .await;
            }
            MqttActionType::ListUser => {
                self.list_user(client_poll.clone(), params.clone()).await;
            }
        }
    }

    async fn status(&self, client_poll: Arc<ClientPool>, params: MqttCliCommandParam) {
        let request = ClusterStatusRequest {};
        match cluster_status(client_poll, grpc_addr(params.server), request).await {
            Ok(data) => {
                println!("cluster name: {}", data.cluster_name);
                println!("node list:");
                for node in data.nodes {
                    println!("- {}", node);
                }
                println!("MQTT broker cluster up and running")
            }
            Err(e) => {
                println!("MQTT broker cluster normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn create_user(
        &self,
        client_poll: Arc<ClientPool>,
        params: MqttCliCommandParam,
        cli_request: CreateUserCliRequest,
    ) {
        let request = CreateUserRequest {
            username: cli_request.username,
            password: cli_request.password,
            is_superuser: cli_request.is_superuser,
        };
        match mqtt_broker_create_user(client_poll.clone(), grpc_addr(params.server), request).await
        {
            Ok(_) => {
                println!("Created successfully!",)
            }
            Err(e) => {
                println!("MQTT broker create user normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn delete_user(
        &self,
        client_poll: Arc<ClientPool>,
        params: MqttCliCommandParam,
        cli_request: DeleteUserCliRequest,
    ) {
        let request = DeleteUserRequest {
            username: cli_request.username,
        };
        match mqtt_broker_delete_user(client_poll.clone(), grpc_addr(params.server), request).await
        {
            Ok(_) => {
                println!("Deleted successfully!");
            }
            Err(e) => {
                println!("MQTT broker delete user normal exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_user(&self, client_poll: Arc<ClientPool>, params: MqttCliCommandParam) {
        let request = ListUserRequest {};
        match mqtt_broker_list_user(client_poll.clone(), grpc_addr(params.server), request).await {
            Ok(data) => {
                println!("user list:");
                for user in data.users {
                    let mqtt_user = serde_json::from_slice::<MqttUser>(user.as_slice()).unwrap();
                    println!("{},", mqtt_user.username);
                }
            }
            Err(e) => {
                println!("MQTT broker list user exception");
                error_info(e.to_string());
            }
        }
    }
}
