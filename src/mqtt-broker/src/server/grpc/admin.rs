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

use crate::admin::acl::{create_acl_by_req, delete_acl_by_req, list_acl_by_req};
use crate::admin::blacklist::{
    create_blacklist_by_req, delete_blacklist_by_req, list_blacklist_by_req,
};
use crate::admin::client::list_client_by_req;
use crate::admin::cluster::{get_cluster_config_by_req, set_cluster_config_by_req};
use crate::admin::connector::{
    create_connector_by_req, delete_connector_by_req, list_connector_by_req,
    update_connector_by_req,
};
use crate::admin::observability::{
    list_slow_subscribe_by_req, list_system_alarm_by_req, set_slow_subscribe_config_by_req,
    set_system_alarm_config_by_req,
};
use crate::admin::schema::{
    bind_schema_by_req, create_schema_by_req, delete_schema_by_req, list_bind_schema_by_req,
    list_schema_by_req, unbind_schema_by_req, update_schema_by_req,
};
use crate::admin::session::list_session_by_req;
use crate::admin::subscribe::{
    delete_auto_subscribe_rule_by_req, list_auto_subscribe_rule_by_req, list_subscribe_by_req,
    set_auto_subscribe_rule_by_req, subscribe_detail_by_req,
};
use crate::admin::topic::{
    create_topic_rewrite_rule_by_req, delete_topic_rewrite_rule_by_req,
    get_all_topic_rewrite_rule_by_req, list_topic_by_req,
};
use crate::admin::user::{create_user_by_req, delete_user_by_req, list_user_by_req};
use crate::admin::{
    cluster_overview_metrics_by_req, cluster_status_by_req, enable_flapping_detect_by_req,
    list_connection_by_req, list_flapping_detect_by_req,
};
use crate::common::metrics_cache::MetricsCacheManager;
use crate::handler::cache::CacheManager;
use crate::server::common::connection_manager::ConnectionManager;
use crate::subscribe::manager::SubscribeManager;
use grpc_clients::pool::ClientPool;
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_server::MqttBrokerAdminService;
use protocol::broker_mqtt::broker_mqtt_admin::{
    BindSchemaReply, BindSchemaRequest, ClusterOverviewMetricsReply, ClusterOverviewMetricsRequest,
    ClusterStatusReply, ClusterStatusRequest, CreateAclReply, CreateAclRequest,
    CreateBlacklistReply, CreateBlacklistRequest, CreateConnectorReply, CreateConnectorRequest,
    CreateSchemaReply, CreateSchemaRequest, CreateTopicRewriteRuleReply,
    CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest, DeleteAclReply,
    DeleteAclRequest, DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest,
    DeleteBlacklistReply, DeleteBlacklistRequest, DeleteConnectorReply, DeleteConnectorRequest,
    DeleteSchemaReply, DeleteSchemaRequest, DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest, EnableFlappingDetectReply,
    EnableFlappingDetectRequest, GetClusterConfigReply, GetClusterConfigRequest, ListAclReply,
    ListAclRequest, ListAutoSubscribeRuleReply, ListAutoSubscribeRuleRequest, ListBindSchemaReply,
    ListBindSchemaRequest, ListBlacklistReply, ListBlacklistRequest, ListClientReply,
    ListClientRequest, ListConnectionReply, ListConnectionRequest, ListConnectorReply,
    ListConnectorRequest, ListFlappingDetectReply, ListFlappingDetectRequest,
    ListRewriteTopicRuleReply, ListRewriteTopicRuleRequest, ListSchemaReply, ListSchemaRequest,
    ListSessionReply, ListSessionRequest, ListSlowSubscribeReply, ListSlowSubscribeRequest,
    ListSubscribeReply, ListSubscribeRequest, ListSystemAlarmReply, ListSystemAlarmRequest,
    ListTopicReply, ListTopicRequest, ListUserReply, ListUserRequest, SetAutoSubscribeRuleReply,
    SetAutoSubscribeRuleRequest, SetClusterConfigReply, SetClusterConfigRequest,
    SetSlowSubscribeConfigReply, SetSlowSubscribeConfigRequest, SetSystemAlarmConfigReply,
    SetSystemAlarmConfigRequest, SubscribeDetailReply, SubscribeDetailRequest, UnbindSchemaReply,
    UnbindSchemaRequest, UpdateConnectorReply, UpdateConnectorRequest, UpdateSchemaReply,
    UpdateSchemaRequest,
};
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct GrpcAdminServices {
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
    connection_manager: Arc<ConnectionManager>,
    subscribe_manager: Arc<SubscribeManager>,
    metrics_cache_manager: Arc<MetricsCacheManager>,
}

impl GrpcAdminServices {
    pub fn new(
        client_pool: Arc<ClientPool>,
        cache_manager: Arc<CacheManager>,
        connection_manager: Arc<ConnectionManager>,
        subscribe_manager: Arc<SubscribeManager>,
        metrics_cache_manager: Arc<MetricsCacheManager>,
    ) -> Self {
        GrpcAdminServices {
            client_pool,
            cache_manager,
            connection_manager,
            subscribe_manager,
            metrics_cache_manager,
        }
    }
}

#[tonic::async_trait]
impl MqttBrokerAdminService for GrpcAdminServices {
    async fn set_cluster_config(
        &self,
        request: Request<SetClusterConfigRequest>,
    ) -> Result<Response<SetClusterConfigReply>, Status> {
        let request = request.into_inner().clone();
        set_cluster_config_by_req(&self.cache_manager, &self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn get_cluster_config(
        &self,
        _request: Request<GetClusterConfigRequest>,
    ) -> Result<Response<GetClusterConfigReply>, Status> {
        get_cluster_config_by_req(&self.cache_manager)
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    // --- cluster ---
    async fn cluster_status(
        &self,
        _: Request<ClusterStatusRequest>,
    ) -> Result<Response<ClusterStatusReply>, Status> {
        cluster_status_by_req(
            &self.client_pool,
            &self.subscribe_manager,
            &self.connection_manager,
            &self.cache_manager,
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }

    async fn cluster_overview_metrics(
        &self,
        request: Request<ClusterOverviewMetricsRequest>,
    ) -> Result<Response<ClusterOverviewMetricsReply>, Status> {
        let request = request.into_inner();
        cluster_overview_metrics_by_req(&self.metrics_cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    // --- user ---

    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<CreateUserReply>, Status> {
        let request = request.into_inner();
        create_user_by_req(&self.cache_manager, &self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserReply>, Status> {
        let request = request.into_inner();
        delete_user_by_req(&self.cache_manager, &self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_user(
        &self,
        request: Request<ListUserRequest>,
    ) -> Result<Response<ListUserReply>, Status> {
        let request = request.into_inner();
        list_user_by_req(&self.cache_manager, &self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_client(
        &self,
        request: Request<ListClientRequest>,
    ) -> Result<Response<ListClientReply>, Status> {
        let request = request.into_inner();
        list_client_by_req(&self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_session(
        &self,
        request: Request<ListSessionRequest>,
    ) -> Result<Response<ListSessionReply>, Status> {
        let request = request.into_inner();
        list_session_by_req(&self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_acl(
        &self,
        _request: Request<ListAclRequest>,
    ) -> Result<Response<ListAclReply>, Status> {
        list_acl_by_req(&self.cache_manager, &self.client_pool)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn create_acl(
        &self,
        request: Request<CreateAclRequest>,
    ) -> Result<Response<CreateAclReply>, Status> {
        let request = request.into_inner();
        create_acl_by_req(&self.cache_manager, &self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_acl(
        &self,
        request: Request<DeleteAclRequest>,
    ) -> Result<Response<DeleteAclReply>, Status> {
        let request = request.into_inner();
        delete_acl_by_req(&self.cache_manager, &self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_blacklist(
        &self,
        request: Request<ListBlacklistRequest>,
    ) -> Result<Response<ListBlacklistReply>, Status> {
        let request = request.into_inner();
        list_blacklist_by_req(&self.cache_manager, &self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_blacklist(
        &self,
        request: Request<DeleteBlacklistRequest>,
    ) -> Result<Response<DeleteBlacklistReply>, Status> {
        let request = request.into_inner();
        delete_blacklist_by_req(&self.cache_manager, &self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn create_blacklist(
        &self,
        request: Request<CreateBlacklistRequest>,
    ) -> Result<Response<CreateBlacklistReply>, Status> {
        let request = request.into_inner();
        create_blacklist_by_req(&self.cache_manager, &self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn enable_flapping_detect(
        &self,
        request: Request<EnableFlappingDetectRequest>,
    ) -> Result<Response<EnableFlappingDetectReply>, Status> {
        let request = request.into_inner();
        enable_flapping_detect_by_req(&self.client_pool, &self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_flapping_detect(
        &self,
        request: Request<ListFlappingDetectRequest>,
    ) -> Result<Response<ListFlappingDetectReply>, Status> {
        let request = request.into_inner();
        list_flapping_detect_by_req(&self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn set_system_alarm_config(
        &self,
        request: Request<SetSystemAlarmConfigRequest>,
    ) -> Result<Response<SetSystemAlarmConfigReply>, Status> {
        let request = request.into_inner();
        set_system_alarm_config_by_req(&self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_system_alarm(
        &self,
        request: Request<ListSystemAlarmRequest>,
    ) -> Result<Response<ListSystemAlarmReply>, Status> {
        let request = request.into_inner();
        list_system_alarm_by_req(&self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    // --- connection ---

    async fn list_connection(
        &self,
        _request: Request<ListConnectionRequest>,
    ) -> Result<Response<ListConnectionReply>, Status> {
        list_connection_by_req(&self.connection_manager, &self.cache_manager)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn set_slow_subscribe_config(
        &self,
        request: Request<SetSlowSubscribeConfigRequest>,
    ) -> Result<Response<SetSlowSubscribeConfigReply>, Status> {
        let request = request.into_inner();
        set_slow_subscribe_config_by_req(&self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_slow_subscribe(
        &self,
        request: Request<ListSlowSubscribeRequest>,
    ) -> Result<Response<ListSlowSubscribeReply>, Status> {
        let request = request.into_inner();
        list_slow_subscribe_by_req(&self.cache_manager, &self.metrics_cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_topic(
        &self,
        request: Request<ListTopicRequest>,
    ) -> Result<Response<ListTopicReply>, Status> {
        let request = request.into_inner();
        list_topic_by_req(&self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_topic_rewrite_rule(
        &self,
        request: Request<DeleteTopicRewriteRuleRequest>,
    ) -> Result<Response<DeleteTopicRewriteRuleReply>, Status> {
        let request = request.into_inner();
        delete_topic_rewrite_rule_by_req(&self.client_pool, &self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn create_topic_rewrite_rule(
        &self,
        request: Request<CreateTopicRewriteRuleRequest>,
    ) -> Result<Response<CreateTopicRewriteRuleReply>, Status> {
        let request = request.into_inner();
        create_topic_rewrite_rule_by_req(&self.client_pool, &self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn get_all_topic_rewrite_rule(
        &self,
        _request: Request<ListRewriteTopicRuleRequest>,
    ) -> Result<Response<ListRewriteTopicRuleReply>, Status> {
        get_all_topic_rewrite_rule_by_req(&self.cache_manager)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_connector(
        &self,
        request: Request<ListConnectorRequest>,
    ) -> Result<Response<ListConnectorReply>, Status> {
        let request = request.into_inner();
        list_connector_by_req(&self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn create_connector(
        &self,
        request: Request<CreateConnectorRequest>,
    ) -> Result<Response<CreateConnectorReply>, Status> {
        let request = request.into_inner();
        create_connector_by_req(&self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_connector(
        &self,
        request: Request<DeleteConnectorRequest>,
    ) -> Result<Response<DeleteConnectorReply>, Status> {
        let request = request.into_inner();
        delete_connector_by_req(&self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn update_connector(
        &self,
        request: Request<UpdateConnectorRequest>,
    ) -> Result<Response<UpdateConnectorReply>, Status> {
        let request = request.into_inner();
        update_connector_by_req(&self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    // --- schema ---

    async fn list_schema(
        &self,
        request: Request<ListSchemaRequest>,
    ) -> Result<Response<ListSchemaReply>, Status> {
        let request = request.into_inner();
        list_schema_by_req(&self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn create_schema(
        &self,
        request: Request<CreateSchemaRequest>,
    ) -> Result<Response<CreateSchemaReply>, Status> {
        let request = request.into_inner();
        create_schema_by_req(&self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn update_schema(
        &self,
        request: Request<UpdateSchemaRequest>,
    ) -> Result<Response<UpdateSchemaReply>, Status> {
        let request = request.into_inner();
        update_schema_by_req(&self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_schema(
        &self,
        request: Request<DeleteSchemaRequest>,
    ) -> Result<Response<DeleteSchemaReply>, Status> {
        let request = request.into_inner();
        delete_schema_by_req(&self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_bind_schema(
        &self,
        request: Request<ListBindSchemaRequest>,
    ) -> Result<Response<ListBindSchemaReply>, Status> {
        let request = request.into_inner();
        list_bind_schema_by_req(&self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn bind_schema(
        &self,
        request: Request<BindSchemaRequest>,
    ) -> Result<Response<BindSchemaReply>, Status> {
        let request = request.into_inner();
        bind_schema_by_req(&self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn unbind_schema(
        &self,
        request: Request<UnbindSchemaRequest>,
    ) -> Result<Response<UnbindSchemaReply>, Status> {
        let request = request.into_inner();
        unbind_schema_by_req(&self.client_pool, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn set_auto_subscribe_rule(
        &self,
        request: Request<SetAutoSubscribeRuleRequest>,
    ) -> Result<Response<SetAutoSubscribeRuleReply>, Status> {
        let request = request.into_inner();
        set_auto_subscribe_rule_by_req(&self.client_pool, &self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn delete_auto_subscribe_rule(
        &self,
        request: Request<DeleteAutoSubscribeRuleRequest>,
    ) -> Result<Response<DeleteAutoSubscribeRuleReply>, Status> {
        let request = request.into_inner();
        delete_auto_subscribe_rule_by_req(&self.client_pool, &self.cache_manager, &request)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_auto_subscribe_rule(
        &self,
        _request: Request<ListAutoSubscribeRuleRequest>,
    ) -> Result<Response<ListAutoSubscribeRuleReply>, Status> {
        list_auto_subscribe_rule_by_req(&self.cache_manager)
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn list_subscribe(
        &self,
        request: Request<ListSubscribeRequest>,
    ) -> Result<Response<ListSubscribeReply>, Status> {
        list_subscribe_by_req(&self.subscribe_manager, request.into_inner())
            .await
            .map_err(|e| Status::internal(e.to_string()))
            .map(Response::new)
    }

    async fn get_subscribe_detail(
        &self,
        request: Request<SubscribeDetailRequest>,
    ) -> Result<Response<SubscribeDetailReply>, Status> {
        subscribe_detail_by_req(
            &self.subscribe_manager,
            &self.client_pool,
            request.into_inner(),
        )
        .await
        .map_err(|e| Status::internal(e.to_string()))
        .map(Response::new)
    }
}
