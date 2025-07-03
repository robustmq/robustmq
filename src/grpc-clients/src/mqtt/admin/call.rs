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
    ListBindSchemaRequest, ListBlacklistReply, ListBlacklistRequest, ListConnectionReply,
    ListConnectionRequest, ListConnectorReply, ListConnectorRequest, ListFlappingDetectReply,
    ListFlappingDetectRequest, ListSchemaReply, ListSchemaRequest, ListSessionReply,
    ListSessionRequest, ListSlowSubscribeReply, ListSlowSubscribeRequest, ListSubscribeReply,
    ListSubscribeRequest, ListSystemAlarmReply, ListSystemAlarmRequest, ListTopicReply,
    ListTopicRequest, ListUserReply, ListUserRequest, SetAutoSubscribeRuleReply,
    SetAutoSubscribeRuleRequest, SetClusterConfigReply, SetClusterConfigRequest,
    SetSystemAlarmConfigReply, SetSystemAlarmConfigRequest, SubscribeDetailReply,
    SubscribeDetailRequest, UnbindSchemaReply, UnbindSchemaRequest, UpdateConnectorReply,
    UpdateConnectorRequest, UpdateSchemaReply, UpdateSchemaRequest,
};
use tonic::Streaming;

use crate::pool::ClientPool;

macro_rules! generate_mqtt_admin_service_call {
    ($fn_name:ident, $req_ty:ty, $rep_ty:ty, $variant:ident) => {
        pub async fn $fn_name(
            client_pool: &ClientPool,
            addrs: &[impl AsRef<str>],
            request: $req_ty,
        ) -> Result<$rep_ty, CommonError> {
            $crate::utils::retry_call(client_pool, addrs, request).await
        }
    };
}

// --- common ------
generate_mqtt_admin_service_call!(
    mqtt_broker_set_cluster_config,
    SetClusterConfigRequest,
    SetClusterConfigReply,
    SetClusterConfig
);

generate_mqtt_admin_service_call!(
    mqtt_broker_get_cluster_config,
    GetClusterConfigRequest,
    GetClusterConfigReply,
    GetClusterConfig
);

// ---- cluster ------
generate_mqtt_admin_service_call!(
    mqtt_broker_cluster_status,
    ClusterStatusRequest,
    ClusterStatusReply,
    ClusterStatus
);

// ------ user -------
generate_mqtt_admin_service_call!(
    mqtt_broker_list_user,
    ListUserRequest,
    ListUserReply,
    ListUser
);

generate_mqtt_admin_service_call!(
    mqtt_broker_create_user,
    CreateUserRequest,
    CreateUserReply,
    CreateUser
);

generate_mqtt_admin_service_call!(
    mqtt_broker_delete_user,
    DeleteUserRequest,
    DeleteUserReply,
    DeleteUser
);

generate_mqtt_admin_service_call!(mqtt_broker_list_acl, ListAclRequest, ListAclReply, ListAcl);

generate_mqtt_admin_service_call!(
    mqtt_broker_create_acl,
    CreateAclRequest,
    CreateAclReply,
    CreateAcl
);

generate_mqtt_admin_service_call!(
    mqtt_broker_delete_acl,
    DeleteAclRequest,
    DeleteAclReply,
    DeleteAcl
);

generate_mqtt_admin_service_call!(
    mqtt_broker_list_blacklist,
    ListBlacklistRequest,
    ListBlacklistReply,
    ListBlacklist
);

generate_mqtt_admin_service_call!(
    mqtt_broker_create_blacklist,
    CreateBlacklistRequest,
    CreateBlacklistReply,
    CreateBlacklist
);

generate_mqtt_admin_service_call!(
    mqtt_broker_delete_blacklist,
    DeleteBlacklistRequest,
    DeleteBlacklistReply,
    DeleteBlacklist
);

// ------- connection  -----------
generate_mqtt_admin_service_call!(
    mqtt_broker_list_connection,
    ListConnectionRequest,
    ListConnectionReply,
    ListConnection
);

// -------flapping detect feat  -----------
generate_mqtt_admin_service_call!(
    mqtt_broker_enable_flapping_detect,
    EnableFlappingDetectRequest,
    EnableFlappingDetectReply,
    EnableFlappingDetect
);

generate_mqtt_admin_service_call!(
    mqtt_broker_list_flapping_detect,
    ListFlappingDetectRequest,
    Streaming<ListFlappingDetectReply>,
    ListFlappingDetect
);

// #### observability ####
// ---- slow subscribe features ----

generate_mqtt_admin_service_call!(
    mqtt_broker_list_slow_subscribe,
    ListSlowSubscribeRequest,
    ListSlowSubscribeReply,
    ListSlowSubscribe
);

// ---- system alarm ----
generate_mqtt_admin_service_call!(
    mqtt_broker_set_system_alarm_config,
    SetSystemAlarmConfigRequest,
    SetSystemAlarmConfigReply,
    SetSystemAlarmConfig
);

generate_mqtt_admin_service_call!(
    mqtt_broker_list_system_alarm,
    ListSystemAlarmRequest,
    ListSystemAlarmReply,
    ListSystemAlarm
);

generate_mqtt_admin_service_call!(
    mqtt_broker_list_topic,
    ListTopicRequest,
    ListTopicReply,
    ListTopic
);

generate_mqtt_admin_service_call!(
    mqtt_broker_create_topic_rewrite_rule,
    CreateTopicRewriteRuleRequest,
    CreateTopicRewriteRuleReply,
    CreateTopicRewriteRule
);

generate_mqtt_admin_service_call!(
    mqtt_broker_delete_topic_rewrite_rule,
    DeleteTopicRewriteRuleRequest,
    DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRule
);

// connector command line CRUD
generate_mqtt_admin_service_call!(
    mqtt_broker_list_connector,
    ListConnectorRequest,
    ListConnectorReply,
    MqttListConnector
);

generate_mqtt_admin_service_call!(
    mqtt_broker_create_connector,
    CreateConnectorRequest,
    CreateConnectorReply,
    MqttCreateConnector
);

generate_mqtt_admin_service_call!(
    mqtt_broker_update_connector,
    UpdateConnectorRequest,
    UpdateConnectorReply,
    MqttUpdateConnector
);

generate_mqtt_admin_service_call!(
    mqtt_broker_delete_connector,
    DeleteConnectorRequest,
    DeleteConnectorReply,
    MqttDeleteConnector
);

// schema command line CRUD
generate_mqtt_admin_service_call!(
    mqtt_broker_list_schema,
    ListSchemaRequest,
    ListSchemaReply,
    MqttListSchema
);

generate_mqtt_admin_service_call!(
    mqtt_broker_create_schema,
    CreateSchemaRequest,
    CreateSchemaReply,
    MqttCreateSchema
);

generate_mqtt_admin_service_call!(
    mqtt_broker_update_schema,
    UpdateSchemaRequest,
    UpdateSchemaReply,
    MqttUpdateSchema
);

generate_mqtt_admin_service_call!(
    mqtt_broker_delete_schema,
    DeleteSchemaRequest,
    DeleteSchemaReply,
    MqttDeleteSchema
);

generate_mqtt_admin_service_call!(
    mqtt_broker_list_bind_schema,
    ListBindSchemaRequest,
    ListBindSchemaReply,
    MqttListBindSchema
);

generate_mqtt_admin_service_call!(
    mqtt_broker_bind_schema,
    BindSchemaRequest,
    BindSchemaReply,
    MqttBindSchema
);

generate_mqtt_admin_service_call!(
    mqtt_broker_unbind_schema,
    UnbindSchemaRequest,
    UnbindSchemaReply,
    MqttUnbindSchema
);

generate_mqtt_admin_service_call!(
    mqtt_broker_set_auto_subscribe_rule,
    SetAutoSubscribeRuleRequest,
    SetAutoSubscribeRuleReply,
    SetAutoSubscribeRule
);

generate_mqtt_admin_service_call!(
    mqtt_broker_delete_auto_subscribe_rule,
    DeleteAutoSubscribeRuleRequest,
    DeleteAutoSubscribeRuleReply,
    DeleteAutoSubscribeRule
);

generate_mqtt_admin_service_call!(
    mqtt_broker_list_auto_subscribe_rule,
    ListAutoSubscribeRuleRequest,
    ListAutoSubscribeRuleReply,
    ListAutoSubscribeRule
);

generate_mqtt_admin_service_call!(
    mqtt_broker_list_session,
    ListSessionRequest,
    ListSessionReply,
    ListSession
);

generate_mqtt_admin_service_call!(
    mqtt_broker_cluster_overview_metrics,
    ClusterOverviewMetricsRequest,
    ClusterOverviewMetricsReply,
    ClusterOverviewMetrics
);

generate_mqtt_admin_service_call!(
    mqtt_broker_list_subscribe,
    ListSubscribeRequest,
    ListSubscribeReply,
    ListSubscribe
);

generate_mqtt_admin_service_call!(
    mqtt_broker_subscribe_detail,
    SubscribeDetailRequest,
    SubscribeDetailReply,
    SubscribeDetail
);
