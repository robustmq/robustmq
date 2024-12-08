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
use metadata_struct::acl::mqtt_acl::MqttAcl;
use metadata_struct::acl::mqtt_blacklist::{MqttAclBlackList, MqttAclBlackListType};
use metadata_struct::mqtt::user::MqttUser;
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_server::MqttBrokerAdminService;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, ClusterStatusRequest, CreateAclReply, CreateAclRequest,
    CreateBlacklistReply, CreateBlacklistRequest, CreateUserReply, CreateUserRequest,
    DeleteAclReply, DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest,
    DeleteUserReply, DeleteUserRequest, EnableSlowSubScribeReply, EnableSlowSubscribeRequest,
    ListAclReply, ListAclRequest, ListBlacklistReply, ListBlacklistRequest, ListConnectionRaw,
    ListConnectionReply, ListConnectionRequest, ListSlowSubScribeRaw, ListSlowSubscribeReply,
    ListSlowSubscribeRequest, ListTopicReply, ListTopicRequest, ListUserReply, ListUserRequest,
    MqttTopic,
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
        let auth_driver = AuthDriver::new(self.cache_manager.clone(), self.client_pool.clone());
        match auth_driver.read_all_user().await {
            Ok(data) => {
                let mut users = Vec::new();
                for ele in data {
                    users.push(ele.1.encode());
                }
                reply.users = users;
                return Ok(Response::new(reply));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn mqtt_broker_list_acl(
        &self,
        _: Request<ListAclRequest>,
    ) -> Result<Response<ListAclReply>, Status> {
        let mut reply = ListAclReply::default();

        let auth_driver = AuthDriver::new(self.cache_manager.clone(), self.client_pool.clone());
        match auth_driver.read_all_acl().await {
            Ok(data) => {
                let mut acls_list = Vec::new();
                for ele in data {
                    match ele.encode() {
                        Ok(acl) => acls_list.push(acl),
                        Err(e) => return Err(Status::cancelled(e.to_string())),
                    }
                }
                reply.acls = acls_list;
                return Ok(Response::new(reply));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn mqtt_broker_create_acl(
        &self,
        request: Request<CreateAclRequest>,
    ) -> Result<Response<CreateAclReply>, Status> {
        let req = request.into_inner();

        let mqtt_acl = MqttAcl::decode(&req.acl).unwrap();

        let auth_driver = AuthDriver::new(self.cache_manager.clone(), self.client_pool.clone());
        match auth_driver.save_acl(mqtt_acl).await {
            Ok(_) => return Ok(Response::new(CreateAclReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn mqtt_broker_delete_acl(
        &self,
        request: Request<DeleteAclRequest>,
    ) -> Result<Response<DeleteAclReply>, Status> {
        let req = request.into_inner();
        let mqtt_acl = MqttAcl::decode(&req.acl).unwrap();

        let auth_driver = AuthDriver::new(self.cache_manager.clone(), self.client_pool.clone());
        match auth_driver.delete_acl(mqtt_acl).await {
            Ok(_) => return Ok(Response::new(DeleteAclReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn mqtt_broker_list_blacklist(
        &self,
        _: Request<ListBlacklistRequest>,
    ) -> Result<Response<ListBlacklistReply>, Status> {
        let mut reply = ListBlacklistReply::default();
        let auth_driver = AuthDriver::new(self.cache_manager.clone(), self.client_pool.clone());
        match auth_driver.read_all_blacklist().await {
            Ok(data) => {
                let mut blacklists = Vec::new();
                for ele in data {
                    match ele.encode() {
                        Ok(blacklist) => blacklists.push(blacklist),
                        Err(e) => return Err(Status::cancelled(e.to_string())),
                    }
                }
                reply.blacklists = blacklists;
                return Ok(Response::new(reply));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn mqtt_broker_delete_blacklist(
        &self,
        request: Request<DeleteBlacklistRequest>,
    ) -> Result<Response<DeleteBlacklistReply>, Status> {
        let req = request.into_inner();
        let mqtt_blacklist = MqttAclBlackList {
            blacklist_type: match req.blacklist_type.as_str() {
                "ClientId" => MqttAclBlackListType::ClientId,
                "User" => MqttAclBlackListType::User,
                "Ip" => MqttAclBlackListType::Ip,
                "ClientIdMatch" => MqttAclBlackListType::ClientIdMatch,
                "UserMatch" => MqttAclBlackListType::UserMatch,
                "IPCIDR" => MqttAclBlackListType::IPCIDR,
                _ => return Err(Status::cancelled("invalid blacklist type".to_string())),
            },
            resource_name: req.resource_name,
            end_time: 0,
            desc: "".to_string(),
        };

        let auth_driver = AuthDriver::new(self.cache_manager.clone(), self.client_pool.clone());
        match auth_driver.delete_blacklist(mqtt_blacklist).await {
            Ok(_) => return Ok(Response::new(DeleteBlacklistReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn mqtt_broker_create_blacklist(
        &self,
        request: Request<CreateBlacklistRequest>,
    ) -> Result<Response<CreateBlacklistReply>, Status> {
        let req = request.into_inner();
        let mqtt_blacklist = MqttAclBlackList::decode(&req.blacklist).unwrap();

        let auth_driver = AuthDriver::new(self.cache_manager.clone(), self.client_pool.clone());
        match auth_driver.save_blacklist(mqtt_blacklist).await {
            Ok(_) => return Ok(Response::new(CreateBlacklistReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
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

    async fn mqtt_broker_enable_slow_subscribe(
        &self,
        request: Request<EnableSlowSubscribeRequest>,
    ) -> Result<Response<EnableSlowSubScribeReply>, Status> {
        let subscribe_request = request.into_inner();

        match self
            .cache_manager
            .enable_slow_sub(subscribe_request.is_enable)
            .await
        {
            Ok(_) => Ok(Response::new(EnableSlowSubScribeReply {
                is_enable: subscribe_request.is_enable,
            })),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_list_slow_subscribe(
        &self,
        request: Request<ListSlowSubscribeRequest>,
    ) -> Result<Response<ListSlowSubscribeReply>, Status> {
        let _list_slow_subscribe_request = request.into_inner();
        let list_slow_subscribe_raw: Vec<ListSlowSubScribeRaw> = Vec::new();
        Ok(Response::new(ListSlowSubscribeReply {
            list_slow_subscribe_raw,
        }))
    }

    async fn mqtt_broker_list_topic(
        &self,
        request: Request<ListTopicRequest>,
    ) -> Result<Response<ListTopicReply>, Status> {
        let req = request.into_inner();
        let topic_query_result: Vec<MqttTopic> = match req.match_option {
            0 => self
                .cache_manager
                .get_topic_by_name(&req.topic_name)
                .into_iter()
                .take(10)
                .map(|entry| MqttTopic {
                    topic_id: entry.topic_id.clone(),
                    topic_name: entry.topic_name.clone(),
                    cluster_name: entry.cluster_name.clone(),
                    is_contain_retain_message: entry.retain_message.is_some(),
                })
                .collect(),
            option => self
                .cache_manager
                .topic_info
                .iter()
                .filter(|entry| match option {
                    1 => entry.value().topic_name.starts_with(&req.topic_name),
                    2 => entry.value().topic_name.contains(&req.topic_name),
                    _ => false,
                })
                .take(10)
                .map(|entry| MqttTopic {
                    topic_id: entry.value().topic_id.clone(),
                    topic_name: entry.value().topic_name.clone(),
                    cluster_name: entry.value().cluster_name.clone(),
                    is_contain_retain_message: entry.value().retain_message.is_some(),
                })
                .collect(),
        };

        let reply = ListTopicReply {
            topics: topic_query_result,
        };

        return Ok(Response::new(reply));
    }
}
