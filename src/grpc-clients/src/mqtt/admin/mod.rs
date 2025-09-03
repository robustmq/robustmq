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

use common_base::error::common::CommonError;
use mobc::Manager;
use protocol::broker::broker_mqtt_admin::mqtt_broker_admin_service_client::MqttBrokerAdminServiceClient;
use protocol::broker::broker_mqtt_admin::{
    BindSchemaReply, BindSchemaRequest, CreateAclReply, CreateAclRequest, CreateBlacklistReply,
    CreateBlacklistRequest, CreateConnectorReply, CreateConnectorRequest, CreateSchemaReply,
    CreateSchemaRequest, CreateTopicRewriteRuleReply, CreateTopicRewriteRuleRequest,
    CreateUserReply, CreateUserRequest, DeleteAclReply, DeleteAclRequest,
    DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest, DeleteBlacklistReply,
    DeleteBlacklistRequest, DeleteConnectorReply, DeleteConnectorRequest, DeleteSchemaReply,
    DeleteSchemaRequest, DeleteTopicRewriteRuleReply, DeleteTopicRewriteRuleRequest,
    DeleteUserReply, DeleteUserRequest, EnableFlappingDetectReply, EnableFlappingDetectRequest,
    GetClusterConfigReply, GetClusterConfigRequest, ListAclReply, ListAclRequest,
    ListAutoSubscribeRuleReply, ListAutoSubscribeRuleRequest, ListBindSchemaReply,
    ListBindSchemaRequest, ListBlacklistReply, ListBlacklistRequest, ListConnectionReply,
    ListConnectionRequest, ListConnectorReply, ListConnectorRequest, ListFlappingDetectReply,
    ListFlappingDetectRequest, ListSchemaReply, ListSchemaRequest, ListSessionReply,
    ListSessionRequest, ListSlowSubscribeReply, ListSlowSubscribeRequest, ListSubscribeReply,
    ListSubscribeRequest, ListSystemAlarmReply, ListSystemAlarmRequest, ListTopicReply,
    ListTopicRequest, ListUserReply, ListUserRequest, SetAutoSubscribeRuleReply,
    SetAutoSubscribeRuleRequest, SetClusterConfigReply, SetClusterConfigRequest,
    SetSlowSubscribeConfigReply, SetSlowSubscribeConfigRequest, SetSystemAlarmConfigReply,
    SetSystemAlarmConfigRequest, SubscribeDetailReply, SubscribeDetailRequest, UnbindSchemaReply,
    UnbindSchemaRequest, UpdateConnectorReply, UpdateConnectorRequest, UpdateSchemaReply,
    UpdateSchemaRequest,
};
use tonic::transport::Channel;

use crate::macros::impl_retriable_request;

pub mod call;

#[derive(Clone)]
pub struct MqttBrokerAdminServiceManager {
    pub addr: String,
}

impl MqttBrokerAdminServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for MqttBrokerAdminServiceManager {
    type Connection = MqttBrokerAdminServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match MqttBrokerAdminServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => Ok(client),
            Err(err) => Err(CommonError::CommonError(format!(
                "{},{}",
                err,
                self.addr.clone()
            ))),
        }
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

impl_retriable_request!(
    SetClusterConfigRequest,
    MqttBrokerAdminServiceClient<Channel>,
    SetClusterConfigReply,
    mqtt_broker_admin_services_client,
    set_cluster_config
);

impl_retriable_request!(
    GetClusterConfigRequest,
    MqttBrokerAdminServiceClient<Channel>,
    GetClusterConfigReply,
    mqtt_broker_admin_services_client,
    get_cluster_config
);

impl_retriable_request!(
    ListUserRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListUserReply,
    mqtt_broker_admin_services_client,
    list_user
);

impl_retriable_request!(
    CreateUserRequest,
    MqttBrokerAdminServiceClient<Channel>,
    CreateUserReply,
    mqtt_broker_admin_services_client,
    create_user
);

impl_retriable_request!(
    DeleteUserRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteUserReply,
    mqtt_broker_admin_services_client,
    delete_user
);

impl_retriable_request!(
    ListAclRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListAclReply,
    mqtt_broker_admin_services_client,
    list_acl
);

impl_retriable_request!(
    CreateAclRequest,
    MqttBrokerAdminServiceClient<Channel>,
    CreateAclReply,
    mqtt_broker_admin_services_client,
    create_acl
);

impl_retriable_request!(
    DeleteAclRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteAclReply,
    mqtt_broker_admin_services_client,
    delete_acl
);

impl_retriable_request!(
    ListBlacklistRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListBlacklistReply,
    mqtt_broker_admin_services_client,
    list_blacklist
);

impl_retriable_request!(
    CreateBlacklistRequest,
    MqttBrokerAdminServiceClient<Channel>,
    CreateBlacklistReply,
    mqtt_broker_admin_services_client,
    create_blacklist
);

impl_retriable_request!(
    DeleteBlacklistRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteBlacklistReply,
    mqtt_broker_admin_services_client,
    delete_blacklist
);

impl_retriable_request!(
    ListConnectionRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListConnectionReply,
    mqtt_broker_admin_services_client,
    list_connection
);

impl_retriable_request!(
    EnableFlappingDetectRequest,
    MqttBrokerAdminServiceClient<Channel>,
    EnableFlappingDetectReply,
    mqtt_broker_admin_services_client,
    enable_flapping_detect
);

impl_retriable_request!(
    ListFlappingDetectRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListFlappingDetectReply,
    mqtt_broker_admin_services_client,
    list_flapping_detect
);

// #### observability ####

// ---- slow subscribe ----
impl_retriable_request!(
    SetSlowSubscribeConfigRequest,
    MqttBrokerAdminServiceClient<Channel>,
    SetSlowSubscribeConfigReply,
    mqtt_broker_admin_services_client,
    set_slow_subscribe_config
);

impl_retriable_request!(
    ListSlowSubscribeRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListSlowSubscribeReply,
    mqtt_broker_admin_services_client,
    list_slow_subscribe
);

// ---- system alarm ----
impl_retriable_request!(
    SetSystemAlarmConfigRequest,
    MqttBrokerAdminServiceClient<Channel>,
    SetSystemAlarmConfigReply,
    mqtt_broker_admin_services_client,
    set_system_alarm_config
);

impl_retriable_request!(
    ListSystemAlarmRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListSystemAlarmReply,
    mqtt_broker_admin_services_client,
    list_system_alarm
);

impl_retriable_request!(
    ListTopicRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListTopicReply,
    mqtt_broker_admin_services_client,
    list_topic
);

impl_retriable_request!(
    CreateTopicRewriteRuleRequest,
    MqttBrokerAdminServiceClient<Channel>,
    CreateTopicRewriteRuleReply,
    mqtt_broker_admin_services_client,
    create_topic_rewrite_rule
);

impl_retriable_request!(
    DeleteTopicRewriteRuleRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteTopicRewriteRuleReply,
    mqtt_broker_admin_services_client,
    delete_topic_rewrite_rule
);

// connector command line CRUD
impl_retriable_request!(
    ListConnectorRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListConnectorReply,
    mqtt_broker_admin_services_client,
    list_connector
);

impl_retriable_request!(
    CreateConnectorRequest,
    MqttBrokerAdminServiceClient<Channel>,
    CreateConnectorReply,
    mqtt_broker_admin_services_client,
    create_connector
);

impl_retriable_request!(
    UpdateConnectorRequest,
    MqttBrokerAdminServiceClient<Channel>,
    UpdateConnectorReply,
    mqtt_broker_admin_services_client,
    update_connector
);

impl_retriable_request!(
    DeleteConnectorRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteConnectorReply,
    mqtt_broker_admin_services_client,
    delete_connector
);

// schema command line CRUD
impl_retriable_request!(
    ListSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListSchemaReply,
    mqtt_broker_admin_services_client,
    list_schema
);

impl_retriable_request!(
    CreateSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    CreateSchemaReply,
    mqtt_broker_admin_services_client,
    create_schema
);

impl_retriable_request!(
    UpdateSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    UpdateSchemaReply,
    mqtt_broker_admin_services_client,
    update_schema
);

impl_retriable_request!(
    DeleteSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteSchemaReply,
    mqtt_broker_admin_services_client,
    delete_schema
);

impl_retriable_request!(
    ListBindSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListBindSchemaReply,
    mqtt_broker_admin_services_client,
    list_bind_schema
);

impl_retriable_request!(
    BindSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    BindSchemaReply,
    mqtt_broker_admin_services_client,
    bind_schema
);

impl_retriable_request!(
    UnbindSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    UnbindSchemaReply,
    mqtt_broker_admin_services_client,
    unbind_schema
);

impl_retriable_request!(
    ListAutoSubscribeRuleRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListAutoSubscribeRuleReply,
    mqtt_broker_admin_services_client,
    list_auto_subscribe_rule
);

impl_retriable_request!(
    SetAutoSubscribeRuleRequest,
    MqttBrokerAdminServiceClient<Channel>,
    SetAutoSubscribeRuleReply,
    mqtt_broker_admin_services_client,
    set_auto_subscribe_rule
);

impl_retriable_request!(
    DeleteAutoSubscribeRuleRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteAutoSubscribeRuleReply,
    mqtt_broker_admin_services_client,
    delete_auto_subscribe_rule
);

impl_retriable_request!(
    ListSessionRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListSessionReply,
    mqtt_broker_admin_services_client,
    list_session
);

impl_retriable_request!(
    ListSubscribeRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListSubscribeReply,
    mqtt_broker_admin_services_client,
    list_subscribe
);

impl_retriable_request!(
    SubscribeDetailRequest,
    MqttBrokerAdminServiceClient<Channel>,
    SubscribeDetailReply,
    mqtt_broker_admin_services_client,
    get_subscribe_detail
);
