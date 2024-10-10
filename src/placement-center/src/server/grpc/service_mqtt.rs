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

use prost::Message;
use protocol::placement_center::generate::common::CommonReply;
use protocol::placement_center::generate::mqtt::mqtt_service_server::MqttService;
use protocol::placement_center::generate::mqtt::{
    CreateAclRequest, CreateBlacklistRequest, CreateSessionRequest, CreateTopicRequest,
    CreateUserRequest, DeleteAclRequest, DeleteBlacklistRequest, DeleteSessionRequest,
    DeleteTopicRequest, DeleteUserRequest, GetShareSubLeaderReply, GetShareSubLeaderRequest,
    ListAclReply, ListAclRequest, ListBlacklistReply, ListBlacklistRequest, ListSessionReply,
    ListSessionRequest, ListTopicReply, ListTopicRequest, ListUserReply, ListUserRequest,
    SaveLastWillMessageRequest, SetTopicRetainMessageRequest, UpdateSessionRequest,
};
use tonic::{Request, Response, Status};

use crate::cache::placement::PlacementCacheManager;
use crate::core::share_sub::ShareSubLeader;
use crate::storage::mqtt::acl::AclStorage;
use crate::storage::mqtt::blacklist::MQTTBlackListStorage;
use crate::storage::mqtt::session::MQTTSessionStorage;
use crate::storage::mqtt::topic::MQTTTopicStorage;
use crate::storage::mqtt::user::MQTTUserStorage;
use crate::storage::rocksdb::RocksDBEngine;
use crate::storage::route::apply::RaftMachineApply;
use crate::storage::route::data::{StorageData, StorageDataType};

pub struct GrpcMqttService {
    cluster_cache: Arc<PlacementCacheManager>,
    raft_machine_apply: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GrpcMqttService {
    pub fn new(
        cluster_cache: Arc<PlacementCacheManager>,
        raft_machine_apply: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
    ) -> Self {
        GrpcMqttService {
            cluster_cache,
            raft_machine_apply,
            rocksdb_engine_handler,
        }
    }
}

#[tonic::async_trait]
impl MqttService for GrpcMqttService {
    async fn get_share_sub_leader(
        &self,
        request: Request<GetShareSubLeaderRequest>,
    ) -> Result<Response<GetShareSubLeaderReply>, Status> {
        let req = request.into_inner();
        let cluster_name = req.cluster_name;
        let group_name = req.group_name;
        let mut reply = GetShareSubLeaderReply::default();
        let share_sub = ShareSubLeader::new(
            self.cluster_cache.clone(),
            self.rocksdb_engine_handler.clone(),
        );

        let leader_broker = match share_sub.get_leader_node(&cluster_name, &group_name) {
            Ok(data) => data,
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        };

        if let Some(node) = self
            .cluster_cache
            .get_broker_node(&cluster_name, leader_broker)
        {
            reply.broker_id = leader_broker;
            reply.broker_addr = node.node_inner_addr;
            reply.extend_info = node.extend;
        }

        return Ok(Response::new(reply));
    }

    async fn list_user(
        &self,
        request: Request<ListUserRequest>,
    ) -> Result<Response<ListUserReply>, Status> {
        let req = request.into_inner();
        let storage = MQTTUserStorage::new(self.rocksdb_engine_handler.clone());

        if !req.user_name.is_empty() {
            match storage.get(&req.cluster_name, &req.user_name) {
                Ok(Some(data)) => {
                    return Ok(Response::new(ListUserReply {
                        users: vec![data.encode()],
                    }));
                }
                Ok(None) => {
                    return Ok(Response::new(ListUserReply::default()));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else {
            match storage.list(&req.cluster_name) {
                Ok(data) => {
                    let mut result = Vec::new();
                    for raw in data {
                        result.push(raw.encode());
                    }
                    return Ok(Response::new(ListUserReply { users: result }));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        let data = StorageData::new(
            StorageDataType::MQTTCreateUser,
            CreateUserRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();

        let data = StorageData::new(
            StorageDataType::MQTTDeleteUser,
            DeleteUserRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn create_topic(
        &self,
        request: Request<CreateTopicRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTCreateTopic,
            CreateTopicRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_topic(
        &self,
        request: Request<DeleteTopicRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTDeleteTopic,
            DeleteTopicRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn list_topic(
        &self,
        request: Request<ListTopicRequest>,
    ) -> Result<Response<ListTopicReply>, Status> {
        let req = request.into_inner();
        let storage = MQTTTopicStorage::new(self.rocksdb_engine_handler.clone());
        if !req.topic_name.is_empty() {
            match storage.get(&req.cluster_name, &req.topic_name) {
                Ok(Some(data)) => {
                    return Ok(Response::new(ListTopicReply {
                        topics: vec![data.encode()],
                    }));
                }
                Ok(None) => {
                    return Ok(Response::new(ListTopicReply::default()));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else {
            match storage.list(&req.cluster_name) {
                Ok(data) => {
                    let mut result = Vec::new();
                    for raw in data {
                        result.push(raw.encode());
                    }
                    return Ok(Response::new(ListTopicReply { topics: result }));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }
    }

    async fn list_session(
        &self,
        request: Request<ListSessionRequest>,
    ) -> Result<Response<ListSessionReply>, Status> {
        let req = request.into_inner();
        let storage = MQTTSessionStorage::new(self.rocksdb_engine_handler.clone());

        if !req.client_id.is_empty() {
            match storage.get(&req.cluster_name, &req.client_id) {
                Ok(Some(data)) => {
                    return Ok(Response::new(ListSessionReply {
                        sessions: vec![data.encode()],
                    }));
                }
                Ok(None) => {
                    return Ok(Response::new(ListSessionReply::default()));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        } else {
            match storage.list(&req.cluster_name) {
                Ok(data) => {
                    let mut result = Vec::new();
                    for raw in data {
                        result.push(raw.data);
                    }
                    let reply = ListSessionReply { sessions: result };
                    return Ok(Response::new(reply));
                }
                Err(e) => {
                    return Err(Status::cancelled(e.to_string()));
                }
            }
        }
    }

    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTCreateSession,
            CreateSessionRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTDeleteSession,
            DeleteSessionRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn set_topic_retain_message(
        &self,
        request: Request<SetTopicRetainMessageRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTSetTopicRetainMessage,
            SetTopicRetainMessageRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn update_session(
        &self,
        request: Request<UpdateSessionRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTUpdateSession,
            UpdateSessionRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn save_last_will_message(
        &self,
        request: Request<SaveLastWillMessageRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTSaveLastWillMessage,
            SaveLastWillMessageRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
    async fn list_acl(
        &self,
        request: Request<ListAclRequest>,
    ) -> Result<Response<ListAclReply>, Status> {
        let req = request.into_inner();
        let acl_storage = AclStorage::new(self.rocksdb_engine_handler.clone());
        match acl_storage.list(&req.cluster_name) {
            Ok(list) => {
                let mut acls = Vec::new();
                for acl in list {
                    match acl.encode() {
                        Ok(data) => {
                            acls.push(data);
                        }
                        Err(e) => {
                            return Err(Status::cancelled(e.to_string()));
                        }
                    }
                }

                return Ok(Response::new(ListAclReply { acls }));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn create_acl(
        &self,
        request: Request<CreateAclRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTCreateAcl,
            CreateAclRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_acl(
        &self,
        request: Request<DeleteAclRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTDeleteAcl,
            DeleteAclRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn list_blacklist(
        &self,
        request: Request<ListBlacklistRequest>,
    ) -> Result<Response<ListBlacklistReply>, Status> {
        let req = request.into_inner();
        let blacklist_storage = MQTTBlackListStorage::new(self.rocksdb_engine_handler.clone());
        match blacklist_storage.list(&req.cluster_name) {
            Ok(list) => {
                let mut blacklists = Vec::new();
                for acl in list {
                    match acl.encode() {
                        Ok(data) => {
                            blacklists.push(data);
                        }
                        Err(e) => {
                            return Err(Status::cancelled(e.to_string()));
                        }
                    }
                }
                return Ok(Response::new(ListBlacklistReply { blacklists }));
            }
            Err(e) => {
                println!("error:{:?}", e);
                return Err(Status::internal(e.to_string()));
            }
        }
    }

    async fn create_blacklist(
        &self,
        request: Request<CreateBlacklistRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTCreateBlacklist,
            CreateBlacklistRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_blacklist(
        &self,
        request: Request<DeleteBlacklistRequest>,
    ) -> Result<Response<CommonReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MQTTDeleteBlacklist,
            DeleteBlacklistRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CommonReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }
}
