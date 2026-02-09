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

use crate::controller::call_broker::call::BrokerCallManager;
use crate::core::cache::CacheManager;
use crate::raft::manager::MultiRaftManager;
use crate::server::services::mqtt::acl::{
    create_acl_by_req, create_blacklist_by_req, delete_acl_by_req, delete_blacklist_by_req,
    list_acl_by_req, list_blacklist_by_req,
};
use crate::server::services::mqtt::connector::{
    connector_heartbeat_by_req, create_connector_by_req, delete_connector_by_req,
    list_connectors_by_req, update_connector_by_req,
};
use crate::server::services::mqtt::session::{
    create_session_by_req, delete_session_by_req, get_last_will_message_by_req,
    list_session_by_req, save_last_will_message_by_req,
};
use crate::server::services::mqtt::share_sub::get_share_sub_leader_by_req;
use crate::server::services::mqtt::subscribe::{
    delete_auto_subscribe_rule_by_req, delete_subscribe_by_req, list_auto_subscribe_rule_by_req,
    list_subscribe_by_req, set_auto_subscribe_rule_by_req, set_subscribe_by_req,
};
use crate::server::services::mqtt::topic::{
    create_topic_by_req, create_topic_rewrite_rule_by_req, delete_topic_by_req,
    delete_topic_rewrite_rule_by_req, get_topic_retain_message_by_req, list_topic_by_req,
    list_topic_rewrite_rule_by_req, set_topic_retain_message_by_req,
};
use crate::server::services::mqtt::user::{
    create_user_by_req, delete_user_by_req, list_user_by_req,
};
use grpc_clients::pool::ClientPool;
use prost_validate::Validator;
use protocol::meta::meta_service_mqtt::mqtt_service_server::MqttService;
use protocol::meta::meta_service_mqtt::{
    ConnectorHeartbeatReply, ConnectorHeartbeatRequest, CreateAclReply, CreateAclRequest,
    CreateBlacklistReply, CreateBlacklistRequest, CreateConnectorReply, CreateConnectorRequest,
    CreateSessionReply, CreateSessionRequest, CreateTopicReply, CreateTopicRequest,
    CreateTopicRewriteRuleReply, CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest,
    DeleteAclReply, DeleteAclRequest, DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest,
    DeleteBlacklistReply, DeleteBlacklistRequest, DeleteConnectorReply, DeleteConnectorRequest,
    DeleteSessionReply, DeleteSessionRequest, DeleteSubscribeReply, DeleteSubscribeRequest,
    DeleteTopicReply, DeleteTopicRequest, DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest, GetLastWillMessageReply,
    GetLastWillMessageRequest, GetShareSubLeaderReply, GetShareSubLeaderRequest,
    GetTopicRetainMessageReply, GetTopicRetainMessageRequest, ListAclReply, ListAclRequest,
    ListAutoSubscribeRuleReply, ListAutoSubscribeRuleRequest, ListBlacklistReply,
    ListBlacklistRequest, ListConnectorReply, ListConnectorRequest, ListSessionReply,
    ListSessionRequest, ListSubscribeReply, ListSubscribeRequest, ListTopicReply, ListTopicRequest,
    ListTopicRewriteRuleReply, ListTopicRewriteRuleRequest, ListUserReply, ListUserRequest,
    SaveLastWillMessageReply, SaveLastWillMessageRequest, SetAutoSubscribeRuleReply,
    SetAutoSubscribeRuleRequest, SetSubscribeReply, SetSubscribeRequest,
    SetTopicRetainMessageReply, SetTopicRetainMessageRequest, UpdateConnectorReply,
    UpdateConnectorRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::{Request, Response, Status};

pub struct GrpcMqttService {
    cache_manager: Arc<CacheManager>,
    raft_manager: Arc<MultiRaftManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    call_manager: Arc<BrokerCallManager>,
    client_pool: Arc<ClientPool>,
}

impl GrpcMqttService {
    pub fn new(
        cache_manager: Arc<CacheManager>,
        raft_manager: Arc<MultiRaftManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        call_manager: Arc<BrokerCallManager>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        GrpcMqttService {
            cache_manager,
            raft_manager,
            rocksdb_engine_handler,
            call_manager,
            client_pool,
        }
    }

    // Helper: Validate request and convert errors
    fn validate_request<T: Validator>(&self, req: &T) -> Result<(), Status> {
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))
    }

    // Helper: Convert MetaServiceError to Status
    fn to_status<E: ToString>(e: E) -> Status {
        Status::internal(e.to_string())
    }
}

#[tonic::async_trait]
impl MqttService for GrpcMqttService {
    // User
    async fn list_user(
        &self,
        request: Request<ListUserRequest>,
    ) -> Result<Response<ListUserReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_user_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        create_user_by_req(
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_user_by_req(
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    // Session
    async fn list_session(
        &self,
        request: Request<ListSessionRequest>,
    ) -> Result<Response<ListSessionReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_session_by_req(&self.cache_manager, &self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        create_session_by_req(
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<DeleteSessionReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_session_by_req(
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            &self.cache_manager,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    // Topic
    type ListTopicStream = Pin<Box<dyn Stream<Item = Result<ListTopicReply, Status>> + Send>>;

    async fn list_topic(
        &self,
        request: Request<ListTopicRequest>,
    ) -> Result<Response<Self::ListTopicStream>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_topic_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn create_topic(
        &self,
        request: Request<CreateTopicRequest>,
    ) -> Result<Response<CreateTopicReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;
        create_topic_by_req(
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn delete_topic(
        &self,
        request: Request<DeleteTopicRequest>,
    ) -> Result<Response<DeleteTopicReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_topic_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn set_topic_retain_message(
        &self,
        request: Request<SetTopicRetainMessageRequest>,
    ) -> Result<Response<SetTopicRetainMessageReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        set_topic_retain_message_by_req(&self.raft_manager, &self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn get_topic_retain_message(
        &self,
        request: Request<GetTopicRetainMessageRequest>,
    ) -> Result<Response<GetTopicRetainMessageReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        get_topic_retain_message_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // Share Subscription
    async fn get_share_sub_leader(
        &self,
        request: Request<GetShareSubLeaderRequest>,
    ) -> Result<Response<GetShareSubLeaderReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        get_share_sub_leader_by_req(&self.cache_manager, &self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // Last Will
    async fn save_last_will_message(
        &self,
        request: Request<SaveLastWillMessageRequest>,
    ) -> Result<Response<SaveLastWillMessageReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        save_last_will_message_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn get_last_will_message(
        &self,
        request: Request<GetLastWillMessageRequest>,
    ) -> Result<Response<GetLastWillMessageReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        get_last_will_message_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // ACL
    async fn list_acl(
        &self,
        request: Request<ListAclRequest>,
    ) -> Result<Response<ListAclReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_acl_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn create_acl(
        &self,
        request: Request<CreateAclRequest>,
    ) -> Result<Response<CreateAclReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        create_acl_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn delete_acl(
        &self,
        request: Request<DeleteAclRequest>,
    ) -> Result<Response<DeleteAclReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_acl_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // Blacklist
    async fn list_blacklist(
        &self,
        request: Request<ListBlacklistRequest>,
    ) -> Result<Response<ListBlacklistReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_blacklist_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn create_blacklist(
        &self,
        request: Request<CreateBlacklistRequest>,
    ) -> Result<Response<CreateBlacklistReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        create_blacklist_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn delete_blacklist(
        &self,
        request: Request<DeleteBlacklistRequest>,
    ) -> Result<Response<DeleteBlacklistReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_blacklist_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // Topic Rewrite Rule
    async fn create_topic_rewrite_rule(
        &self,
        request: Request<CreateTopicRewriteRuleRequest>,
    ) -> Result<Response<CreateTopicRewriteRuleReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        create_topic_rewrite_rule_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn delete_topic_rewrite_rule(
        &self,
        request: Request<DeleteTopicRewriteRuleRequest>,
    ) -> Result<Response<DeleteTopicRewriteRuleReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_topic_rewrite_rule_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn list_topic_rewrite_rule(
        &self,
        request: Request<ListTopicRewriteRuleRequest>,
    ) -> Result<Response<ListTopicRewriteRuleReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_topic_rewrite_rule_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // Subscribe
    async fn list_subscribe(
        &self,
        request: Request<ListSubscribeRequest>,
    ) -> Result<Response<ListSubscribeReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_subscribe_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn set_subscribe(
        &self,
        request: Request<SetSubscribeRequest>,
    ) -> Result<Response<SetSubscribeReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        set_subscribe_by_req(
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn delete_subscribe(
        &self,
        request: Request<DeleteSubscribeRequest>,
    ) -> Result<Response<DeleteSubscribeReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_subscribe_by_req(
            &self.raft_manager,
            &self.rocksdb_engine_handler,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    // Connector
    async fn list_connectors(
        &self,
        request: Request<ListConnectorRequest>,
    ) -> Result<Response<ListConnectorReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_connectors_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn create_connector(
        &self,
        request: Request<CreateConnectorRequest>,
    ) -> Result<Response<CreateConnectorReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        create_connector_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &self.cache_manager,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn update_connector(
        &self,
        request: Request<UpdateConnectorRequest>,
    ) -> Result<Response<UpdateConnectorReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        update_connector_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &self.cache_manager,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn delete_connector(
        &self,
        request: Request<DeleteConnectorRequest>,
    ) -> Result<Response<DeleteConnectorReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_connector_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_manager,
            &self.call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(Self::to_status)
        .map(Response::new)
    }

    async fn connector_heartbeat(
        &self,
        request: Request<ConnectorHeartbeatRequest>,
    ) -> Result<Response<ConnectorHeartbeatReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        connector_heartbeat_by_req(&self.cache_manager, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }

    // Auto Subscribe Rule
    async fn set_auto_subscribe_rule(
        &self,
        request: Request<SetAutoSubscribeRuleRequest>,
    ) -> Result<Response<SetAutoSubscribeRuleReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        set_auto_subscribe_rule_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn delete_auto_subscribe_rule(
        &self,
        request: Request<DeleteAutoSubscribeRuleRequest>,
    ) -> Result<Response<DeleteAutoSubscribeRuleReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        delete_auto_subscribe_rule_by_req(&self.raft_manager, &req)
            .await
            .map_err(Self::to_status)
            .map(Response::new)
    }

    async fn list_auto_subscribe_rule(
        &self,
        request: Request<ListAutoSubscribeRuleRequest>,
    ) -> Result<Response<ListAutoSubscribeRuleReply>, Status> {
        let req = request.into_inner();
        self.validate_request(&req)?;

        list_auto_subscribe_rule_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(Self::to_status)
            .map(Response::new)
    }
}
