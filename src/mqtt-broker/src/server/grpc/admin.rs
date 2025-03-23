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

use crate::admin::acl::{
    create_acl_by_req, create_blacklist_by_req, delete_acl_by_req, delete_blacklist_by_req,
    list_acl_by_req, list_blacklist_by_req,
};
use crate::admin::connector::{
    create_connector_by_req, delete_connector_by_req, list_connector_by_req,
    update_connector_by_req,
};
use crate::admin::subscribe::{
    delete_auto_subscribe_rule, list_auto_subscribe_rule_by_req, set_auto_subscribe_rule,
};
use crate::admin::topic::{
    create_topic_rewrite_rule_by_req, delete_topic_rewrite_rule_by_req, list_topic_by_req,
};
use crate::admin::user::{create_user_by_req, delete_user_by_req, list_user_by_req};
use crate::admin::{
    cluster_status_by_req, enable_flapping_detect_by_req, enable_slow_subscribe_by_req,
    list_connection_by_req, list_slow_subscribe_by_req,
};
use crate::handler::cache::CacheManager;
use crate::server::connection_manager::ConnectionManager;
use crate::storage::schema::{
    bind_schema_by_req, create_schema_by_req, delete_schema_by_req, list_bind_schema_by_req,
    list_schema_by_req, unbind_schema_by_req, update_schema_by_req,
};
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_server::MqttBrokerAdminService;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, ClusterStatusRequest, CreateAclReply, CreateAclRequest,
    CreateBlacklistReply, CreateBlacklistRequest, CreateTopicRewriteRuleReply,
    CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest, DeleteAclReply,
    DeleteAclRequest, DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest,
    DeleteBlacklistReply, DeleteBlacklistRequest, DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest, EnableFlappingDetectReply,
    EnableFlappingDetectRequest, EnableSlowSubScribeReply, EnableSlowSubscribeRequest,
    ListAclReply, ListAclRequest, ListAutoSubscribeRuleReply, ListAutoSubscribeRuleRequest,
    ListBlacklistReply, ListBlacklistRequest, ListConnectionReply, ListConnectionRequest,
    ListSlowSubscribeReply, ListSlowSubscribeRequest, ListTopicReply, ListTopicRequest,
    ListUserReply, ListUserRequest, MqttBindSchemaReply, MqttBindSchemaRequest,
    MqttCreateConnectorReply, MqttCreateConnectorRequest, MqttCreateSchemaReply,
    MqttCreateSchemaRequest, MqttDeleteConnectorReply, MqttDeleteConnectorRequest,
    MqttDeleteSchemaReply, MqttDeleteSchemaRequest, MqttListBindSchemaReply,
    MqttListBindSchemaRequest, MqttListConnectorReply, MqttListConnectorRequest,
    MqttListSchemaReply, MqttListSchemaRequest, MqttUnbindSchemaReply, MqttUnbindSchemaRequest,
    MqttUpdateConnectorReply, MqttUpdateConnectorRequest, MqttUpdateSchemaReply,
    MqttUpdateSchemaRequest, SetAutoSubscribeRuleReply, SetAutoSubscribeRuleRequest,
};
use tonic::{Request, Response, Status};

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
        match cluster_status_by_req(&self.client_pool).await {
            Ok(reply) => Ok(Response::new(reply)),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    // --- user ---
    async fn mqtt_broker_create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        create_user_by_req(&self.cache_manager, &self.client_pool, request).await
    }

    async fn mqtt_broker_delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        delete_user_by_req(&self.cache_manager, &self.client_pool, request).await
    }

    async fn mqtt_broker_list_user(
        &self,
        _: Request<ListUserRequest>,
    ) -> Result<Response<ListUserReply>, Status> {
        list_user_by_req(&self.cache_manager, &self.client_pool).await
    }

    async fn mqtt_broker_list_acl(
        &self,
        _: Request<ListAclRequest>,
    ) -> Result<Response<ListAclReply>, Status> {
        list_acl_by_req(&self.cache_manager, &self.client_pool).await
    }

    async fn mqtt_broker_create_acl(
        &self,
        request: Request<CreateAclRequest>,
    ) -> Result<Response<CreateAclReply>, Status> {
        create_acl_by_req(&self.cache_manager, &self.client_pool, request).await
    }

    async fn mqtt_broker_delete_acl(
        &self,
        request: Request<DeleteAclRequest>,
    ) -> Result<Response<DeleteAclReply>, Status> {
        delete_acl_by_req(&self.cache_manager, &self.client_pool, request).await
    }

    async fn mqtt_broker_list_blacklist(
        &self,
        _: Request<ListBlacklistRequest>,
    ) -> Result<Response<ListBlacklistReply>, Status> {
        list_blacklist_by_req(&self.cache_manager, &self.client_pool).await
    }

    async fn mqtt_broker_delete_blacklist(
        &self,
        request: Request<DeleteBlacklistRequest>,
    ) -> Result<Response<DeleteBlacklistReply>, Status> {
        delete_blacklist_by_req(&self.cache_manager, &self.client_pool, request).await
    }

    async fn mqtt_broker_create_blacklist(
        &self,
        request: Request<CreateBlacklistRequest>,
    ) -> Result<Response<CreateBlacklistReply>, Status> {
        create_blacklist_by_req(&self.cache_manager, &self.client_pool, request).await
    }

    async fn mqtt_broker_enable_flapping_detect(
        &self,
        request: Request<EnableFlappingDetectRequest>,
    ) -> Result<Response<EnableFlappingDetectReply>, Status> {
        enable_flapping_detect_by_req(&self.cache_manager, request).await
    }

    // --- connection ---
    async fn mqtt_broker_list_connection(
        &self,
        _: Request<ListConnectionRequest>,
    ) -> Result<Response<ListConnectionReply>, Status> {
        list_connection_by_req(&self.connection_manager, &self.cache_manager)
    }

    async fn mqtt_broker_enable_slow_subscribe(
        &self,
        request: Request<EnableSlowSubscribeRequest>,
    ) -> Result<Response<EnableSlowSubScribeReply>, Status> {
        enable_slow_subscribe_by_req(&self.cache_manager, request).await
    }

    async fn mqtt_broker_list_slow_subscribe(
        &self,
        request: Request<ListSlowSubscribeRequest>,
    ) -> Result<Response<ListSlowSubscribeReply>, Status> {
        list_slow_subscribe_by_req(&self.cache_manager, request)
    }

    async fn mqtt_broker_list_topic(
        &self,
        request: Request<ListTopicRequest>,
    ) -> Result<Response<ListTopicReply>, Status> {
        list_topic_by_req(&self.cache_manager, request)
    }

    async fn mqtt_broker_delete_topic_rewrite_rule(
        &self,
        request: Request<DeleteTopicRewriteRuleRequest>,
    ) -> Result<Response<DeleteTopicRewriteRuleReply>, Status> {
        delete_topic_rewrite_rule_by_req(&self.client_pool, &self.cache_manager, request).await
    }

    async fn mqtt_broker_create_topic_rewrite_rule(
        &self,
        request: Request<CreateTopicRewriteRuleRequest>,
    ) -> Result<Response<CreateTopicRewriteRuleReply>, Status> {
        create_topic_rewrite_rule_by_req(&self.client_pool, &self.cache_manager, request).await
    }

    async fn mqtt_broker_list_connector(
        &self,
        request: Request<MqttListConnectorRequest>,
    ) -> Result<Response<MqttListConnectorReply>, Status> {
        list_connector_by_req(&self.client_pool, request).await
    }

    async fn mqtt_broker_create_connector(
        &self,
        request: Request<MqttCreateConnectorRequest>,
    ) -> Result<Response<MqttCreateConnectorReply>, Status> {
        create_connector_by_req(&self.client_pool, request).await
    }

    async fn mqtt_broker_delete_connector(
        &self,
        request: Request<MqttDeleteConnectorRequest>,
    ) -> Result<Response<MqttDeleteConnectorReply>, Status> {
        delete_connector_by_req(&self.client_pool, request).await
    }

    async fn mqtt_broker_update_connector(
        &self,
        request: Request<MqttUpdateConnectorRequest>,
    ) -> Result<Response<MqttUpdateConnectorReply>, Status> {
        update_connector_by_req(&self.client_pool, request).await
    }

    // --- schema ---
    async fn mqtt_broker_list_schema(
        &self,
        request: Request<MqttListSchemaRequest>,
    ) -> Result<Response<MqttListSchemaReply>, Status> {
        list_schema_by_req(&self.client_pool, request).await
    }

    async fn mqtt_broker_create_schema(
        &self,
        request: Request<MqttCreateSchemaRequest>,
    ) -> Result<Response<MqttCreateSchemaReply>, Status> {
        create_schema_by_req(&self.client_pool, request).await
    }

    async fn mqtt_broker_update_schema(
        &self,
        request: Request<MqttUpdateSchemaRequest>,
    ) -> Result<Response<MqttUpdateSchemaReply>, Status> {
        update_schema_by_req(&self.client_pool, request).await
    }

    async fn mqtt_broker_delete_schema(
        &self,
        request: Request<MqttDeleteSchemaRequest>,
    ) -> Result<Response<MqttDeleteSchemaReply>, Status> {
        delete_schema_by_req(&self.client_pool, request).await
    }

    async fn mqtt_broker_list_bind_schema(
        &self,
        request: Request<MqttListBindSchemaRequest>,
    ) -> Result<Response<MqttListBindSchemaReply>, Status> {
        list_bind_schema_by_req(&self.client_pool, request).await
    }

    async fn mqtt_broker_bind_schema(
        &self,
        request: Request<MqttBindSchemaRequest>,
    ) -> Result<Response<MqttBindSchemaReply>, Status> {
        bind_schema_by_req(&self.client_pool, request).await
    }

    async fn mqtt_broker_unbind_schema(
        &self,
        request: Request<MqttUnbindSchemaRequest>,
    ) -> Result<Response<MqttUnbindSchemaReply>, Status> {
        unbind_schema_by_req(&self.client_pool, request).await
    }

    async fn mqtt_broker_set_auto_subscribe_rule(
        &self,
        request: Request<SetAutoSubscribeRuleRequest>,
    ) -> Result<Response<SetAutoSubscribeRuleReply>, Status> {
        set_auto_subscribe_rule(&self.client_pool, &self.cache_manager, request).await
    }

    async fn mqtt_broker_delete_auto_subscribe_rule(
        &self,
        request: Request<DeleteAutoSubscribeRuleRequest>,
    ) -> Result<Response<DeleteAutoSubscribeRuleReply>, Status> {
        delete_auto_subscribe_rule(&self.client_pool, &self.cache_manager, request).await
    }

    async fn mqtt_broker_list_auto_subscribe_rule(
        &self,
        _request: Request<ListAutoSubscribeRuleRequest>,
    ) -> Result<Response<ListAutoSubscribeRuleReply>, Status> {
        list_auto_subscribe_rule_by_req(&self.cache_manager)
    }
}
