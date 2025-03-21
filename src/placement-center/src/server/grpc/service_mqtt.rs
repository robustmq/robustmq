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
use crate::core::error::PlacementCenterError;
use crate::mqtt::cache::MqttCacheManager;
use crate::mqtt::connector::request::connector_heartbeat_by_req;
use crate::mqtt::connector::status::save_connector;
use crate::mqtt::controller::call_broker::{
    update_cache_by_delete_connector, MQTTInnerCallManager,
};
use crate::mqtt::services::acl::{
    create_acl_by_req, create_blacklist_by_req, delete_acl_by_req, delete_blacklist_by_req,
    list_acl_by_req, list_blacklist_by_req,
};
use crate::mqtt::services::session::{
    create_session_by_req, delete_session_by_req, list_session_by_req, update_session_by_req,
};
use crate::mqtt::services::share_sub::get_share_sub_leader_by_req;
use crate::mqtt::services::subscribe::{
    delete_subscribe_by_req, list_subscribe_by_req, set_subscribe_by_req,
};
use crate::mqtt::services::topic::{
    create_topic_by_req, create_topic_rewrite_rule_by_req, delete_topic_by_req,
    delete_topic_rewrite_rule_by_req, list_topic_by_req, list_topic_rewrite_rule_by_req,
    save_last_will_message_by_req, set_topic_retain_message_by_req,
};
use crate::mqtt::services::user::{create_user_by_req, delete_user_by_req, list_user_by_req};
use crate::route::apply::RaftMachineApply;
use crate::route::data::{StorageData, StorageDataType};
use crate::storage::mqtt::connector::MqttConnectorStorage;
use crate::storage::mqtt::subscribe::MqttSubscribeStorage;
use crate::storage::rocksdb::RocksDBEngine;
use grpc_clients::pool::ClientPool;
use prost::Message;
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
        list_user_by_req(&self.rocksdb_engine_handler, request)
    }

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        create_user_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            request,
        )
        .await
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        delete_user_by_req(
            &self.raft_machine_apply,
            &self.mqtt_call_manager,
            &self.client_pool,
            &self.rocksdb_engine_handler,
            request,
        )
        .await
    }

    // Session
    async fn list_session(
        &self,
        request: Request<ListSessionRequest>,
    ) -> Result<Response<ListSessionReply>, Status> {
        list_session_by_req(&self.rocksdb_engine_handler, request)
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
            request,
        )
        .await
    }

    // Topic
    async fn list_topic(
        &self,
        request: Request<ListTopicRequest>,
    ) -> Result<Response<ListTopicReply>, Status> {
        list_topic_by_req(&self.rocksdb_engine_handler, request)
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
    }

    async fn get_share_sub_leader(
        &self,
        request: Request<GetShareSubLeaderRequest>,
    ) -> Result<Response<GetShareSubLeaderReply>, Status> {
        get_share_sub_leader_by_req(&self.cluster_cache, &self.rocksdb_engine_handler, request)
    }

    async fn save_last_will_message(
        &self,
        request: Request<SaveLastWillMessageRequest>,
    ) -> Result<Response<SaveLastWillMessageReply>, Status> {
        save_last_will_message_by_req(&self.raft_machine_apply, request).await
    }

    // ACL
    async fn list_acl(
        &self,
        request: Request<ListAclRequest>,
    ) -> Result<Response<ListAclReply>, Status> {
        list_acl_by_req(&self.rocksdb_engine_handler, request)
    }

    async fn delete_acl(
        &self,
        request: Request<DeleteAclRequest>,
    ) -> Result<Response<DeleteAclReply>, Status> {
        delete_acl_by_req(&self.raft_machine_apply, request).await
    }

    async fn create_acl(
        &self,
        request: Request<CreateAclRequest>,
    ) -> Result<Response<CreateAclReply>, Status> {
        create_acl_by_req(&self.raft_machine_apply, request).await
    }

    // BlackList
    async fn list_blacklist(
        &self,
        request: Request<ListBlacklistRequest>,
    ) -> Result<Response<ListBlacklistReply>, Status> {
        list_blacklist_by_req(&self.rocksdb_engine_handler, request)
    }

    async fn delete_blacklist(
        &self,
        request: Request<DeleteBlacklistRequest>,
    ) -> Result<Response<DeleteBlacklistReply>, Status> {
        delete_blacklist_by_req(&self.raft_machine_apply, request).await
    }

    async fn create_blacklist(
        &self,
        request: Request<CreateBlacklistRequest>,
    ) -> Result<Response<CreateBlacklistReply>, Status> {
        create_blacklist_by_req(&self.raft_machine_apply, request).await
    }

    // TopicRewriteRule
    async fn create_topic_rewrite_rule(
        &self,
        request: Request<CreateTopicRewriteRuleRequest>,
    ) -> Result<Response<CreateTopicRewriteRuleReply>, Status> {
        create_topic_rewrite_rule_by_req(&self.raft_machine_apply, request).await
    }

    async fn delete_topic_rewrite_rule(
        &self,
        request: Request<DeleteTopicRewriteRuleRequest>,
    ) -> Result<Response<DeleteTopicRewriteRuleReply>, Status> {
        delete_topic_rewrite_rule_by_req(&self.raft_machine_apply, request).await
    }

    async fn list_topic_rewrite_rule(
        &self,
        request: Request<ListTopicRewriteRuleRequest>,
    ) -> Result<Response<ListTopicRewriteRuleReply>, Status> {
        list_topic_rewrite_rule_by_req(&self.rocksdb_engine_handler, request)
    }

    // Subscribe
    async fn list_subscribe(
        &self,
        request: Request<ListSubscribeRequest>,
    ) -> Result<Response<ListSubscribeReply>, Status> {
        list_subscribe_by_req(&self.rocksdb_engine_handler, request)
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
    }

    // Connector
    async fn list_connectors(
        &self,
        request: Request<ListConnectorRequest>,
    ) -> Result<Response<ListConnectorReply>, Status> {
        let req = request.into_inner();
        let storage = MqttConnectorStorage::new(self.rocksdb_engine_handler.clone());

        if !req.connector_name.is_empty() {
            if let Some(data) = storage.get(&req.cluster_name, &req.connector_name)? {
                let data = vec![data.encode()];
                return Ok(Response::new(ListConnectorReply { connectors: data }));
            }
        } else {
            let data = storage.list(&req.cluster_name)?;
            let mut result = Vec::new();
            for raw in data {
                result.push(raw.encode());
            }
            return Ok(Response::new(ListConnectorReply { connectors: result }));
        }
        Ok(Response::new(ListConnectorReply {
            connectors: Vec::new(),
        }))
    }

    async fn create_connector(
        &self,
        request: Request<CreateConnectorRequest>,
    ) -> Result<Response<CreateConnectorReply>, Status> {
        let req = request.into_inner();
        let storage = MqttConnectorStorage::new(self.rocksdb_engine_handler.clone());
        let connector = storage.get(&req.cluster_name, &req.connector_name)?;
        if connector.is_some() {
            return Err(Status::cancelled(
                PlacementCenterError::ConnectorAlreadyExist(req.connector_name).to_string(),
            ));
        }

        if let Err(e) = save_connector(
            &self.raft_machine_apply,
            req,
            &self.mqtt_call_manager,
            &self.client_pool,
        )
        .await
        {
            return Err(Status::cancelled(e.to_string()));
        };
        Ok(Response::new(CreateConnectorReply::default()))
    }

    async fn update_connector(
        &self,
        request: Request<UpdateConnectorRequest>,
    ) -> Result<Response<UpdateConnectorReply>, Status> {
        let req = request.into_inner();
        let storage = MqttConnectorStorage::new(self.rocksdb_engine_handler.clone());
        let connector = storage.get(&req.cluster_name, &req.connector_name)?;
        if connector.is_none() {
            return Err(Status::cancelled(
                PlacementCenterError::ConnectorNotFound(req.connector_name).to_string(),
            ));
        }

        let create_req = CreateConnectorRequest {
            cluster_name: req.cluster_name.clone(),
            connector_name: req.connector_name.clone(),
            connector: req.connector.clone(),
        };

        if let Err(e) = save_connector(
            &self.raft_machine_apply,
            create_req,
            &self.mqtt_call_manager,
            &self.client_pool,
        )
        .await
        {
            return Err(Status::cancelled(e.to_string()));
        };

        Ok(Response::new(UpdateConnectorReply::default()))
    }

    async fn delete_connector(
        &self,
        request: Request<DeleteConnectorRequest>,
    ) -> Result<Response<DeleteConnectorReply>, Status> {
        let req = request.into_inner();
        let storage = MqttConnectorStorage::new(self.rocksdb_engine_handler.clone());
        let connector = storage.get(&req.cluster_name, &req.connector_name)?;
        if connector.is_none() {
            return Err(Status::cancelled(
                PlacementCenterError::ConnectorNotFound(req.connector_name).to_string(),
            ));
        }
        let data = StorageData::new(
            StorageDataType::MqttDeleteConnector,
            DeleteConnectorRequest::encode_to_vec(&req),
        );
        if let Err(e) = self.raft_machine_apply.client_write(data).await {
            return Err(Status::cancelled(e.to_string()));
        };

        if let Err(e) = update_cache_by_delete_connector(
            &req.cluster_name,
            &self.mqtt_call_manager,
            &self.client_pool,
            connector.unwrap(),
        )
        .await
        {
            return Err(Status::cancelled(e.to_string()));
        };

        Ok(Response::new(DeleteConnectorReply::default()))
    }

    async fn connector_heartbeat(
        &self,
        request: Request<ConnectorHeartbeatRequest>,
    ) -> Result<Response<ConnectorHeartbeatReply>, Status> {
        connector_heartbeat_by_req(&self.mqtt_cache, request)
    }

    // AutoSubscribeRule
    async fn set_auto_subscribe_rule(
        &self,
        request: Request<SetAutoSubscribeRuleRequest>,
    ) -> Result<Response<SetAutoSubscribeRuleReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MqttSetAutoSubscribeRule,
            SetAutoSubscribeRuleRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => Ok(Response::new(SetAutoSubscribeRuleReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn delete_auto_subscribe_rule(
        &self,
        request: Request<DeleteAutoSubscribeRuleRequest>,
    ) -> Result<Response<DeleteAutoSubscribeRuleReply>, Status> {
        let req = request.into_inner();
        let data = StorageData::new(
            StorageDataType::MqttDeleteAutoSubscribeRule,
            DeleteAutoSubscribeRuleRequest::encode_to_vec(&req),
        );

        match self.raft_machine_apply.client_write(data).await {
            Ok(_) => Ok(Response::new(DeleteAutoSubscribeRuleReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn list_auto_subscribe_rule(
        &self,
        request: Request<ListAutoSubscribeRuleRequest>,
    ) -> Result<Response<ListAutoSubscribeRuleReply>, Status> {
        let req = request.into_inner();
        let storage = MqttSubscribeStorage::new(self.rocksdb_engine_handler.clone());
        match storage.list_auto_subscribe_rule(&req.cluster_name) {
            Ok(data) => {
                let mut result = Vec::new();
                for raw in data {
                    result.push(raw.encode());
                }
                Ok(Response::new(ListAutoSubscribeRuleReply {
                    auto_subscribe_rules: result,
                }))
            }
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }
}
