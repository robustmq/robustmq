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
use grpc_clients::poll::ClientPool;
use metadata_struct::mqtt::user::MqttUser;
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_server::MqttBrokerAdminService;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, ClusterStatusRequest, CreateUserReply, CreateUserRequest, DeleteUserReply,
    DeleteUserRequest, ListUserReply, ListUserRequest,
};
use tonic::{Request, Response, Status};

use crate::storage::cluster::ClusterStorage;
use crate::storage::user::UserStorage;

pub struct GrpcAdminServices {
    client_poll: Arc<ClientPool>,
}

impl GrpcAdminServices {
    pub fn new(client_poll: Arc<ClientPool>) -> Self {
        GrpcAdminServices { client_poll }
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
        let cluster_storage = ClusterStorage::new(self.client_poll.clone());
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

    async fn list_user(
        &self,
        _: Request<ListUserRequest>,
    ) -> Result<Response<ListUserReply>, Status> {
        let mut reply = ListUserReply::default();

        let mut user_list = Vec::new();
        let user_storage = UserStorage::new(self.client_poll.clone());
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

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        let req = request.into_inner();

        let mqtt_user = MqttUser {
            username: req.username,
            password: req.password,
            is_superuser: req.is_superuser,
        };

        let user_storage = UserStorage::new(self.client_poll.clone());

        // TODO: 参数校验
        // TODO：重复校验

        match user_storage.save_user(mqtt_user).await {
            Ok(_) => return Ok(Response::new(CreateUserReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        let req = request.into_inner();

        let user_storage = UserStorage::new(self.client_poll.clone());

        // TODO: 验证username是否存在

        match user_storage.delete_user(req.username).await {
            Ok(_) => return Ok(Response::new(DeleteUserReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
