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
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::user::MqttUser;
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_server::MqttBrokerAdminService;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, ClusterStatusRequest, CreateUserReply, CreateUserRequest, DeleteUserReply,
    DeleteUserRequest, ListUserReply, ListUserRequest,
};
use tonic::{Request, Response, Status};

use crate::handler::cache::CacheManager;
use crate::storage::cluster::ClusterStorage;
use crate::storage::user::UserStorage;

pub struct GrpcAdminServices {
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
}

impl GrpcAdminServices {
    pub fn new(client_pool: Arc<ClientPool>, cache_manager: Arc<CacheManager>) -> Self {
        GrpcAdminServices {
            client_pool,
            cache_manager,
        }
    }
}

#[tonic::async_trait]
impl MqttBrokerAdminService for GrpcAdminServices {
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

    async fn mqtt_broker_list_user(
        &self,
        _: Request<ListUserRequest>,
    ) -> Result<Response<ListUserReply>, Status> {
        let mut reply = ListUserReply::default();

        let mut user_list = Vec::new();
        let user_storage = UserStorage::new(self.client_pool.clone());
        match user_storage.user_list().await {
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

    async fn mqtt_broker_create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        let req = request.into_inner();
        let username = req.username.clone();
        let mqtt_user = MqttUser {
            username: req.username,
            password: req.password,
            is_superuser: req.is_superuser,
        };

        let contains_admin = self.cache_manager.user_info.contains_key(&username);
        if contains_admin {
            return Err(Status::cancelled("user has beed existed"));
        };

        let user_storage = UserStorage::new(self.client_pool.clone());

        match user_storage.save_user(mqtt_user.clone()).await {
            Ok(_) => {}
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }

        self.cache_manager.add_user(mqtt_user.clone());

        return Ok(Response::new(CreateUserReply::default()));
    }

    async fn mqtt_broker_delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        let req = request.into_inner();
        let username = req.username;

        let contains_admin = self.cache_manager.user_info.contains_key(&username);
        if contains_admin {
            self.cache_manager.del_user(username.clone());
        } else {
            return Err(Status::cancelled("user does not exist"));
        }

        let user_storage = UserStorage::new(self.client_pool.clone());

        match user_storage.delete_user(username.clone()).await {
            Ok(_) => return Ok(Response::new(DeleteUserReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
