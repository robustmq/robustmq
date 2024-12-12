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
use protocol::placement_center::placement_center_mqtt::mqtt_service_client::MqttServiceClient;
use protocol::placement_center::placement_center_mqtt::{
    CreateAclReply, CreateAclRequest, CreateBlacklistReply, CreateBlacklistRequest,
    CreateSessionReply, CreateSessionRequest, CreateTopicReply, CreateTopicRequest,
    CreateUserReply, CreateUserRequest, DeleteAclReply, DeleteAclRequest, DeleteBlacklistReply,
    DeleteBlacklistRequest, DeleteExclusiveTopicReply, DeleteExclusiveTopicRequest,
    DeleteSessionReply, DeleteSessionRequest, DeleteTopicReply, DeleteTopicRequest,
    DeleteUserReply, DeleteUserRequest, GetShareSubLeaderReply, GetShareSubLeaderRequest,
    ListAclReply, ListAclRequest, ListBlacklistReply, ListBlacklistRequest, ListSessionReply,
    ListSessionRequest, ListTopicReply, ListTopicRequest, ListUserReply, ListUserRequest,
    SaveLastWillMessageReply, SaveLastWillMessageRequest, SetExclusiveTopicReply,
    SetExclusiveTopicRequest, SetTopicRetainMessageReply, SetTopicRetainMessageRequest,
    UpdateSessionReply, UpdateSessionRequest,
};
use tonic::transport::Channel;

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
    placement_center_mqtt_services_client,
    get_share_sub_leader,
    true
);

impl_retriable_request!(
    CreateUserRequest,
    MqttServiceClient<Channel>,
    CreateUserReply,
    placement_center_mqtt_services_client,
    create_user,
    true
);

impl_retriable_request!(
    DeleteUserRequest,
    MqttServiceClient<Channel>,
    DeleteUserReply,
    placement_center_mqtt_services_client,
    delete_user,
    true
);

impl_retriable_request!(
    ListUserRequest,
    MqttServiceClient<Channel>,
    ListUserReply,
    placement_center_mqtt_services_client,
    list_user,
    true
);

impl_retriable_request!(
    CreateTopicRequest,
    MqttServiceClient<Channel>,
    CreateTopicReply,
    placement_center_mqtt_services_client,
    create_topic,
    true
);

impl_retriable_request!(
    DeleteTopicRequest,
    MqttServiceClient<Channel>,
    DeleteTopicReply,
    placement_center_mqtt_services_client,
    delete_topic,
    true
);

impl_retriable_request!(
    ListTopicRequest,
    MqttServiceClient<Channel>,
    ListTopicReply,
    placement_center_mqtt_services_client,
    list_topic,
    true
);

impl_retriable_request!(
    SetTopicRetainMessageRequest,
    MqttServiceClient<Channel>,
    SetTopicRetainMessageReply,
    placement_center_mqtt_services_client,
    set_topic_retain_message,
    true
);

impl_retriable_request!(
    SetExclusiveTopicRequest,
    MqttServiceClient<Channel>,
    SetExclusiveTopicReply,
    placement_center_mqtt_services_client,
    set_nx_exclusive_topic,
    true
);

impl_retriable_request!(
    DeleteExclusiveTopicRequest,
    MqttServiceClient<Channel>,
    DeleteExclusiveTopicReply,
    placement_center_mqtt_services_client,
    delete_exclusive_topic,
    true
);

impl_retriable_request!(
    CreateSessionRequest,
    MqttServiceClient<Channel>,
    CreateSessionReply,
    placement_center_mqtt_services_client,
    create_session,
    true
);

impl_retriable_request!(
    DeleteSessionRequest,
    MqttServiceClient<Channel>,
    DeleteSessionReply,
    placement_center_mqtt_services_client,
    delete_session,
    true
);

impl_retriable_request!(
    ListSessionRequest,
    MqttServiceClient<Channel>,
    ListSessionReply,
    placement_center_mqtt_services_client,
    list_session,
    true
);

impl_retriable_request!(
    UpdateSessionRequest,
    MqttServiceClient<Channel>,
    UpdateSessionReply,
    placement_center_mqtt_services_client,
    update_session,
    true
);

impl_retriable_request!(
    SaveLastWillMessageRequest,
    MqttServiceClient<Channel>,
    SaveLastWillMessageReply,
    placement_center_mqtt_services_client,
    save_last_will_message,
    true
);

impl_retriable_request!(
    CreateAclRequest,
    MqttServiceClient<Channel>,
    CreateAclReply,
    placement_center_mqtt_services_client,
    create_acl,
    true
);

impl_retriable_request!(
    DeleteAclRequest,
    MqttServiceClient<Channel>,
    DeleteAclReply,
    placement_center_mqtt_services_client,
    delete_acl,
    true
);

impl_retriable_request!(
    ListAclRequest,
    MqttServiceClient<Channel>,
    ListAclReply,
    placement_center_mqtt_services_client,
    list_acl,
    true
);

impl_retriable_request!(
    CreateBlacklistRequest,
    MqttServiceClient<Channel>,
    CreateBlacklistReply,
    placement_center_mqtt_services_client,
    create_blacklist,
    true
);

impl_retriable_request!(
    DeleteBlacklistRequest,
    MqttServiceClient<Channel>,
    DeleteBlacklistReply,
    placement_center_mqtt_services_client,
    delete_blacklist,
    true
);

impl_retriable_request!(
    ListBlacklistRequest,
    MqttServiceClient<Channel>,
    ListBlacklistReply,
    placement_center_mqtt_services_client,
    list_blacklist,
    true
);
