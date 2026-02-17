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

use protocol::meta::meta_service_mqtt::mqtt_service_client::MqttServiceClient;
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
    UpdateConnectorRequest,
};
use tonic::transport::Channel;
use tonic::Streaming;

use crate::macros::impl_retriable_request;

pub mod call;

impl_retriable_request!(
    GetShareSubLeaderRequest,
    MqttServiceClient<Channel>,
    GetShareSubLeaderReply,
    get_share_sub_leader,
    "MqttService",
    "GetShareSubLeader",
    true
);

impl_retriable_request!(
    CreateUserRequest,
    MqttServiceClient<Channel>,
    CreateUserReply,
    create_user,
    "MqttService",
    "CreateUser",
    true
);

impl_retriable_request!(
    DeleteUserRequest,
    MqttServiceClient<Channel>,
    DeleteUserReply,
    delete_user,
    "MqttService",
    "DeleteUser",
    true
);

impl_retriable_request!(
    ListUserRequest,
    MqttServiceClient<Channel>,
    ListUserReply,
    list_user,
    "MqttService",
    "ListUser",
    true
);

impl_retriable_request!(
    CreateTopicRequest,
    MqttServiceClient<Channel>,
    CreateTopicReply,
    create_topic,
    "MqttService",
    "CreateTopic",
    true
);

impl_retriable_request!(
    DeleteTopicRequest,
    MqttServiceClient<Channel>,
    DeleteTopicReply,
    delete_topic,
    "MqttService",
    "DeleteTopic",
    true
);

impl_retriable_request!(
    ListTopicRequest,
    MqttServiceClient<Channel>,
    Streaming<ListTopicReply>,
    list_topic,
    "MqttService",
    "ListTopic",
    true
);

impl_retriable_request!(
    SetTopicRetainMessageRequest,
    MqttServiceClient<Channel>,
    SetTopicRetainMessageReply,
    set_topic_retain_message,
    "MqttService",
    "SetTopicRetainMessage",
    true
);

impl_retriable_request!(
    GetTopicRetainMessageRequest,
    MqttServiceClient<Channel>,
    GetTopicRetainMessageReply,
    get_topic_retain_message,
    "MqttService",
    "GetTopicRetainMessage",
    true
);

impl_retriable_request!(
    CreateSessionRequest,
    MqttServiceClient<Channel>,
    CreateSessionReply,
    create_session,
    "MqttService",
    "CreateSession",
    true
);

impl_retriable_request!(
    DeleteSessionRequest,
    MqttServiceClient<Channel>,
    DeleteSessionReply,
    delete_session,
    "MqttService",
    "DeleteSession",
    true
);

impl_retriable_request!(
    ListSessionRequest,
    MqttServiceClient<Channel>,
    ListSessionReply,
    list_session,
    "MqttService",
    "ListSession",
    true
);

impl_retriable_request!(
    SaveLastWillMessageRequest,
    MqttServiceClient<Channel>,
    SaveLastWillMessageReply,
    save_last_will_message,
    "MqttService",
    "SaveLastWillMessage",
    true
);

impl_retriable_request!(
    GetLastWillMessageRequest,
    MqttServiceClient<Channel>,
    GetLastWillMessageReply,
    get_last_will_message,
    "MqttService",
    "GetLastWillMessage",
    true
);

impl_retriable_request!(
    CreateAclRequest,
    MqttServiceClient<Channel>,
    CreateAclReply,
    create_acl,
    "MqttService",
    "CreateAcl",
    true
);

impl_retriable_request!(
    DeleteAclRequest,
    MqttServiceClient<Channel>,
    DeleteAclReply,
    delete_acl,
    "MqttService",
    "DeleteAcl",
    true
);

impl_retriable_request!(
    ListAclRequest,
    MqttServiceClient<Channel>,
    ListAclReply,
    list_acl,
    "MqttService",
    "ListAcl",
    true
);

impl_retriable_request!(
    CreateBlacklistRequest,
    MqttServiceClient<Channel>,
    CreateBlacklistReply,
    create_blacklist,
    "MqttService",
    "CreateBlacklist",
    true
);

impl_retriable_request!(
    DeleteBlacklistRequest,
    MqttServiceClient<Channel>,
    DeleteBlacklistReply,
    delete_blacklist,
    "MqttService",
    "DeleteBlacklist",
    true
);

impl_retriable_request!(
    ListBlacklistRequest,
    MqttServiceClient<Channel>,
    ListBlacklistReply,
    list_blacklist,
    "MqttService",
    "ListBlacklist",
    true
);

impl_retriable_request!(
    ListTopicRewriteRuleRequest,
    MqttServiceClient<Channel>,
    ListTopicRewriteRuleReply,
    list_topic_rewrite_rule,
    "MqttService",
    "ListTopicRewriteRule",
    true
);

impl_retriable_request!(
    CreateTopicRewriteRuleRequest,
    MqttServiceClient<Channel>,
    CreateTopicRewriteRuleReply,
    create_topic_rewrite_rule,
    "MqttService",
    "CreateTopicRewriteRule",
    true
);

impl_retriable_request!(
    DeleteTopicRewriteRuleRequest,
    MqttServiceClient<Channel>,
    DeleteTopicRewriteRuleReply,
    delete_topic_rewrite_rule,
    "MqttService",
    "DeleteTopicRewriteRule",
    true
);

impl_retriable_request!(
    SetSubscribeRequest,
    MqttServiceClient<Channel>,
    SetSubscribeReply,
    set_subscribe,
    "MqttService",
    "SetSubscribe",
    true
);

impl_retriable_request!(
    DeleteSubscribeRequest,
    MqttServiceClient<Channel>,
    DeleteSubscribeReply,
    delete_subscribe,
    "MqttService",
    "DeleteSubscribe",
    true
);

impl_retriable_request!(
    ListSubscribeRequest,
    MqttServiceClient<Channel>,
    ListSubscribeReply,
    list_subscribe,
    "MqttService",
    "ListSubscribe",
    true
);

impl_retriable_request!(
    ListConnectorRequest,
    MqttServiceClient<Channel>,
    ListConnectorReply,
    list_connectors,
    "MqttService",
    "ListConnectors",
    true
);

impl_retriable_request!(
    CreateConnectorRequest,
    MqttServiceClient<Channel>,
    CreateConnectorReply,
    create_connector,
    "MqttService",
    "CreateConnector",
    true
);

impl_retriable_request!(
    UpdateConnectorRequest,
    MqttServiceClient<Channel>,
    UpdateConnectorReply,
    update_connector,
    "MqttService",
    "UpdateConnector",
    true
);

impl_retriable_request!(
    DeleteConnectorRequest,
    MqttServiceClient<Channel>,
    DeleteConnectorReply,
    delete_connector,
    "MqttService",
    "DeleteConnector",
    true
);

impl_retriable_request!(
    ConnectorHeartbeatRequest,
    MqttServiceClient<Channel>,
    ConnectorHeartbeatReply,
    connector_heartbeat,
    "MqttService",
    "ConnectorHeartbeat",
    true
);

impl_retriable_request!(
    ListAutoSubscribeRuleRequest,
    MqttServiceClient<Channel>,
    ListAutoSubscribeRuleReply,
    list_auto_subscribe_rule,
    "MqttService",
    "ListAutoSubscribeRule",
    true
);

impl_retriable_request!(
    SetAutoSubscribeRuleRequest,
    MqttServiceClient<Channel>,
    SetAutoSubscribeRuleReply,
    set_auto_subscribe_rule,
    "MqttService",
    "SetAutoSubscribeRule",
    true
);

impl_retriable_request!(
    DeleteAutoSubscribeRuleRequest,
    MqttServiceClient<Channel>,
    DeleteAutoSubscribeRuleReply,
    delete_auto_subscribe_rule,
    "MqttService",
    "DeleteAutoSubscribeRule",
    true
);
