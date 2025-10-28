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
use protocol::meta::meta_service_mqtt::{
    ConnectorHeartbeatReply, ConnectorHeartbeatRequest, CreateAclReply, CreateAclRequest,
    CreateBlacklistReply, CreateBlacklistRequest, CreateConnectorReply, CreateConnectorRequest,
    CreateSessionReply, CreateSessionRequest, CreateTopicReply, CreateTopicRequest,
    CreateTopicRewriteRuleReply, CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest,
    DeleteAclReply, DeleteAclRequest, DeleteAutoSubscribeRuleReply, DeleteAutoSubscribeRuleRequest,
    DeleteBlacklistReply, DeleteBlacklistRequest, DeleteConnectorReply, DeleteConnectorRequest,
    DeleteSessionReply, DeleteSessionRequest, DeleteSubscribeReply, DeleteSubscribeRequest,
    DeleteTopicReply, DeleteTopicRequest, DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest, GetLastWillMessageReply,
    GetLastWillMessageRequest, GetShareSubLeaderReply, GetShareSubLeaderRequest,
    GetTopicRetainMessageReply, GetTopicRetainMessageRequest, ListAclReply, ListAclRequest,
    ListAutoSubscribeRuleReply, ListAutoSubscribeRuleRequest, ListBlacklistReply,
    ListBlacklistRequest, ListConnectorReply, ListConnectorRequest, ListSessionReply,
    ListSessionRequest, ListSubscribeReply, ListSubscribeRequest, ListTopicReply, ListTopicRequest,
    ListTopicRewriteRuleReply, ListTopicRewriteRuleRequest, ListUserReply, ListUserRequest,
    SaveLastWillMessageReply, SaveLastWillMessageRequest, SetAutoSubscribeRuleReply,
    SetAutoSubscribeRuleRequest, SetSubscribeReply, SetSubscribeRequest,
    SetTopicRetainMessageReply, SetTopicRetainMessageRequest, UpdateConnectorReply,
    UpdateConnectorRequest, UpdateSessionReply, UpdateSessionRequest,
};
use tonic::Streaming;

use crate::pool::ClientPool;

macro_rules! generate_mqtt_service_call {
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

generate_mqtt_service_call!(
    placement_get_share_sub_leader,
    GetShareSubLeaderRequest,
    GetShareSubLeaderReply,
    GetShareSubLeader
);
generate_mqtt_service_call!(
    placement_create_user,
    CreateUserRequest,
    CreateUserReply,
    CreateUser
);
generate_mqtt_service_call!(
    placement_delete_user,
    DeleteUserRequest,
    DeleteUserReply,
    DeleteUser
);
generate_mqtt_service_call!(
    placement_list_user,
    ListUserRequest,
    ListUserReply,
    ListUser
);
generate_mqtt_service_call!(
    placement_create_topic,
    CreateTopicRequest,
    CreateTopicReply,
    CreateTopic
);
generate_mqtt_service_call!(
    placement_delete_topic,
    DeleteTopicRequest,
    DeleteTopicReply,
    DeleteTopic
);
generate_mqtt_service_call!(
    placement_list_topic,
    ListTopicRequest,
    Streaming<ListTopicReply>,
    ListTopic
);
generate_mqtt_service_call!(
    placement_set_topic_retain_message,
    SetTopicRetainMessageRequest,
    SetTopicRetainMessageReply,
    SetTopicRetainMessage
);

generate_mqtt_service_call!(
    placement_get_topic_retain_message,
    GetTopicRetainMessageRequest,
    GetTopicRetainMessageReply,
    GetTopicRetainMessage
);

generate_mqtt_service_call!(
    placement_create_session,
    CreateSessionRequest,
    CreateSessionReply,
    CreateSession
);
generate_mqtt_service_call!(
    placement_delete_session,
    DeleteSessionRequest,
    DeleteSessionReply,
    DeleteSession
);
generate_mqtt_service_call!(
    placement_list_session,
    ListSessionRequest,
    ListSessionReply,
    ListSession
);
generate_mqtt_service_call!(
    placement_update_session,
    UpdateSessionRequest,
    UpdateSessionReply,
    UpdateSession
);
generate_mqtt_service_call!(
    placement_save_last_will_message,
    SaveLastWillMessageRequest,
    SaveLastWillMessageReply,
    SaveLastWillMessage
);
generate_mqtt_service_call!(
    placement_get_last_will_message,
    GetLastWillMessageRequest,
    GetLastWillMessageReply,
    GetLastWillMessage
);
generate_mqtt_service_call!(create_acl, CreateAclRequest, CreateAclReply, CreateAcl);
generate_mqtt_service_call!(delete_acl, DeleteAclRequest, DeleteAclReply, DeleteAcl);
generate_mqtt_service_call!(list_acl, ListAclRequest, ListAclReply, ListAcl);
generate_mqtt_service_call!(
    create_blacklist,
    CreateBlacklistRequest,
    CreateBlacklistReply,
    CreateBlacklist
);
generate_mqtt_service_call!(
    list_blacklist,
    ListBlacklistRequest,
    ListBlacklistReply,
    ListBlacklist
);
generate_mqtt_service_call!(
    delete_blacklist,
    DeleteBlacklistRequest,
    DeleteBlacklistReply,
    DeleteBlacklist
);
generate_mqtt_service_call!(
    placement_list_topic_rewrite_rule,
    ListTopicRewriteRuleRequest,
    ListTopicRewriteRuleReply,
    ListTopicRewriteRule
);
generate_mqtt_service_call!(
    placement_create_topic_rewrite_rule,
    CreateTopicRewriteRuleRequest,
    CreateTopicRewriteRuleReply,
    CreateTopicRewriteRule
);
generate_mqtt_service_call!(
    placement_delete_topic_rewrite_rule,
    DeleteTopicRewriteRuleRequest,
    DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRule
);

generate_mqtt_service_call!(
    placement_set_subscribe,
    SetSubscribeRequest,
    SetSubscribeReply,
    SetSubscribe
);

generate_mqtt_service_call!(
    placement_delete_subscribe,
    DeleteSubscribeRequest,
    DeleteSubscribeReply,
    DeleteSubscribe
);

generate_mqtt_service_call!(
    placement_list_subscribe,
    ListSubscribeRequest,
    ListSubscribeReply,
    ListSubscribe
);

generate_mqtt_service_call!(
    placement_list_connector,
    ListConnectorRequest,
    ListConnectorReply,
    ListConnector
);

generate_mqtt_service_call!(
    placement_create_connector,
    CreateConnectorRequest,
    CreateConnectorReply,
    CreateConnector
);

generate_mqtt_service_call!(
    placement_update_connector,
    UpdateConnectorRequest,
    UpdateConnectorReply,
    UpdateConnector
);

generate_mqtt_service_call!(
    placement_delete_connector,
    DeleteConnectorRequest,
    DeleteConnectorReply,
    DeleteConnector
);

generate_mqtt_service_call!(
    placement_connector_heartbeat,
    ConnectorHeartbeatRequest,
    ConnectorHeartbeatReply,
    ConnectorHeartbeat
);

generate_mqtt_service_call!(
    placement_list_auto_subscribe_rule,
    ListAutoSubscribeRuleRequest,
    ListAutoSubscribeRuleReply,
    ListAutoSubscribeRule
);
generate_mqtt_service_call!(
    placement_set_auto_subscribe_rule,
    SetAutoSubscribeRuleRequest,
    SetAutoSubscribeRuleReply,
    SetAutoSubscribeRule
);
generate_mqtt_service_call!(
    placement_delete_auto_subscribe_rule,
    DeleteAutoSubscribeRuleRequest,
    DeleteAutoSubscribeRuleReply,
    DeleteAutoSubscribeRule
);
