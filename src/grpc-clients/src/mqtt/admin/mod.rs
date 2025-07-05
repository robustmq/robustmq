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
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_client::MqttBrokerAdminServiceClient;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterOverviewMetricsReply, ClusterOverviewMetricsRequest, ClusterStatusReply,
    ClusterStatusRequest, DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest,
    GetClusterConfigReply, GetClusterConfigRequest, ListAutoSubscribeRuleReply,
    ListAutoSubscribeRuleRequest, ListSessionReply, ListSessionRequest, ListSubscribeReply,
    ListSubscribeRequest, ListSystemAlarmReply, ListSystemAlarmRequest, MqttCreateConnectorReply,
    MqttCreateConnectorRequest, MqttDeleteConnectorReply, MqttDeleteConnectorRequest,
    MqttListConnectorReply, MqttListConnectorRequest, MqttUpdateConnectorReply,
    MqttUpdateConnectorRequest, SetAutoSubscribeRuleReply, SetAutoSubscribeRuleRequest,
    SetClusterConfigReply, SetClusterConfigRequest, SetSystemAlarmConfigReply,
    SetSystemAlarmConfigRequest, SubscribeDetailReply, SubscribeDetailRequest,
};
use protocol::broker_mqtt::broker_mqtt_admin::{
    CreateAclReply, CreateAclRequest, CreateBlacklistReply, CreateBlacklistRequest,
    CreateTopicRewriteRuleReply, CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest,
    DeleteAclReply, DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest,
    DeleteTopicRewriteRuleReply, DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest,
    EnableFlappingDetectReply, EnableFlappingDetectRequest, ListAclReply, ListAclRequest,
    ListBlacklistReply, ListBlacklistRequest, ListConnectionReply, ListConnectionRequest,
    ListSlowSubscribeReply, ListSlowSubscribeRequest, ListTopicReply, ListTopicRequest,
    ListUserReply, ListUserRequest, MqttBindSchemaReply, MqttBindSchemaRequest,
    MqttCreateSchemaReply, MqttCreateSchemaRequest, MqttDeleteSchemaReply, MqttDeleteSchemaRequest,
    MqttListBindSchemaReply, MqttListBindSchemaRequest, MqttListSchemaReply, MqttListSchemaRequest,
    MqttUnbindSchemaReply, MqttUnbindSchemaRequest, MqttUpdateSchemaReply, MqttUpdateSchemaRequest,
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
    mqtt_broker_set_cluster_config
);

impl_retriable_request!(
    GetClusterConfigRequest,
    MqttBrokerAdminServiceClient<Channel>,
    GetClusterConfigReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_get_cluster_config
);

impl_retriable_request!(
    ClusterStatusRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ClusterStatusReply,
    mqtt_broker_admin_services_client,
    cluster_status
);

impl_retriable_request!(
    ListUserRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListUserReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_user
);

impl_retriable_request!(
    CreateUserRequest,
    MqttBrokerAdminServiceClient<Channel>,
    CreateUserReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_create_user
);

impl_retriable_request!(
    DeleteUserRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteUserReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_delete_user
);

impl_retriable_request!(
    ListAclRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListAclReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_acl
);

impl_retriable_request!(
    CreateAclRequest,
    MqttBrokerAdminServiceClient<Channel>,
    CreateAclReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_create_acl
);

impl_retriable_request!(
    DeleteAclRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteAclReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_delete_acl
);

impl_retriable_request!(
    ListBlacklistRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListBlacklistReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_blacklist
);

impl_retriable_request!(
    CreateBlacklistRequest,
    MqttBrokerAdminServiceClient<Channel>,
    CreateBlacklistReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_create_blacklist
);

impl_retriable_request!(
    DeleteBlacklistRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteBlacklistReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_delete_blacklist
);

impl_retriable_request!(
    ListConnectionRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListConnectionReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_connection
);

impl_retriable_request!(
    EnableFlappingDetectRequest,
    MqttBrokerAdminServiceClient<Channel>,
    EnableFlappingDetectReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_enable_flapping_detect
);

// #### observability ####

// ---- slow subscribe ----
impl_retriable_request!(
    ListSlowSubscribeRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListSlowSubscribeReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_slow_subscribe
);

// ---- system alarm ----
impl_retriable_request!(
    SetSystemAlarmConfigRequest,
    MqttBrokerAdminServiceClient<Channel>,
    SetSystemAlarmConfigReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_set_system_alarm_config
);

impl_retriable_request!(
    ListSystemAlarmRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListSystemAlarmReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_system_alarm
);

impl_retriable_request!(
    ListTopicRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListTopicReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_topic
);

impl_retriable_request!(
    CreateTopicRewriteRuleRequest,
    MqttBrokerAdminServiceClient<Channel>,
    CreateTopicRewriteRuleReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_create_topic_rewrite_rule
);

impl_retriable_request!(
    DeleteTopicRewriteRuleRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteTopicRewriteRuleReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_delete_topic_rewrite_rule
);

// connector command line CRUD
impl_retriable_request!(
    MqttListConnectorRequest,
    MqttBrokerAdminServiceClient<Channel>,
    MqttListConnectorReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_connector
);

impl_retriable_request!(
    MqttCreateConnectorRequest,
    MqttBrokerAdminServiceClient<Channel>,
    MqttCreateConnectorReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_create_connector
);

impl_retriable_request!(
    MqttUpdateConnectorRequest,
    MqttBrokerAdminServiceClient<Channel>,
    MqttUpdateConnectorReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_update_connector
);

impl_retriable_request!(
    MqttDeleteConnectorRequest,
    MqttBrokerAdminServiceClient<Channel>,
    MqttDeleteConnectorReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_delete_connector
);

// schema command line CRUD
impl_retriable_request!(
    MqttListSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    MqttListSchemaReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_schema
);

impl_retriable_request!(
    MqttCreateSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    MqttCreateSchemaReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_create_schema
);

impl_retriable_request!(
    MqttUpdateSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    MqttUpdateSchemaReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_update_schema
);

impl_retriable_request!(
    MqttDeleteSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    MqttDeleteSchemaReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_delete_schema
);

impl_retriable_request!(
    MqttListBindSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    MqttListBindSchemaReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_bind_schema
);

impl_retriable_request!(
    MqttBindSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    MqttBindSchemaReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_bind_schema
);

impl_retriable_request!(
    MqttUnbindSchemaRequest,
    MqttBrokerAdminServiceClient<Channel>,
    MqttUnbindSchemaReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_unbind_schema
);

impl_retriable_request!(
    ListAutoSubscribeRuleRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListAutoSubscribeRuleReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_auto_subscribe_rule
);

impl_retriable_request!(
    SetAutoSubscribeRuleRequest,
    MqttBrokerAdminServiceClient<Channel>,
    SetAutoSubscribeRuleReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_set_auto_subscribe_rule
);

impl_retriable_request!(
    DeleteAutoSubscribeRuleRequest,
    MqttBrokerAdminServiceClient<Channel>,
    DeleteAutoSubscribeRuleReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_delete_auto_subscribe_rule
);

impl_retriable_request!(
    ListSessionRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListSessionReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_session
);

impl_retriable_request!(
    ClusterOverviewMetricsRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ClusterOverviewMetricsReply,
    mqtt_broker_admin_services_client,
    cluster_overview_metrics
);

impl_retriable_request!(
    ListSubscribeRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListSubscribeReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_subscribe
);

impl_retriable_request!(
    SubscribeDetailRequest,
    MqttBrokerAdminServiceClient<Channel>,
    SubscribeDetailReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_subscribe_detail
);
