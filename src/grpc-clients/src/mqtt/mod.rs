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
    CreateBlacklistReply, CreateBlacklistRequest, CreateUserReply, CreateUserRequest,
    DeleteAclReply, DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest,
    DeleteUserReply, DeleteUserRequest, EnableSlowSubScribeReply, EnableSlowSubscribeRequest,
    ListAclReply, ListAclRequest, ListBlacklistReply, ListBlacklistRequest, ListConnectionReply,
    ListConnectionRequest, ListSlowSubscribeReply, ListSlowSubscribeRequest, ListTopicReply,
    ListTopicRequest, ListUserReply, ListUserRequest,
};
use protocol::broker_mqtt::broker_mqtt_inner::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateCacheReply, UpdateCacheRequest,
};

use crate::pool::ClientPool;

/// Enum wrapper for all possible requests to the mqtt broker
#[derive(Debug, Clone)]
pub enum MqttBrokerPlacementRequest {
    // inner
    DeleteSession(DeleteSessionRequest),
    UpdateCache(UpdateCacheRequest),
    SendLastWillMessage(SendLastWillMessageRequest),

    // admin
    ClusterStatus(ClusterStatusRequest),
    ListUser(ListUserRequest),
    CreateUser(CreateUserRequest),
    DeleteUser(DeleteUserRequest),
    ListAcl(ListAclRequest),
    CreateAcl(CreateAclRequest),
    DeleteAcl(DeleteAclRequest),
    ListBlacklist(ListBlacklistRequest),
    CreateBlacklist(CreateBlacklistRequest),
    DeleteBlacklist(DeleteBlacklistRequest),

    // connection
    ListConnection(ListConnectionRequest),

    // slow subscribe
    EnableSlowSubscribe(EnableSlowSubscribeRequest),
    ListSlowSubscribe(ListSlowSubscribeRequest),

    ListTopic(ListTopicRequest),
}

/// Enum wrapper for all possible replies from the mqtt broker
#[derive(Debug, Clone)]
pub enum MqttBrokerPlacementReply {
    // placement
    DeleteSession(DeleteSessionReply),
    UpdateCache(UpdateCacheReply),
    SendLastWillMessage(SendLastWillMessageReply),

    // admin
    ClusterStatus(ClusterStatusReply),
    ListUser(ListUserReply),
    CreateUser(CreateUserReply),
    DeleteUser(DeleteUserReply),
    ListAcl(ListAclReply),
    CreateAcl(CreateAclReply),
    DeleteAcl(DeleteAclReply),
    ListBlacklist(ListBlacklistReply),
    CreateBlacklist(CreateBlacklistReply),
    DeleteBlacklist(DeleteBlacklistReply),

    // connection
    ListConnection(ListConnectionReply),

    // slow subscribe
    EnableSlowSubscribe(EnableSlowSubScribeReply),

    ListTopic(ListTopicReply),
    ListSlowSubscribe(ListSlowSubscribeReply),
}

pub mod admin;
pub mod inner;

async fn call_once(
    client_pool: &ClientPool,
    addr: &str,
    request: MqttBrokerPlacementRequest,
) -> Result<MqttBrokerPlacementReply, CommonError> {
    use MqttBrokerPlacementRequest::*;

    match request {
        DeleteSession(delete_session_request) => {
            let mut client = client_pool.mqtt_broker_mqtt_services_client(addr).await?;
            let reply = client.delete_session(delete_session_request).await?;
            Ok(MqttBrokerPlacementReply::DeleteSession(reply.into_inner()))
        }
        UpdateCache(update_cache_request) => {
            let mut client = client_pool.mqtt_broker_mqtt_services_client(addr).await?;
            let reply = client.update_cache(update_cache_request).await?;
            Ok(MqttBrokerPlacementReply::UpdateCache(reply.into_inner()))
        }
        SendLastWillMessage(send_last_will_message_request) => {
            let mut client = client_pool.mqtt_broker_mqtt_services_client(addr).await?;
            let reply = client
                .send_last_will_message(send_last_will_message_request)
                .await?;
            Ok(MqttBrokerPlacementReply::SendLastWillMessage(
                reply.into_inner(),
            ))
        }
        ClusterStatus(cluster_status_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.cluster_status(cluster_status_request).await?;
            Ok(MqttBrokerPlacementReply::ClusterStatus(reply.into_inner()))
        }
        ListUser(list_user_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.mqtt_broker_list_user(list_user_request).await?;
            Ok(MqttBrokerPlacementReply::ListUser(reply.into_inner()))
        }
        CreateUser(create_user_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.mqtt_broker_create_user(create_user_request).await?;
            Ok(MqttBrokerPlacementReply::CreateUser(reply.into_inner()))
        }
        DeleteUser(delete_user_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.mqtt_broker_delete_user(delete_user_request).await?;
            Ok(MqttBrokerPlacementReply::DeleteUser(reply.into_inner()))
        }
        ListAcl(list_acl_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.mqtt_broker_list_acl(list_acl_request).await?;
            Ok(MqttBrokerPlacementReply::ListAcl(reply.into_inner()))
        }
        CreateAcl(create_acl_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.mqtt_broker_create_acl(create_acl_request).await?;
            Ok(MqttBrokerPlacementReply::CreateAcl(reply.into_inner()))
        }
        DeleteAcl(delete_acl_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.mqtt_broker_delete_acl(delete_acl_request).await?;
            Ok(MqttBrokerPlacementReply::DeleteAcl(reply.into_inner()))
        }
        ListBlacklist(list_blacklist_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client
                .mqtt_broker_list_blacklist(list_blacklist_request)
                .await?;
            Ok(MqttBrokerPlacementReply::ListBlacklist(reply.into_inner()))
        }
        CreateBlacklist(create_blacklist_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client
                .mqtt_broker_create_blacklist(create_blacklist_request)
                .await?;
            Ok(MqttBrokerPlacementReply::CreateBlacklist(
                reply.into_inner(),
            ))
        }
        DeleteBlacklist(delete_blacklist_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client
                .mqtt_broker_delete_blacklist(delete_blacklist_request)
                .await?;
            Ok(MqttBrokerPlacementReply::DeleteBlacklist(
                reply.into_inner(),
            ))
        }
        ListConnection(list_connection_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client
                .mqtt_broker_list_connection(list_connection_request)
                .await?;
            Ok(MqttBrokerPlacementReply::ListConnection(reply.into_inner()))
        }

        EnableSlowSubscribe(enable_slow_subscribe_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client
                .mqtt_broker_enable_slow_subscribe(enable_slow_subscribe_request)
                .await?;
            Ok(MqttBrokerPlacementReply::EnableSlowSubscribe(
                reply.into_inner(),
            ))
        }
        ListSlowSubscribe(list_slow_subscribe_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client
                .mqtt_broker_list_slow_subscribe(list_slow_subscribe_request)
                .await?;
            Ok(MqttBrokerPlacementReply::ListSlowSubscribe(
                reply.into_inner(),
            ))
        }

        ListTopic(list_topic_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.mqtt_broker_list_topic(list_topic_request).await?;
            Ok(MqttBrokerPlacementReply::ListTopic(reply.into_inner()))
        }
    }
}

#[cfg(test)]
mod tests {}
