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

use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_client::MqttBrokerAdminServiceClient;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, ClusterStatusRequest, CreateAclReply, CreateAclRequest,
    CreateBlacklistReply, CreateBlacklistRequest, CreateTopicRewriteRuleReply,
    CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest, DeleteAclReply,
    DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest, DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest, EnableSlowSubScribeReply,
    EnableSlowSubscribeRequest, ListAclReply, ListAclRequest, ListBlacklistReply,
    ListBlacklistRequest, ListConnectionReply, ListConnectionRequest, ListSlowSubscribeReply,
    ListSlowSubscribeRequest, ListTopicReply, ListTopicRequest, ListUserReply, ListUserRequest,
};
use protocol::broker_mqtt::broker_mqtt_inner::mqtt_broker_inner_service_client::MqttBrokerInnerServiceClient;
use protocol::broker_mqtt::broker_mqtt_inner::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateCacheReply, UpdateCacheRequest,
};
use tonic::transport::Channel;

use crate::macros::impl_retriable_request;

pub mod admin;
pub mod inner;

impl_retriable_request!(
    DeleteSessionRequest,
    MqttBrokerInnerServiceClient<Channel>,
    DeleteSessionReply,
    mqtt_broker_mqtt_services_client,
    delete_session
);

impl_retriable_request!(
    UpdateCacheRequest,
    MqttBrokerInnerServiceClient<Channel>,
    UpdateCacheReply,
    mqtt_broker_mqtt_services_client,
    update_cache
);

impl_retriable_request!(
    SendLastWillMessageRequest,
    MqttBrokerInnerServiceClient<Channel>,
    SendLastWillMessageReply,
    mqtt_broker_mqtt_services_client,
    send_last_will_message
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
    EnableSlowSubscribeRequest,
    MqttBrokerAdminServiceClient<Channel>,
    EnableSlowSubScribeReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_enable_slow_subscribe
);

impl_retriable_request!(
    ListSlowSubscribeRequest,
    MqttBrokerAdminServiceClient<Channel>,
    ListSlowSubscribeReply,
    mqtt_broker_admin_services_client,
    mqtt_broker_list_slow_subscribe
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

#[cfg(test)]
mod tests {}
