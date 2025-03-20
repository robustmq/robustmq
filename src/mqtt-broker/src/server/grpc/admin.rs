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

use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
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
    ListBlacklistReply, ListBlacklistRequest, ListConnectionRaw, ListConnectionReply,
    ListConnectionRequest, ListSlowSubScribeRaw, ListSlowSubscribeReply, ListSlowSubscribeRequest,
    ListTopicReply, ListTopicRequest, ListUserReply, ListUserRequest, MqttBindSchemaReply,
    MqttBindSchemaRequest, MqttCreateConnectorReply, MqttCreateConnectorRequest,
    MqttCreateSchemaReply, MqttCreateSchemaRequest, MqttDeleteConnectorReply,
    MqttDeleteConnectorRequest, MqttDeleteSchemaReply, MqttDeleteSchemaRequest,
    MqttListBindSchemaReply, MqttListBindSchemaRequest, MqttListConnectorReply,
    MqttListConnectorRequest, MqttListSchemaReply, MqttListSchemaRequest, MqttTopic,
    MqttUnbindSchemaReply, MqttUnbindSchemaRequest, MqttUpdateConnectorReply,
    MqttUpdateConnectorRequest, MqttUpdateSchemaReply, MqttUpdateSchemaRequest,
    SetAutoSubscribeRuleReply, SetAutoSubscribeRuleRequest,
};
use protocol::mqtt::common::{qos, Error, QoS, RetainForwardRule};
use tonic::{Request, Response, Status};

use crate::admin::{
    cluster_status_by_req, create_acl_by_req, create_blacklist_by_req,
    create_topic_rewrite_rule_by_req, create_user_by_req, delete_acl_by_req,
    delete_blacklist_by_req, delete_topic_rewrite_rule_by_req, delete_user_by_req,
    enable_flapping_detect_by_req, enable_slow_subscribe_by_req, list_acl_by_req,
    list_blacklist_by_req, list_connection_by_req, list_slow_subscribe_by_req, list_topic_by_req,
    list_user_by_req,
};
use crate::bridge::request::{
    create_connector_by_req, delete_connector_by_req, list_connector_by_req,
    update_connector_by_req,
};
use crate::handler::cache::CacheManager;
use crate::server::connection_manager::ConnectionManager;
use crate::storage::auto_subscribe::AutoSubscribeStorage;
use crate::storage::schema::{
    bind_schema_by_req, create_schema_by_req, delete_schema_by_req, list_bind_schema_by_req,
    list_schema_by_req, unbind_schema_by_req, update_schema_by_req,
};

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
        let req = request.into_inner();
        match list_connector_by_req(&self.client_pool, &req).await {
            Ok(data) => Ok(Response::new(MqttListConnectorReply { connectors: data })),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_create_connector(
        &self,
        request: Request<MqttCreateConnectorRequest>,
    ) -> Result<Response<MqttCreateConnectorReply>, Status> {
        let req = request.into_inner();
        match create_connector_by_req(&self.client_pool, &req).await {
            Ok(_) => Ok(Response::new(MqttCreateConnectorReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_delete_connector(
        &self,
        request: Request<MqttDeleteConnectorRequest>,
    ) -> Result<Response<MqttDeleteConnectorReply>, Status> {
        let req = request.into_inner();
        match delete_connector_by_req(&self.client_pool, &req).await {
            Ok(_) => Ok(Response::new(MqttDeleteConnectorReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_update_connector(
        &self,
        request: Request<MqttUpdateConnectorRequest>,
    ) -> Result<Response<MqttUpdateConnectorReply>, Status> {
        let req = request.into_inner();
        match update_connector_by_req(&self.client_pool, &req).await {
            Ok(_) => Ok(Response::new(MqttUpdateConnectorReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    // --- schema ---
    async fn mqtt_broker_list_schema(
        &self,
        request: Request<MqttListSchemaRequest>,
    ) -> Result<Response<MqttListSchemaReply>, Status> {
        let req = request.into_inner();
        match list_schema_by_req(&self.client_pool, &req).await {
            Ok(data) => Ok(Response::new(MqttListSchemaReply { schemas: data })),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_create_schema(
        &self,
        request: Request<MqttCreateSchemaRequest>,
    ) -> Result<Response<MqttCreateSchemaReply>, Status> {
        let req = request.into_inner();
        match create_schema_by_req(&self.client_pool, &req).await {
            Ok(_) => Ok(Response::new(MqttCreateSchemaReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_update_schema(
        &self,
        request: Request<MqttUpdateSchemaRequest>,
    ) -> Result<Response<MqttUpdateSchemaReply>, Status> {
        let req = request.into_inner();
        match update_schema_by_req(&self.client_pool, &req).await {
            Ok(_) => Ok(Response::new(MqttUpdateSchemaReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_delete_schema(
        &self,
        request: Request<MqttDeleteSchemaRequest>,
    ) -> Result<Response<MqttDeleteSchemaReply>, Status> {
        let req = request.into_inner();
        match delete_schema_by_req(&self.client_pool, &req).await {
            Ok(_) => Ok(Response::new(MqttDeleteSchemaReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_list_bind_schema(
        &self,
        request: Request<MqttListBindSchemaRequest>,
    ) -> Result<Response<MqttListBindSchemaReply>, Status> {
        let req = request.into_inner();

        match list_bind_schema_by_req(&self.client_pool, &req).await {
            Ok(data) => Ok(Response::new(MqttListBindSchemaReply {
                schema_binds: data,
            })),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_bind_schema(
        &self,
        request: Request<MqttBindSchemaRequest>,
    ) -> Result<Response<MqttBindSchemaReply>, Status> {
        let req = request.into_inner();

        match bind_schema_by_req(&self.client_pool, &req).await {
            Ok(_) => Ok(Response::new(MqttBindSchemaReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_unbind_schema(
        &self,
        request: Request<MqttUnbindSchemaRequest>,
    ) -> Result<Response<MqttUnbindSchemaReply>, Status> {
        let req = request.into_inner();
        match unbind_schema_by_req(&self.client_pool, &req).await {
            Ok(_) => Ok(Response::new(MqttUnbindSchemaReply::default())),
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_delete_auto_subscribe_rule(
        &self,
        request: Request<DeleteAutoSubscribeRuleRequest>,
    ) -> Result<Response<DeleteAutoSubscribeRuleReply>, Status> {
        let req = request.into_inner();
        let auto_subscribe_storage = AutoSubscribeStorage::new(self.client_pool.clone());
        match auto_subscribe_storage
            .delete_auto_subscribe_rule(req.topic.clone())
            .await
        {
            Ok(_) => {
                let config = broker_mqtt_conf();
                let key = self
                    .cache_manager
                    .auto_subscribe_rule_key(&config.cluster_name, &req.topic);
                self.cache_manager.auto_subscribe_rule.remove(&key);
                Ok(Response::new(DeleteAutoSubscribeRuleReply::default()))
            }
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_set_auto_subscribe_rule(
        &self,
        request: Request<SetAutoSubscribeRuleRequest>,
    ) -> Result<Response<SetAutoSubscribeRuleReply>, Status> {
        let req = request.into_inner();
        let config = broker_mqtt_conf();

        let mut _qos: Option<QoS> = None;
        if req.qos <= u8::MAX as u32 {
            _qos = qos(req.qos.clone() as u8);
        } else {
            return Err(Status::cancelled(
                Error::InvalidRemainingLength(req.qos.clone() as usize).to_string(),
            ));
        };
        let mut _retained_handling: Option<RetainForwardRule> = None;
        if req.retained_handling <= u8::MAX as u32 {
            _retained_handling =
                RetainForwardRule::retain_forward_rule(req.retained_handling.clone() as u8);
        } else {
            return Err(Status::cancelled(
                Error::InvalidRemainingLength(req.retained_handling.clone() as usize).to_string(),
            ));
        };

        let auto_subscribe_rule = MqttAutoSubscribeRule {
            cluster: config.cluster_name.clone(),
            topic: req.topic.clone(),
            qos: _qos.ok_or_else(|| {
                Status::cancelled(Error::InvalidQoS(req.qos.clone() as u8).to_string())
            })?,
            no_local: req.no_local.clone(),
            retain_as_published: req.retain_as_published.clone(),
            retained_handling: _retained_handling.ok_or_else(|| {
                Status::cancelled(
                    Error::InvalidQoS(req.retained_handling.clone() as u8).to_string(),
                )
            })?,
        };
        let auto_subscribe_storage = AutoSubscribeStorage::new(self.client_pool.clone());
        match auto_subscribe_storage
            .set_auto_subscribe_rule(auto_subscribe_rule.clone())
            .await
        {
            Ok(_) => {
                let key = self
                    .cache_manager
                    .auto_subscribe_rule_key(&config.cluster_name, &req.topic);
                self.cache_manager
                    .auto_subscribe_rule
                    .insert(key, auto_subscribe_rule);
                Ok(Response::new(SetAutoSubscribeRuleReply::default()))
            }
            Err(e) => Err(Status::cancelled(e.to_string())),
        }
    }

    async fn mqtt_broker_list_auto_subscribe_rule(
        &self,
        _request: Request<ListAutoSubscribeRuleRequest>,
    ) -> Result<Response<ListAutoSubscribeRuleReply>, Status> {
        return Ok(Response::new(ListAutoSubscribeRuleReply {
            auto_subscribe_rules: self
                .cache_manager
                .auto_subscribe_rule
                .iter()
                .map(|entry| entry.value().clone().encode())
                .collect(),
        }));
    }
}
