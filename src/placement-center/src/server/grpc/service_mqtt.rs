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

use common_base::utils::vec_util;
use grpc_clients::pool::ClientPool;
use prost::Message;
use protocol::placement_center::placement_center_mqtt::mqtt_service_server::MqttService;
use protocol::placement_center::placement_center_mqtt::{
    CreateAclReply, CreateAclRequest, CreateBlacklistReply, CreateBlacklistRequest,
    CreateSessionReply, CreateSessionRequest, CreateTopicReply, CreateTopicRequest,
    CreateTopicRewriteRuleReply, CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest,
    DeleteAclReply, DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest,
    DeleteExclusiveTopicReply, DeleteExclusiveTopicRequest, DeleteSessionReply,
    DeleteSessionRequest, DeleteSubscribeReply, DeleteSubscribeRequest, DeleteTopicReply,
    DeleteTopicRequest, DeleteTopicRewriteRuleReply, DeleteTopicRewriteRuleRequest,
    DeleteUserReply, DeleteUserRequest, GetShareSubLeaderReply, GetShareSubLeaderRequest,
    ListAclReply, ListAclRequest, ListBlacklistReply, ListBlacklistRequest, ListSessionReply,
    ListSessionRequest, ListSubscribeReply, ListSubscribeRequest, ListTopicReply, ListTopicRequest,
    ListTopicRewriteRuleReply, ListTopicRewriteRuleRequest, ListUserReply, ListUserRequest,
    SaveLastWillMessageReply, SaveLastWillMessageRequest, SetExclusiveTopicReply,
    SetExclusiveTopicRequest, SetSubscribeReply, SetSubscribeRequest, SetTopicRetainMessageReply,
    SetTopicRetainMessageRequest, UpdateSessionReply, UpdateSessionRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::core::cache::PlacementCacheManager;
use crate::core::error::PlacementCenterError;
use crate::mqtt::controller::call_broker::MQTTInnerCallManager;
use crate::mqtt::services::session::{
    delete_session_by_req, list_session_by_req, save_session_by_req, update_session_by_req,
};
use crate::mqtt::services::share_sub::ShareSubLeader;
use crate::mqtt::services::subscribe::{delete_subscribe_by_req, save_subscribe_by_req};
use crate::mqtt::services::topic::{
    create_topic_by_req, delete_topic_by_req, list_topic_by_req, set_topic_retain_message_req,
};
use crate::mqtt::services::user::{delete_user_by_req, list_user_by_req, save_user_by_req};
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};
use crate::server::grpc::validate::ValidateExt;
use crate::storage::mqtt::acl::AclStorage;
use crate::storage::mqtt::blacklist::MqttBlackListStorage;
use crate::storage::mqtt::subscribe::MqttSubscribeStorage;
use crate::storage::mqtt::topic::MqttTopicStorage;
use crate::storage::rocksdb::RocksDBEngine;

pub struct GrpcMqttService {
    cluster_cache: Arc<PlacementCacheManager>,
    raft_machine_apply: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
    client_pool: Arc<ClientPool>,
}

impl GrpcMqttService {
    pub fn new(
        cluster_cache: Arc<PlacementCacheManager>,
        raft_machine_apply: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        mqtt_call_manager: Arc<MQTTInnerCallManager>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        GrpcMqttService {
            cluster_cache,
            raft_machine_apply,
            rocksdb_engine_handler,
            mqtt_call_manager,
            client_pool,
        }
    }
}

#[tonic::async_trait]
impl MqttService for GrpcMqttService {
    async fn list_user(
        &self,
        request: Request<ListUserRequest>,
    ) -> Result<Response<ListUserReply>, Status> {
        let req = request.into_inner();
        match list_user_by_req(&self.rocksdb_engine_handler, req).await {
            Ok(data) => Ok(Response::new(ListUserReply { users: data })),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        let req = request.into_inner();
        match save_user_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            req,
        )
        .await
        {
            Ok(_) => Ok(Response::new(CreateUserReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        let req = request.into_inner();
        match delete_user_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            req,
        )
        .await
        {
            Ok(_) => Ok(Response::new(DeleteUserReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn list_session(
        &self,
        request: Request<ListSessionRequest>,
    ) -> Result<Response<ListSessionReply>, Status> {
        let req = request.into_inner();
        match list_session_by_req(&self.rocksdb_engine_handler, req).await {
            Ok(data) => Ok(Response::new(ListSessionReply { sessions: data })),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionReply>, Status> {
        let req: CreateSessionRequest = request.into_inner();
        match save_session_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            req,
        )
        .await
        {
            Ok(_) => Ok(Response::new(CreateSessionReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn update_session(
        &self,
        request: Request<UpdateSessionRequest>,
    ) -> Result<Response<UpdateSessionReply>, Status> {
        let req = request.into_inner();
        match update_session_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            req,
        )
        .await
        {
            Ok(_) => Ok(Response::new(UpdateSessionReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<DeleteSessionReply>, Status> {
        let req = request.into_inner();
        match delete_session_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            req,
        )
        .await
        {
            Ok(_) => Ok(Response::new(DeleteSessionReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn list_topic(
        &self,
        request: Request<ListTopicRequest>,
    ) -> Result<Response<ListTopicReply>, Status> {
        let req = request.into_inner();
        match list_topic_by_req(&self.rocksdb_engine_handler, req).await {
            Ok(data) => Ok(Response::new(ListTopicReply { topics: data })),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn create_topic(
        &self,
        request: Request<CreateTopicRequest>,
    ) -> Result<Response<CreateTopicReply>, Status> {
        let req = request.into_inner();

        match create_topic_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            req,
        )
        .await
        {
            Ok(_) => Ok(Response::new(CreateTopicReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn delete_topic(
        &self,
        request: Request<DeleteTopicRequest>,
    ) -> Result<Response<DeleteTopicReply>, Status> {
        let req = request.into_inner();

        match delete_topic_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            req,
        )
        .await
        {
            Ok(_) => Ok(Response::new(DeleteTopicReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn set_topic_retain_message(
        &self,
        request: Request<SetTopicRetainMessageRequest>,
    ) -> Result<Response<SetTopicRetainMessageReply>, Status> {
        let req = request.into_inner();

        match set_topic_retain_message_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            req,
        )
        .await
        {
            Ok(_) => return Ok(Response::new(SetTopicRetainMessageReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn set_nx_exclusive_topic(
        &self,
        request: Request<SetExclusiveTopicRequest>,
    ) -> Result<Response<SetExclusiveTopicReply>, Status> {
        let mut reply = SetExclusiveTopicReply::default();
        let data = StorageData::new(
            StorageDataType::MqttSetNxExclusiveTopic,
            SetExclusiveTopicRequest::encode_to_vec(request.get_ref()),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(Some(resp)) => {
                reply.success = false;
                if let Some(value) = resp.data.value {
                    reply.success = vec_util::vec_to_bool(&value);
                }
                return Ok(Response::new(reply));
            }
            Ok(None) => {
                return Err(Status::cancelled(
                    PlacementCenterError::ExecutionResultIsEmpty.to_string(),
                ));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn delete_exclusive_topic(
        &self,
        request: Request<DeleteExclusiveTopicRequest>,
    ) -> Result<Response<DeleteExclusiveTopicReply>, Status> {
        let reply = DeleteExclusiveTopicReply::default();
        let data = StorageData::new(
            StorageDataType::MqttDeleteExclusiveTopic,
            DeleteExclusiveTopicRequest::encode_to_vec(request.get_ref()),
        );
        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => {
                return Ok(Response::new(reply));
            }
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn get_share_sub_leader(
        &self,
        request: Request<GetShareSubLeaderRequest>,
    ) -> Result<Response<GetShareSubLeaderReply>, Status> {
        let req = request.into_inner();
        let _ = req.validate_ext()?;
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

    async fn save_last_will_message(
        &self,
        request: Request<SaveLastWillMessageRequest>,
    ) -> Result<Response<SaveLastWillMessageReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MqttSaveLastWillMessage,
            SaveLastWillMessageRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(SaveLastWillMessageReply::default())),
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

    async fn delete_acl(
        &self,
        request: Request<DeleteAclRequest>,
    ) -> Result<Response<DeleteAclReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MqttDeleteAcl,
            DeleteAclRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(DeleteAclReply::default())),
            Err(e) => {
                return Err(Status::cancelled(e.to_string()));
            }
        }
    }

    async fn create_acl(
        &self,
        request: Request<CreateAclRequest>,
    ) -> Result<Response<CreateAclReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MqttSetAcl,
            CreateAclRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => return Ok(Response::new(CreateAclReply::default())),
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
        let blacklist_storage = MqttBlackListStorage::new(self.rocksdb_engine_handler.clone());
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

    async fn delete_blacklist(
        &self,
        request: Request<DeleteBlacklistRequest>,
    ) -> Result<Response<DeleteBlacklistReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MqttDeleteBlacklist,
            DeleteBlacklistRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => Ok(Response::new(DeleteBlacklistReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn create_blacklist(
        &self,
        request: Request<CreateBlacklistRequest>,
    ) -> Result<Response<CreateBlacklistReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MqttSetBlacklist,
            CreateBlacklistRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => Ok(Response::new(CreateBlacklistReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn create_topic_rewrite_rule(
        &self,
        request: Request<CreateTopicRewriteRuleRequest>,
    ) -> Result<Response<CreateTopicRewriteRuleReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MqttCreateTopicRewriteRule,
            CreateTopicRewriteRuleRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => Ok(Response::new(CreateTopicRewriteRuleReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn delete_topic_rewrite_rule(
        &self,
        request: Request<DeleteTopicRewriteRuleRequest>,
    ) -> Result<Response<DeleteTopicRewriteRuleReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MqttDeleteTopicRewriteRule,
            DeleteTopicRewriteRuleRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => Ok(Response::new(DeleteTopicRewriteRuleReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn list_topic_rewrite_rule(
        &self,
        request: Request<ListTopicRewriteRuleRequest>,
    ) -> Result<Response<ListTopicRewriteRuleReply>, Status> {
        let req = request.into_inner();
        let storage = MqttTopicStorage::new(self.rocksdb_engine_handler.clone());
        match storage.list_topic_rewrite_rule(&req.cluster_name) {
            Ok(data) => {
                let mut result = Vec::new();
                for raw in data {
                    result.push(raw.encode());
                }
                Ok(Response::new(ListTopicRewriteRuleReply {
                    topic_rewrite_rules: result,
                }))
            }
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn list_subscribe(
        &self,
        request: Request<ListSubscribeRequest>,
    ) -> Result<Response<ListSubscribeReply>, Status> {
        let req = request.into_inner();
        let storage = MqttSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        match storage.list_by_cluster(&req.cluster_name) {
            Ok(data) => {
                let mut subscribes = Vec::new();
                for raw in data {
                    subscribes.push(raw.encode());
                }
                Ok(Response::new(ListSubscribeReply { subscribes }))
            }
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn set_subscribe(
        &self,
        request: Request<SetSubscribeRequest>,
    ) -> Result<Response<SetSubscribeReply>, Status> {
        let req = request.into_inner();
        match save_subscribe_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            req,
        )
        .await
        {
            Ok(_) => Ok(Response::new(SetSubscribeReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn delete_subscribe(
        &self,
        request: Request<DeleteSubscribeRequest>,
    ) -> Result<Response<DeleteSubscribeReply>, Status> {
        let req = request.into_inner();
        match delete_subscribe_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            req,
        )
        .await
        {
            Ok(_) => Ok(Response::new(DeleteSubscribeReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }
}
