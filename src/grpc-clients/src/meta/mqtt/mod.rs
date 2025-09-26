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
    DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest, GetShareSubLeaderReply,
    GetShareSubLeaderRequest, GetTopicRetainMessageReply, GetTopicRetainMessageRequest,
    ListAclReply, ListAclRequest, ListAutoSubscribeRuleReply, ListAutoSubscribeRuleRequest,
    ListBlacklistReply, ListBlacklistRequest, ListConnectorReply, ListConnectorRequest,
    ListSessionReply, ListSessionRequest, ListSubscribeReply, ListSubscribeRequest, ListTopicReply,
    ListTopicRequest, ListTopicRewriteRuleReply, ListTopicRewriteRuleRequest, ListUserReply,
    ListUserRequest, SaveLastWillMessageReply, SaveLastWillMessageRequest,
    SetAutoSubscribeRuleReply, SetAutoSubscribeRuleRequest, SetSubscribeReply, SetSubscribeRequest,
    SetTopicRetainMessageReply, SetTopicRetainMessageRequest, UpdateConnectorReply,
    UpdateConnectorRequest, UpdateSessionReply, UpdateSessionRequest,
};
use tonic::transport::Channel;
use tonic::Streaming;

use crate::macros::impl_retriable_request;

pub mod call;

#[derive(Clone)]
pub struct MqttServiceManager {
    pub addr: String,
}

impl MqttServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for MqttServiceManager {
    type Connection = MqttServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match MqttServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(CommonError::CommonError(format!(
                    "{},{}",
                    err,
                    self.addr.clone()
                )))
            }
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

impl_retriable_request!(
    GetShareSubLeaderRequest,
    MqttServiceClient<Channel>,
    GetShareSubLeaderReply,
    meta_service_mqtt_services_client,
    get_share_sub_leader,
    true
);

impl_retriable_request!(
    CreateUserRequest,
    MqttServiceClient<Channel>,
    CreateUserReply,
    meta_service_mqtt_services_client,
    create_user,
    true
);

impl_retriable_request!(
    DeleteUserRequest,
    MqttServiceClient<Channel>,
    DeleteUserReply,
    meta_service_mqtt_services_client,
    delete_user,
    true
);

impl_retriable_request!(
    ListUserRequest,
    MqttServiceClient<Channel>,
    ListUserReply,
    meta_service_mqtt_services_client,
    list_user,
    true
);

impl_retriable_request!(
    CreateTopicRequest,
    MqttServiceClient<Channel>,
    CreateTopicReply,
    meta_service_mqtt_services_client,
    create_topic,
    true
);

impl_retriable_request!(
    DeleteTopicRequest,
    MqttServiceClient<Channel>,
    DeleteTopicReply,
    meta_service_mqtt_services_client,
    delete_topic,
    true
);

impl_retriable_request!(
    ListTopicRequest,
    MqttServiceClient<Channel>,
    Streaming<ListTopicReply>,
    meta_service_mqtt_services_client,
    list_topic,
    true
);

impl_retriable_request!(
    SetTopicRetainMessageRequest,
    MqttServiceClient<Channel>,
    SetTopicRetainMessageReply,
    meta_service_mqtt_services_client,
    set_topic_retain_message,
    true
);

impl_retriable_request!(
    GetTopicRetainMessageRequest,
    MqttServiceClient<Channel>,
    GetTopicRetainMessageReply,
    meta_service_mqtt_services_client,
    get_topic_retain_message,
    true
);

impl_retriable_request!(
    CreateSessionRequest,
    MqttServiceClient<Channel>,
    CreateSessionReply,
    meta_service_mqtt_services_client,
    create_session,
    true
);

impl_retriable_request!(
    DeleteSessionRequest,
    MqttServiceClient<Channel>,
    DeleteSessionReply,
    meta_service_mqtt_services_client,
    delete_session,
    true
);

impl_retriable_request!(
    ListSessionRequest,
    MqttServiceClient<Channel>,
    ListSessionReply,
    meta_service_mqtt_services_client,
    list_session,
    true
);

impl_retriable_request!(
    UpdateSessionRequest,
    MqttServiceClient<Channel>,
    UpdateSessionReply,
    meta_service_mqtt_services_client,
    update_session,
    true
);

impl_retriable_request!(
    SaveLastWillMessageRequest,
    MqttServiceClient<Channel>,
    SaveLastWillMessageReply,
    meta_service_mqtt_services_client,
    save_last_will_message,
    true
);

impl_retriable_request!(
    CreateAclRequest,
    MqttServiceClient<Channel>,
    CreateAclReply,
    meta_service_mqtt_services_client,
    create_acl,
    true
);

impl_retriable_request!(
    DeleteAclRequest,
    MqttServiceClient<Channel>,
    DeleteAclReply,
    meta_service_mqtt_services_client,
    delete_acl,
    true
);

impl_retriable_request!(
    ListAclRequest,
    MqttServiceClient<Channel>,
    ListAclReply,
    meta_service_mqtt_services_client,
    list_acl,
    true
);

impl_retriable_request!(
    CreateBlacklistRequest,
    MqttServiceClient<Channel>,
    CreateBlacklistReply,
    meta_service_mqtt_services_client,
    create_blacklist,
    true
);

impl_retriable_request!(
    DeleteBlacklistRequest,
    MqttServiceClient<Channel>,
    DeleteBlacklistReply,
    meta_service_mqtt_services_client,
    delete_blacklist,
    true
);

impl_retriable_request!(
    ListBlacklistRequest,
    MqttServiceClient<Channel>,
    ListBlacklistReply,
    meta_service_mqtt_services_client,
    list_blacklist,
    true
);

impl_retriable_request!(
    ListTopicRewriteRuleRequest,
    MqttServiceClient<Channel>,
    ListTopicRewriteRuleReply,
    meta_service_mqtt_services_client,
    list_topic_rewrite_rule,
    true
);

impl_retriable_request!(
    CreateTopicRewriteRuleRequest,
    MqttServiceClient<Channel>,
    CreateTopicRewriteRuleReply,
    meta_service_mqtt_services_client,
    create_topic_rewrite_rule,
    true
);

impl_retriable_request!(
    DeleteTopicRewriteRuleRequest,
    MqttServiceClient<Channel>,
    DeleteTopicRewriteRuleReply,
    meta_service_mqtt_services_client,
    delete_topic_rewrite_rule,
    true
);

impl_retriable_request!(
    SetSubscribeRequest,
    MqttServiceClient<Channel>,
    SetSubscribeReply,
    meta_service_mqtt_services_client,
    set_subscribe,
    true
);

impl_retriable_request!(
    DeleteSubscribeRequest,
    MqttServiceClient<Channel>,
    DeleteSubscribeReply,
    meta_service_mqtt_services_client,
    delete_subscribe,
    true
);

impl_retriable_request!(
    ListSubscribeRequest,
    MqttServiceClient<Channel>,
    ListSubscribeReply,
    meta_service_mqtt_services_client,
    list_subscribe,
    true
);

impl_retriable_request!(
    ListConnectorRequest,
    MqttServiceClient<Channel>,
    ListConnectorReply,
    meta_service_mqtt_services_client,
    list_connectors,
    true
);

impl_retriable_request!(
    CreateConnectorRequest,
    MqttServiceClient<Channel>,
    CreateConnectorReply,
    meta_service_mqtt_services_client,
    create_connector,
    true
);

impl_retriable_request!(
    UpdateConnectorRequest,
    MqttServiceClient<Channel>,
    UpdateConnectorReply,
    meta_service_mqtt_services_client,
    update_connector,
    true
);

impl_retriable_request!(
    DeleteConnectorRequest,
    MqttServiceClient<Channel>,
    DeleteConnectorReply,
    meta_service_mqtt_services_client,
    delete_connector,
    true
);

impl_retriable_request!(
    ConnectorHeartbeatRequest,
    MqttServiceClient<Channel>,
    ConnectorHeartbeatReply,
    meta_service_mqtt_services_client,
    connector_heartbeat,
    true
);

impl_retriable_request!(
    ListAutoSubscribeRuleRequest,
    MqttServiceClient<Channel>,
    ListAutoSubscribeRuleReply,
    meta_service_mqtt_services_client,
    list_auto_subscribe_rule,
    true
);

impl_retriable_request!(
    SetAutoSubscribeRuleRequest,
    MqttServiceClient<Channel>,
    SetAutoSubscribeRuleReply,
    meta_service_mqtt_services_client,
    set_auto_subscribe_rule,
    true
);

impl_retriable_request!(
    DeleteAutoSubscribeRuleRequest,
    MqttServiceClient<Channel>,
    DeleteAutoSubscribeRuleReply,
    meta_service_mqtt_services_client,
    delete_auto_subscribe_rule,
    true
);
