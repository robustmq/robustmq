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

use crate::core::cache::PlacementCacheManager;
use crate::mqtt::cache::MqttCacheManager;
use crate::mqtt::controller::call_broker::MQTTInnerCallManager;
use crate::mqtt::services::acl::{
    create_acl_by_req, create_blacklist_by_req, delete_acl_by_req, delete_blacklist_by_req,
    list_acl_by_req, list_blacklist_by_req,
};
use crate::mqtt::services::connector::{
    connector_heartbeat_by_req, create_connector_by_req, delete_connector_by_req,
    list_connectors_by_req, update_connector_by_req,
};
use crate::mqtt::services::session::{
    create_session_by_req, delete_session_by_req, list_session_by_req, update_session_by_req,
};
use crate::mqtt::services::share_sub::get_share_sub_leader_by_req;
use crate::mqtt::services::subscribe::{
    delete_auto_subscribe_rule_by_req, delete_subscribe_by_req, list_auto_subscribe_rule_by_req,
    list_subscribe_by_req, set_auto_subscribe_rule_by_req, set_subscribe_by_req,
};
use crate::mqtt::services::topic::{
    create_topic_by_req, create_topic_rewrite_rule_by_req, delete_topic_by_req,
    delete_topic_rewrite_rule_by_req, list_topic_by_req, list_topic_rewrite_rule_by_req,
    save_last_will_message_by_req, set_topic_retain_message_by_req,
};
use crate::mqtt::services::user::{create_user_by_req, delete_user_by_req, list_user_by_req};
use crate::route::apply::RaftMachineApply;
use crate::storage::rocksdb::RocksDBEngine;
use grpc_clients::pool::ClientPool;
use prost_validate::Validator;
use protocol::placement_center::placement_center_mqtt::mqtt_service_server::MqttService;
use protocol::placement_center::placement_center_mqtt::{
    ConnectorHeartbeatReply, ConnectorHeartbeatRequest, CreateAclReply, CreateAclRequest,
    CreateBlacklistReply, CreateBlacklistRequest, CreateConnectorReply, CreateConnectorRequest,
    CreateSessionReply, CreateSessionRequest, CreateTopicReply, CreateTopicRequest,
    CreateTopicRewriteRuleReply, CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest,
    DeleteAclReply, DeleteAclRequest, DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest,
    DeleteBlacklistReply, DeleteBlacklistRequest, DeleteConnectorReply, DeleteConnectorRequest,
    DeleteSessionReply, DeleteSessionRequest, DeleteSubscribeReply, DeleteSubscribeRequest,
    DeleteTopicReply, DeleteTopicRequest, DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest, GetShareSubLeaderReply,
    GetShareSubLeaderRequest, ListAclReply, ListAclRequest, ListAutoSubscribeRuleReply,
    ListAutoSubscribeRuleRequest, ListBlacklistReply, ListBlacklistRequest, ListConnectorReply,
    ListConnectorRequest, ListSessionReply, ListSessionRequest, ListSubscribeReply,
    ListSubscribeRequest, ListTopicReply, ListTopicRequest, ListTopicRewriteRuleReply,
    ListTopicRewriteRuleRequest, ListUserReply, ListUserRequest, SaveLastWillMessageReply,
    SaveLastWillMessageRequest, SetAutoSubscribeRuleReply, SetAutoSubscribeRuleRequest,
    SetSubscribeReply, SetSubscribeRequest, SetTopicRetainMessageReply,
    SetTopicRetainMessageRequest, UpdateConnectorReply, UpdateConnectorRequest, UpdateSessionReply,
    UpdateSessionRequest,
};
use std::pin::Pin;
use std::sync::Arc;
use tonic::codegen::tokio_stream::Stream;
use tonic::{Request, Response, Status};

pub struct GrpcMqttService {
    cluster_cache: Arc<PlacementCacheManager>,
    mqtt_cache: Arc<MqttCacheManager>,
    raft_machine_apply: Arc<RaftMachineApply>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    mqtt_call_manager: Arc<MQTTInnerCallManager>,
    client_pool: Arc<ClientPool>,
}

impl GrpcMqttService {
    pub fn new(
        cluster_cache: Arc<PlacementCacheManager>,
        mqtt_cache: Arc<MqttCacheManager>,
        raft_machine_apply: Arc<RaftMachineApply>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        mqtt_call_manager: Arc<MQTTInnerCallManager>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        GrpcMqttService {
            cluster_cache,
            mqtt_cache,
            raft_machine_apply,
            rocksdb_engine_handler,
            mqtt_call_manager,
            client_pool,
        }
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

        list_user_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        let req = request.into_inner();

        create_user_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        let req = request.into_inner();

        delete_user_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    // Session
    async fn list_session(
        &self,
        request: Request<ListSessionRequest>,
    ) -> Result<Response<ListSessionReply>, Status> {
        let req = request.into_inner();
        list_session_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionReply>, Status> {
        let req = request.into_inner();

        create_session_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn update_session(
        &self,
        request: Request<UpdateSessionRequest>,
    ) -> Result<Response<UpdateSessionReply>, Status> {
        let req = request.into_inner();

        update_session_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<DeleteSessionReply>, Status> {
        let req = request.into_inner();

        delete_session_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            &self.mqtt_cache,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    type ListTopicStream = Pin<Box<dyn Stream<Item = Result<ListTopicReply, Status>> + Send>>;

    async fn list_topic(
        &self,
        request: Request<ListTopicRequest>,
    ) -> Result<Response<Self::ListTopicStream>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        list_topic_by_req(&self.rocksdb_engine_handler, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    // Topic

    async fn create_topic(
        &self,
        request: Request<CreateTopicRequest>,
    ) -> Result<Response<CreateTopicReply>, Status> {
        let req = request.into_inner();

        create_topic_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn delete_topic(
        &self,
        request: Request<DeleteTopicRequest>,
    ) -> Result<Response<DeleteTopicReply>, Status> {
        let req = request.into_inner();

        delete_topic_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn set_topic_retain_message(
        &self,
        request: Request<SetTopicRetainMessageRequest>,
    ) -> Result<Response<SetTopicRetainMessageReply>, Status> {
        let req = request.into_inner();

        set_topic_retain_message_by_req(
            &self.raft_machine_apply,
            &self.rocksdb_engine_handler,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn get_share_sub_leader(
        &self,
        request: Request<GetShareSubLeaderRequest>,
    ) -> Result<Response<GetShareSubLeaderReply>, Status> {
        let req = request.into_inner();
        req.validate()
            .map_err(|e| Status::invalid_argument(e.to_string()))?;

        get_share_sub_leader_by_req(&self.cluster_cache, &self.rocksdb_engine_handler, &req)
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn save_last_will_message(
        &self,
        request: Request<SaveLastWillMessageRequest>,
    ) -> Result<Response<SaveLastWillMessageReply>, Status> {
        let req = request.into_inner();

        save_last_will_message_by_req(&self.raft_machine_apply, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    // ACL
    async fn list_acl(
        &self,
        request: Request<ListAclRequest>,
    ) -> Result<Response<ListAclReply>, Status> {
        let req = request.into_inner();

        list_acl_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_acl(
        &self,
        request: Request<DeleteAclRequest>,
    ) -> Result<Response<DeleteAclReply>, Status> {
        let req = request.into_inner();

        delete_acl_by_req(&self.raft_machine_apply, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn create_acl(
        &self,
        request: Request<CreateAclRequest>,
    ) -> Result<Response<CreateAclReply>, Status> {
        let req = request.into_inner();

        create_acl_by_req(&self.raft_machine_apply, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_blacklist(
        &self,
        request: Request<ListBlacklistRequest>,
    ) -> Result<Response<ListBlacklistReply>, Status> {
        let req = request.into_inner();

        list_blacklist_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_blacklist(
        &self,
        request: Request<DeleteBlacklistRequest>,
    ) -> Result<Response<DeleteBlacklistReply>, Status> {
        let req = request.into_inner();

        delete_blacklist_by_req(&self.raft_machine_apply, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn create_blacklist(
        &self,
        request: Request<CreateBlacklistRequest>,
    ) -> Result<Response<CreateBlacklistReply>, Status> {
        let req = request.into_inner();

        create_blacklist_by_req(&self.raft_machine_apply, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    // TopicRewriteRule
    async fn create_topic_rewrite_rule(
        &self,
        request: Request<CreateTopicRewriteRuleRequest>,
    ) -> Result<Response<CreateTopicRewriteRuleReply>, Status> {
        let req = request.into_inner();

        create_topic_rewrite_rule_by_req(&self.raft_machine_apply, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_topic_rewrite_rule(
        &self,
        request: Request<DeleteTopicRewriteRuleRequest>,
    ) -> Result<Response<DeleteTopicRewriteRuleReply>, Status> {
        let req = request.into_inner();

        delete_topic_rewrite_rule_by_req(&self.raft_machine_apply, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_topic_rewrite_rule(
        &self,
        request: Request<ListTopicRewriteRuleRequest>,
    ) -> Result<Response<ListTopicRewriteRuleReply>, Status> {
        let req = request.into_inner();

        list_topic_rewrite_rule_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    // Subscribe
    async fn list_subscribe(
        &self,
        request: Request<ListSubscribeRequest>,
    ) -> Result<Response<ListSubscribeReply>, Status> {
        let req = request.into_inner();

        list_subscribe_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn set_subscribe(
        &self,
        request: Request<SetSubscribeRequest>,
    ) -> Result<Response<SetSubscribeReply>, Status> {
        let req = request.into_inner();

        set_subscribe_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn delete_subscribe(
        &self,
        request: Request<DeleteSubscribeRequest>,
    ) -> Result<Response<DeleteSubscribeReply>, Status> {
        let req = request.into_inner();

        delete_subscribe_by_req(
            &self.raft_machine_apply,
            &self.rocksdb_engine_handler,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    // Connector
    async fn list_connectors(
        &self,
        request: Request<ListConnectorRequest>,
    ) -> Result<Response<ListConnectorReply>, Status> {
        let req = request.into_inner();

        list_connectors_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn create_connector(
        &self,
        request: Request<CreateConnectorRequest>,
    ) -> Result<Response<CreateConnectorReply>, Status> {
        let req = request.into_inner();

        create_connector_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn update_connector(
        &self,
        request: Request<UpdateConnectorRequest>,
    ) -> Result<Response<UpdateConnectorReply>, Status> {
        let req = request.into_inner();

        update_connector_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn delete_connector(
        &self,
        request: Request<DeleteConnectorRequest>,
    ) -> Result<Response<DeleteConnectorReply>, Status> {
        let req = request.into_inner();

        delete_connector_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &req,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn connector_heartbeat(
        &self,
        request: Request<ConnectorHeartbeatRequest>,
    ) -> Result<Response<ConnectorHeartbeatReply>, Status> {
        let req = request.into_inner();

        connector_heartbeat_by_req(&self.mqtt_cache, &req)
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    // AutoSubscribeRule
    async fn set_auto_subscribe_rule(
        &self,
        request: Request<SetAutoSubscribeRuleRequest>,
    ) -> Result<Response<SetAutoSubscribeRuleReply>, Status> {
        let req = request.into_inner();

        set_auto_subscribe_rule_by_req(&self.raft_machine_apply, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_auto_subscribe_rule(
        &self,
        request: Request<DeleteAutoSubscribeRuleRequest>,
    ) -> Result<Response<DeleteAutoSubscribeRuleReply>, Status> {
        let req = request.into_inner();

        delete_auto_subscribe_rule_by_req(&self.raft_machine_apply, &req)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_auto_subscribe_rule(
        &self,
        request: Request<ListAutoSubscribeRuleRequest>,
    ) -> Result<Response<ListAutoSubscribeRuleReply>, Status> {
        let req = request.into_inner();

        list_auto_subscribe_rule_by_req(&self.rocksdb_engine_handler, &req)
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }
}
