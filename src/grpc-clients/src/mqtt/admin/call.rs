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
    ClusterStatusReply, ClusterStatusRequest, CreateAclReply, CreateAclRequest,
    CreateBlacklistReply, CreateBlacklistRequest, CreateTopicRewriteRuleReply,
    CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest, DeleteAclReply,
    DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest, DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest, EnableFlappingDetectReply,
    EnableFlappingDetectRequest, EnableSlowSubScribeReply, EnableSlowSubscribeRequest,
    ListAclReply, ListAclRequest, ListBlacklistReply, ListBlacklistRequest, ListConnectionReply,
    ListConnectionRequest, ListSlowSubscribeReply, ListSlowSubscribeRequest, ListTopicReply,
    ListTopicRequest, ListUserReply, ListUserRequest, MqttCreateConnectorReply,
    MqttCreateConnectorRequest, MqttDeleteConnectorReply, MqttDeleteConnectorRequest,
    MqttListConnectorReply, MqttListConnectorRequest, MqttUpdateConnectorReply,
    MqttUpdateConnectorRequest,
};

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

// --------- observability --------
// --------- slow subscribe features ------
generate_mqtt_admin_service_call!(
    mqtt_broker_enable_slow_subscribe,
    EnableSlowSubscribeRequest,
    EnableSlowSubScribeReply,
    EnableSlowSubscribe
);

generate_mqtt_admin_service_call!(
    mqtt_broker_list_slow_subscribe,
    ListSlowSubscribeRequest,
    ListSlowSubscribeReply,
    ListSlowSubscribe
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
    MqttListConnectorRequest,
    MqttListConnectorReply,
    MqttListConnector
);

generate_mqtt_admin_service_call!(
    mqtt_broker_create_connector,
    MqttCreateConnectorRequest,
    MqttCreateConnectorReply,
    MqttCreateConnector
);

generate_mqtt_admin_service_call!(
    mqtt_broker_update_connector,
    MqttUpdateConnectorRequest,
    MqttUpdateConnectorReply,
    MqttUpdateConnector
);

generate_mqtt_admin_service_call!(
    mqtt_broker_delete_connector,
    MqttDeleteConnectorRequest,
    MqttDeleteConnectorReply,
    MqttDeleteConnector
);
