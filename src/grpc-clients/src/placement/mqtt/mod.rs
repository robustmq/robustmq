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

use crate::pool::ClientPool;

pub mod call;

/// Enum wrapper for all possible requests to the mqtt service
#[derive(Debug, Clone)]
pub enum MqttServiceRequest {
    GetShareSubLeader(GetShareSubLeaderRequest),
    CreateUser(CreateUserRequest),
    DeleteUser(DeleteUserRequest),
    ListUser(ListUserRequest),
    CreateTopic(CreateTopicRequest),
    DeleteTopic(DeleteTopicRequest),
    ListTopic(ListTopicRequest),
    SetTopicRetainMessage(SetTopicRetainMessageRequest),
    SetNxExclusiveTopic(SetExclusiveTopicRequest),
    DeleteExclusiveTopic(DeleteExclusiveTopicRequest),
    CreateSession(CreateSessionRequest),
    DeleteSession(DeleteSessionRequest),
    ListSession(ListSessionRequest),
    UpdateSession(UpdateSessionRequest),
    SaveLastWillMessage(SaveLastWillMessageRequest),
    CreateAcl(CreateAclRequest),
    DeleteAcl(DeleteAclRequest),
    ListAcl(ListAclRequest),
    CreateBlacklist(CreateBlacklistRequest),
    DeleteBlacklist(DeleteBlacklistRequest),
    ListBlacklist(ListBlacklistRequest),
}

/// Enum wrapper for all possible replies from the mqtt service
#[derive(Debug, Clone)]
pub enum MqttServiceReply {
    GetShareSubLeader(GetShareSubLeaderReply),
    CreateUser(CreateUserReply),
    DeleteUser(DeleteUserReply),
    ListUser(ListUserReply),
    CreateTopic(CreateTopicReply),
    DeleteTopic(DeleteTopicReply),
    ListTopic(ListTopicReply),
    SetTopicRetainMessage(SetTopicRetainMessageReply),
    SetNxExclusiveTopic(SetExclusiveTopicReply),
    DeleteExclusiveTopic(DeleteExclusiveTopicReply),
    CreateSession(CreateSessionReply),
    DeleteSession(DeleteSessionReply),
    ListSession(ListSessionReply),
    UpdateSession(UpdateSessionReply),
    SaveLastWillMessage(SaveLastWillMessageReply),
    CreateAcl(CreateAclReply),
    DeleteAcl(DeleteAclReply),
    ListAcl(ListAclReply),
    CreateBlacklist(CreateBlacklistReply),
    DeleteBlacklist(DeleteBlacklistReply),
    ListBlacklist(ListBlacklistReply),
}

pub(super) async fn call_mqtt_service_once(
    client_pool: &ClientPool,
    addr: &str,
    request: MqttServiceRequest,
) -> Result<MqttServiceReply, CommonError> {
    use MqttServiceRequest::*;

    match request {
        GetShareSubLeader(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.get_share_sub_leader(request).await?;
            Ok(MqttServiceReply::GetShareSubLeader(reply.into_inner()))
        }
        CreateUser(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.create_user(request).await?;
            Ok(MqttServiceReply::CreateUser(reply.into_inner()))
        }
        DeleteUser(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.delete_user(request).await?;
            Ok(MqttServiceReply::DeleteUser(reply.into_inner()))
        }
        ListUser(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.list_user(request).await?;
            Ok(MqttServiceReply::ListUser(reply.into_inner()))
        }
        CreateTopic(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.create_topic(request).await?;
            Ok(MqttServiceReply::CreateTopic(reply.into_inner()))
        }
        DeleteTopic(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.delete_topic(request).await?;
            Ok(MqttServiceReply::DeleteTopic(reply.into_inner()))
        }
        ListTopic(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.list_topic(request).await?;
            Ok(MqttServiceReply::ListTopic(reply.into_inner()))
        }
        SetTopicRetainMessage(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.set_topic_retain_message(request).await?;
            Ok(MqttServiceReply::SetTopicRetainMessage(reply.into_inner()))
        }
        SetNxExclusiveTopic(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.set_nx_exclusive_topic(request).await?;
            Ok(MqttServiceReply::SetNxExclusiveTopic(reply.into_inner()))
        }
        DeleteExclusiveTopic(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.delete_exclusive_topic(request).await?;
            Ok(MqttServiceReply::DeleteExclusiveTopic(reply.into_inner()))
        }
        CreateSession(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.create_session(request).await?;
            Ok(MqttServiceReply::CreateSession(reply.into_inner()))
        }
        DeleteSession(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.delete_session(request).await?;
            Ok(MqttServiceReply::DeleteSession(reply.into_inner()))
        }
        ListSession(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.list_session(request).await?;
            Ok(MqttServiceReply::ListSession(reply.into_inner()))
        }
        UpdateSession(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.update_session(request).await?;
            Ok(MqttServiceReply::UpdateSession(reply.into_inner()))
        }
        SaveLastWillMessage(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.save_last_will_message(request).await?;
            Ok(MqttServiceReply::SaveLastWillMessage(reply.into_inner()))
        }
        CreateAcl(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.create_acl(request).await?;
            Ok(MqttServiceReply::CreateAcl(reply.into_inner()))
        }
        DeleteAcl(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.delete_acl(request).await?;
            Ok(MqttServiceReply::DeleteAcl(reply.into_inner()))
        }
        ListAcl(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.list_acl(request).await?;
            Ok(MqttServiceReply::ListAcl(reply.into_inner()))
        }
        CreateBlacklist(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.create_blacklist(request).await?;
            Ok(MqttServiceReply::CreateBlacklist(reply.into_inner()))
        }
        DeleteBlacklist(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.delete_blacklist(request).await?;
            Ok(MqttServiceReply::DeleteBlacklist(reply.into_inner()))
        }
        ListBlacklist(request) => {
            let mut client = client_pool
                .placement_center_mqtt_services_client(addr)
                .await?;
            let reply = client.list_blacklist(request).await?;
            Ok(MqttServiceReply::ListBlacklist(reply.into_inner()))
        }
    }
}

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
