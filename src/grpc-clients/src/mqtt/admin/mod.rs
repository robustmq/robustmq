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
    ClusterStatusReply, ClusterStatusRequest, MqttCreateConnectorReply, MqttCreateConnectorRequest,
    MqttDeleteConnectorReply, MqttDeleteConnectorRequest, MqttListConnectorReply,
    MqttListConnectorRequest, MqttUpdateConnectorReply, MqttUpdateConnectorRequest,
};
use protocol::broker_mqtt::broker_mqtt_admin::{
    CreateAclReply, CreateAclRequest, CreateBlacklistReply, CreateBlacklistRequest,
    CreateTopicRewriteRuleReply, CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest,
    DeleteAclReply, DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest,
    DeleteTopicRewriteRuleReply, DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest,
    EnableFlappingDetectReply, EnableFlappingDetectRequest, EnableSlowSubScribeReply,
    EnableSlowSubscribeRequest, ListAclReply, ListAclRequest, ListBlacklistReply,
    ListBlacklistRequest, ListConnectionReply, ListConnectionRequest, ListSlowSubscribeReply,
    ListSlowSubscribeRequest, ListTopicReply, ListTopicRequest, ListUserReply, ListUserRequest,
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
