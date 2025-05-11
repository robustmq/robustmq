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
use std::sync::Arc;
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
        Ok(Response::new(ListSessionReply {
            sessions: list_session_by_req(&self.rocksdb_engine_handler, request)
                .map_err(|e| Status::internal(e.to_string()))?,
        }))
    }

    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<CreateSessionReply>, Status> {
        create_session_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateSessionReply {}))
    }

    async fn update_session(
        &self,
        request: Request<UpdateSessionRequest>,
    ) -> Result<Response<UpdateSessionReply>, Status> {
        update_session_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateSessionReply {}))
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<DeleteSessionReply>, Status> {
        delete_session_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            &self.mqtt_cache,
            request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteSessionReply {}))
    }

    // Topic
    async fn list_topic(
        &self,
        request: Request<ListTopicRequest>,
    ) -> Result<Response<ListTopicReply>, Status> {
        Ok(Response::new(ListTopicReply {
            topics: list_topic_by_req(&self.rocksdb_engine_handler, request)
                .map_err(|e| Status::internal(e.to_string()))?,
        }))
    }

    async fn create_topic(
        &self,
        request: Request<CreateTopicRequest>,
    ) -> Result<Response<CreateTopicReply>, Status> {
        create_topic_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateTopicReply {}))
    }

    async fn delete_topic(
        &self,
        request: Request<DeleteTopicRequest>,
    ) -> Result<Response<DeleteTopicReply>, Status> {
        delete_topic_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteTopicReply {}))
    }

    async fn set_topic_retain_message(
        &self,
        request: Request<SetTopicRetainMessageRequest>,
    ) -> Result<Response<SetTopicRetainMessageReply>, Status> {
        set_topic_retain_message_by_req(
            &self.raft_machine_apply,
            &self.rocksdb_engine_handler,
            request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SetTopicRetainMessageReply {}))
    }

    async fn get_share_sub_leader(
        &self,
        request: Request<GetShareSubLeaderRequest>,
    ) -> Result<Response<GetShareSubLeaderReply>, Status> {
        let (id, node_addr, extend_info) =
            get_share_sub_leader_by_req(&self.cluster_cache, &self.rocksdb_engine_handler, request)
                .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(GetShareSubLeaderReply {
            broker_id: id,
            broker_addr: node_addr,
            extend_info,
        }))
    }

    async fn save_last_will_message(
        &self,
        request: Request<SaveLastWillMessageRequest>,
    ) -> Result<Response<SaveLastWillMessageReply>, Status> {
        save_last_will_message_by_req(&self.raft_machine_apply, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SaveLastWillMessageReply {}))
    }

    // ACL
    async fn list_acl(
        &self,
        request: Request<ListAclRequest>,
    ) -> Result<Response<ListAclReply>, Status> {
        Ok(Response::new(ListAclReply {
            acls: list_acl_by_req(&self.rocksdb_engine_handler, request)
                .map_err(|e| Status::internal(e.to_string()))?,
        }))
    }

    async fn delete_acl(
        &self,
        request: Request<DeleteAclRequest>,
    ) -> Result<Response<DeleteAclReply>, Status> {
        delete_acl_by_req(&self.raft_machine_apply, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteAclReply {}))
    }

    async fn create_acl(
        &self,
        request: Request<CreateAclRequest>,
    ) -> Result<Response<CreateAclReply>, Status> {
        create_acl_by_req(&self.raft_machine_apply, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(CreateAclReply {}))
    }

    async fn list_blacklist(
        &self,
        request: Request<ListBlacklistRequest>,
    ) -> Result<Response<ListBlacklistReply>, Status> {
        Ok(Response::new(ListBlacklistReply {
            blacklists: list_blacklist_by_req(&self.rocksdb_engine_handler, request)
                .map_err(|e| Status::internal(e.to_string()))?,
        }))
    }

    async fn delete_blacklist(
        &self,
        request: Request<DeleteBlacklistRequest>,
    ) -> Result<Response<DeleteBlacklistReply>, Status> {
        delete_blacklist_by_req(&self.raft_machine_apply, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(DeleteBlacklistReply {}))
    }

    async fn create_blacklist(
        &self,
        request: Request<CreateBlacklistRequest>,
    ) -> Result<Response<CreateBlacklistReply>, Status> {
        create_blacklist_by_req(&self.raft_machine_apply, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(CreateBlacklistReply {}))
    }

    // TopicRewriteRule
    async fn create_topic_rewrite_rule(
        &self,
        request: Request<CreateTopicRewriteRuleRequest>,
    ) -> Result<Response<CreateTopicRewriteRuleReply>, Status> {
        create_topic_rewrite_rule_by_req(&self.raft_machine_apply, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateTopicRewriteRuleReply {}))
    }

    async fn delete_topic_rewrite_rule(
        &self,
        request: Request<DeleteTopicRewriteRuleRequest>,
    ) -> Result<Response<DeleteTopicRewriteRuleReply>, Status> {
        delete_topic_rewrite_rule_by_req(&self.raft_machine_apply, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteTopicRewriteRuleReply {}))
    }

    async fn list_topic_rewrite_rule(
        &self,
        request: Request<ListTopicRewriteRuleRequest>,
    ) -> Result<Response<ListTopicRewriteRuleReply>, Status> {
        Ok(Response::new(ListTopicRewriteRuleReply {
            topic_rewrite_rules: list_topic_rewrite_rule_by_req(
                &self.rocksdb_engine_handler,
                request,
            )
            .map_err(|e| Status::internal(e.to_string()))?,
        }))
    }

    // Subscribe
    async fn list_subscribe(
        &self,
        request: Request<ListSubscribeRequest>,
    ) -> Result<Response<ListSubscribeReply>, Status> {
        Ok(Response::new(ListSubscribeReply {
            subscribes: list_subscribe_by_req(&self.rocksdb_engine_handler, request)
                .map_err(|e| Status::internal(e.to_string()))?,
        }))
    }

    async fn set_subscribe(
        &self,
        request: Request<SetSubscribeRequest>,
    ) -> Result<Response<SetSubscribeReply>, Status> {
        set_subscribe_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SetSubscribeReply {}))
    }

    async fn delete_subscribe(
        &self,
        request: Request<DeleteSubscribeRequest>,
    ) -> Result<Response<DeleteSubscribeReply>, Status> {
        delete_subscribe_by_req(
            &self.raft_machine_apply,
            &self.rocksdb_engine_handler,
            &self.mqtt_call_manager,
            &self.client_pool,
            request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteSubscribeReply {}))
    }

    // Connector
    async fn list_connectors(
        &self,
        request: Request<ListConnectorRequest>,
    ) -> Result<Response<ListConnectorReply>, Status> {
        Ok(Response::new(ListConnectorReply {
            connectors: list_connectors_by_req(&self.rocksdb_engine_handler, request)
                .map_err(|e| Status::internal(e.to_string()))?,
        }))
    }

    async fn create_connector(
        &self,
        request: Request<CreateConnectorRequest>,
    ) -> Result<Response<CreateConnectorReply>, Status> {
        create_connector_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(CreateConnectorReply {}))
    }

    async fn update_connector(
        &self,
        request: Request<UpdateConnectorRequest>,
    ) -> Result<Response<UpdateConnectorReply>, Status> {
        update_connector_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(UpdateConnectorReply {}))
    }

    async fn delete_connector(
        &self,
        request: Request<DeleteConnectorRequest>,
    ) -> Result<Response<DeleteConnectorReply>, Status> {
        delete_connector_by_req(
            &self.rocksdb_engine_handler,
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            request,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteConnectorReply {}))
    }

    async fn connector_heartbeat(
        &self,
        request: Request<ConnectorHeartbeatRequest>,
    ) -> Result<Response<ConnectorHeartbeatReply>, Status> {
        connector_heartbeat_by_req(&self.mqtt_cache, request)
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(ConnectorHeartbeatReply {}))
    }

    // AutoSubscribeRule
    async fn set_auto_subscribe_rule(
        &self,
        request: Request<SetAutoSubscribeRuleRequest>,
    ) -> Result<Response<SetAutoSubscribeRuleReply>, Status> {
        set_auto_subscribe_rule_by_req(&self.raft_machine_apply, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SetAutoSubscribeRuleReply {}))
    }

    async fn delete_auto_subscribe_rule(
        &self,
        request: Request<DeleteAutoSubscribeRuleRequest>,
    ) -> Result<Response<DeleteAutoSubscribeRuleReply>, Status> {
        delete_auto_subscribe_rule_by_req(&self.raft_machine_apply, request)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(DeleteAutoSubscribeRuleReply {}))
    }

    async fn list_auto_subscribe_rule(
        &self,
        request: Request<ListAutoSubscribeRuleRequest>,
    ) -> Result<Response<ListAutoSubscribeRuleReply>, Status> {
        Ok(Response::new(ListAutoSubscribeRuleReply {
            auto_subscribe_rules: list_auto_subscribe_rule_by_req(
                &self.rocksdb_engine_handler,
                request,
            )
            .map_err(|e| Status::internal(e.to_string()))?,
        }))
    }
}
