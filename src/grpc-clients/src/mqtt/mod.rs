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
use protocol::broker_mqtt::broker_mqtt_admin::{ClusterStatusReply, ClusterStatusRequest, CreateUserReply, CreateUserRequest, DeleteUserReply, DeleteUserRequest, EnableSlowSubScribeReply, EnableSlowSubscribeRequest, ListConnectionReply, ListConnectionRequest, ListUserReply, ListUserRequest};
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

    // connection
    ListConnection(ListConnectionRequest),

    // slow subscribe
    EnableSlowSubscribe(EnableSlowSubscribeRequest),
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

    // connection
    ListConnection(ListConnectionReply),


    // slow subscribe
    EnableSlowSubscribe(EnableSlowSubScribeReply)
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
            Ok(MqttBrokerPlacementReply::EnableSlowSubscribe(reply.into_inner()))
        }
    }
}

#[cfg(test)]
mod tests {}
