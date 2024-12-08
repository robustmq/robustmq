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
    cluster_status, mqtt_broker_create_user, mqtt_broker_delete_user,
    mqtt_broker_enable_slow_subscribe, mqtt_broker_list_connection,
    mqtt_broker_list_slow_subscribe, mqtt_broker_list_topic, mqtt_broker_list_user,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::user::MqttUser;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusRequest, CreateUserRequest, DeleteUserRequest, EnableSlowSubscribeRequest,
    ListConnectionRequest, ListSlowSubscribeRequest, ListTopicRequest, ListUserRequest,
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

    // User admin
    CreateUser(CreateUserRequest),
    DeleteUser(DeleteUserRequest),
    ListUser,

    // connection
    ListConnection,

    // observability: slow-ub
    EnableSlowSubscribe(EnableSlowSubscribeRequest),
    ListSlowSubscribe,

    ListTopic(ListTopicRequest),
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
        let client_pool = Arc::new(ClientPool::new(100));
        match params.action {
            MqttActionType::Status => {
                self.status(client_pool.clone(), params.clone()).await;
            }
            MqttActionType::CreateUser(ref request) => {
                self.create_user(client_pool.clone(), params.clone(), request.clone())
                    .await;
            }
            MqttActionType::DeleteUser(ref request) => {
                self.delete_user(client_pool.clone(), params.clone(), request.clone())
                    .await;
            }
            MqttActionType::ListUser => {
                self.list_user(client_pool.clone(), params.clone()).await;
            }
            MqttActionType::ListConnection => {
                self.list_connections(client_pool.clone(), params.clone())
                    .await;
            }
            MqttActionType::EnableSlowSubscribe(ref request) => {
                self.enable_slow_subscribe(client_pool.clone(), params.clone(), request.clone())
                    .await;
            }
            MqttActionType::ListTopic(ref request) => {
                self.list_topic(client_pool.clone(), params.clone(), request.clone())
                    .await;
            }
            MqttActionType::ListSlowSubscribe => {
                self.list_slow_subscribe(client_pool.clone(), params.clone())
                    .await;
            }
        }
    }

    async fn status(&self, client_pool: Arc<ClientPool>, params: MqttCliCommandParam) {
        let request = ClusterStatusRequest {};
        match cluster_status(client_pool, &grpc_addr(params.server), request).await {
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
        client_pool: Arc<ClientPool>,
        params: MqttCliCommandParam,
        cli_request: CreateUserRequest,
    ) {
        match mqtt_broker_create_user(client_pool.clone(), &grpc_addr(params.server), cli_request)
            .await
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
        client_pool: Arc<ClientPool>,
        params: MqttCliCommandParam,
        cli_request: DeleteUserRequest,
    ) {
        match mqtt_broker_delete_user(client_pool.clone(), &grpc_addr(params.server), cli_request)
            .await
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

    async fn list_user(&self, client_pool: Arc<ClientPool>, params: MqttCliCommandParam) {
        let request = ListUserRequest {};
        match mqtt_broker_list_user(client_pool.clone(), &grpc_addr(params.server), request).await {
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

    async fn list_connections(&self, client_pool: Arc<ClientPool>, params: MqttCliCommandParam) {
        let request = ListConnectionRequest {};
        match mqtt_broker_list_connection(client_pool.clone(), &grpc_addr(params.server), request)
            .await
        {
            Ok(data) => {
                println!("connection list:");
                for raw in data.list_connection_raw {
                    println!("{:?}", raw)
                }
            }
            Err(e) => {
                println!("MQTT broker list connection exception");
                error_info(e.to_string());
            }
        }
    }

    // ---------------- observability ----------------
    // ------------ slow subscribe features ----------
    async fn enable_slow_subscribe(
        &self,
        client_pool: Arc<ClientPool>,
        params: MqttCliCommandParam,
        cli_request: EnableSlowSubscribeRequest,
    ) {
        match mqtt_broker_enable_slow_subscribe(
            client_pool.clone(),
            &grpc_addr(params.server),
            cli_request,
        )
        .await
        {
            Ok(reply) => {
                if reply.is_enable {
                    println!("The slow subscription feature has been successfully enabled.");
                } else {
                    println!("The slow subscription feature has been successfully closed.");
                }
            }

            Err(e) => {
                println!("The slow subscription feature failed to enable, with the specific reason being:");
                error_info(e.to_string());
            }
        }
    }

    async fn list_slow_subscribe(&self, client_pool: Arc<ClientPool>, params: MqttCliCommandParam) {
        let request = ListSlowSubscribeRequest {};
        match mqtt_broker_list_slow_subscribe(
            client_pool.clone(),
            &grpc_addr(params.server),
            request,
        )
        .await
        {
            Ok(data) => {
                println!("{:?}", data)
            }
            Err(e) => {
                println!("MQTT broker list slow subscribe info exception");
                error_info(e.to_string());
            }
        }
    }

    async fn list_topic(
        &self,
        client_pool: Arc<ClientPool>,
        params: MqttCliCommandParam,
        cli_request: ListTopicRequest,
    ) {
        match mqtt_broker_list_topic(client_pool.clone(), &grpc_addr(params.server), cli_request)
            .await
        {
            Ok(data) => {
                println!("topic list result:");
                for mqtt_topic in data.topics {
                    println!(
                        concat!(
                            "topic id: {}\n",
                            "topic name: {}\n",
                            "cluster name: {}\n",
                            "is contain retain message: {}\n"
                        ),
                        mqtt_topic.topic_id,
                        mqtt_topic.topic_name,
                        mqtt_topic.cluster_name,
                        mqtt_topic.is_contain_retain_message
                    );
                }
            }
            Err(e) => {
                println!("MQTT broker list topic exception");
                error_info(e.to_string());
            }
        }
    }
}
