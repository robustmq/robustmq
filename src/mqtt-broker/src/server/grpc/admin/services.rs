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

use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::tools::serialize_value;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::user::MqttUser;
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_server::MqttBrokerAdminService;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, ClusterStatusRequest, CreateUserReply, CreateUserRequest, DeleteUserReply,
    DeleteUserRequest, ListConnectionRaw, ListConnectionReply, ListConnectionRequest,
    ListUserReply, ListUserRequest,
};
use tonic::{Request, Response, Status};

use crate::handler::cache::CacheManager;
use crate::security::AuthDriver;
use crate::server::connection_manager::ConnectionManager;
use crate::storage::cluster::ClusterStorage;

pub struct GrpcAdminServices {
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
}

impl GrpcAdminServices {
    pub fn new(
        client_pool: Arc<ClientPool>,
        cache_manager: Arc<CacheManager>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        GrpcAdminServices {
            client_pool,
            cache_manager,
            connection_manager,
        }
    }
}

#[tonic::async_trait]
impl MqttBrokerAdminService for GrpcAdminServices {
    // --- cluster ---
    async fn cluster_status(
        &self,
        _: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusReply>, Status> {
        let mut reply = ClusterStatusReply::default();
        let config = broker_mqtt_conf();
        reply.cluster_name = config.cluster_name.clone();
        let mut broker_node_list = Vec::new();
        let cluster_storage = ClusterStorage::new(self.client_pool.clone());
        match cluster_storage.node_list().await {
            Ok(data) => {
                for node in data {
                    broker_node_list.push(format!("{}@{}", node.node_ip, node.node_id));
                }
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
        reply.nodes = broker_node_list;
        return Ok(Response::new(reply));
    }

    // --- user ---
    async fn mqtt_broker_create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        let req = request.into_inner();
        let mqtt_user = MqttUser {
            username: req.username,
            password: req.password,
            is_superuser: req.is_superuser,
        };

        let auth_driver = AuthDriver::new(self.cache_manager.clone(), self.client_pool.clone());
        match auth_driver.save_user(mqtt_user).await {
            Ok(_) => {
                return Ok(Response::new(CreateUserReply::default()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn mqtt_broker_delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        let req = request.into_inner();

        let auth_driver = AuthDriver::new(self.cache_manager.clone(), self.client_pool.clone());
        match auth_driver.delete_user(req.username).await {
            Ok(_) => return Ok(Response::new(DeleteUserReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn mqtt_broker_list_user(
        &self,
        _: Request<ListUserRequest>,
    ) -> Result<Response<ListUserReply>, Status> {
        let mut reply = ListUserReply::default();

        let mut user_list = Vec::new();
        let auth_driver = AuthDriver::new(self.cache_manager.clone(), self.client_pool.clone());
        match auth_driver.read_all_user().await {
            Ok(date) => {
                date.iter()
                    .for_each(|user| user_list.push(user.value().encode()));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
        reply.users = user_list;
        return Ok(Response::new(reply));
    }

    // --- connection ---
    async fn mqtt_broker_list_connection(
        &self,
        _: Request<ListConnectionRequest>,
    ) -> Result<Response<ListConnectionReply>, Status> {
        let mut reply = ListConnectionReply::default();
        let mut list_connection_raw: Vec<ListConnectionRaw> = Vec::new();
        for (key, value) in self.connection_manager.list_connect() {
            if let Some(mqtt_value) = self.cache_manager.connection_info.clone().get(&key) {
                let mqtt_info = serialize_value(mqtt_value.value())?;
                let raw = ListConnectionRaw {
                    connection_id: value.connection_id,
                    connection_type: value.connection_type.to_string(),
                    protocol: match value.protocol {
                        Some(protocol) => protocol.into(),
                        None => "None".to_string(),
                    },
                    source_addr: value.addr.to_string(),
                    info: mqtt_info,
                };
                list_connection_raw.push(raw);
            }
        }
        reply.list_connection_raw = list_connection_raw;
        Ok(Response::new(reply))
    }
}
