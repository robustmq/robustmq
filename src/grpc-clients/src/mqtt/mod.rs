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

use std::sync::Arc;
use std::time::Duration;

use admin::admin_interface_call;
use common_base::error::common::CommonError;
use log::error;
use placement::placement_interface_call;
use protocol::broker_mqtt::broker_mqtt_admin::{ClusterStatusReply, ClusterStatusRequest, CreateUserReply, CreateUserRequest, DeleteUserReply, DeleteUserRequest, ListUserReply, ListUserRequest};
use protocol::broker_mqtt::broker_mqtt_placement::{DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest, UpdateCacheReply, UpdateCacheRequest};
use tokio::time::sleep;

use crate::pool::ClientPool;
use crate::{retry_sleep_time, retry_times};

#[derive(Clone)]
pub enum MqttBrokerService {
    Placement,
    Admin,
}

#[derive(Clone, Debug)]
pub enum MqttBrokerPlacementInterface {
    // placement
    DeleteSession,
    UpdateCache,
    SendLastWillMessage,

    // admin service  functions
    ClusterStatus,
    ListUser,
    CreateUser,
    DeleteUser,

    // connection
    ListConnection,
}

#[derive(Debug, Clone)]
pub enum MqttBrokerPlacementRequest {
    // placement
    DeleteSession(DeleteSessionRequest),
    UpdateCache(UpdateCacheRequest),
    SendLastWillMessage(SendLastWillMessageRequest),

    // admin
    ClusterStatus(ClusterStatusRequest),
    ListUser(ListUserRequest),
    CreateUser(CreateUserRequest),
    DeleteUser(DeleteUserRequest),
}

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
}

pub mod admin;
pub mod placement;

async fn retry_call(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>, // TODO: &[String] or &[&str] should suffice
    request: MqttBrokerPlacementRequest,
) -> Result<MqttBrokerPlacementReply, CommonError> {
    if addrs.is_empty() {
        return Err(CommonError::CommonError(
            "Call address list cannot be empty".to_string(),
        ));
    }

    let mut times = 1;
    loop {
        let index = times % addrs.len();
        let addr = addrs.get(index).unwrap().clone();
        let result = retry_call_inner(&client_pool, addr, request.clone()).await;

        match result {
            Ok(data) => {
                return Ok(data);
            }
            Err(e) => {
                error!("{}", e);
                if times > retry_times() {
                    return Err(e);
                }
                times += 1;
            }
        }

        sleep(Duration::from_secs(retry_sleep_time(times))).await;
    }
}

async fn retry_call_inner(
    client_pool: &ClientPool,
    addr: String,
    request: MqttBrokerPlacementRequest,
) -> Result<MqttBrokerPlacementReply, CommonError> {
    use MqttBrokerPlacementRequest::*;

    match request {
        DeleteSession(delete_session_request) => {
            let mut client = client_pool.mqtt_broker_mqtt_services_client(addr).await?;
            let reply = client.delete_session(delete_session_request).await?;
            Ok(MqttBrokerPlacementReply::DeleteSession(reply.into_inner()))
        },
        UpdateCache(update_cache_request) => {
            let mut client = client_pool.mqtt_broker_mqtt_services_client(addr).await?;
            let reply = client.update_cache(update_cache_request).await?;
            Ok(MqttBrokerPlacementReply::UpdateCache(reply.into_inner()))
        },
        SendLastWillMessage(send_last_will_message_request) => {
            let mut client = client_pool.mqtt_broker_mqtt_services_client(addr).await?;
            let reply = client.send_last_will_message(send_last_will_message_request).await?;
            Ok(MqttBrokerPlacementReply::SendLastWillMessage(reply.into_inner()))
        },
        ClusterStatus(cluster_status_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.cluster_status(cluster_status_request).await?;
            Ok(MqttBrokerPlacementReply::ClusterStatus(reply.into_inner()))
        },
        ListUser(list_user_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.mqtt_broker_list_user(list_user_request).await?;
            Ok(MqttBrokerPlacementReply::ListUser(reply.into_inner()))
        },
        CreateUser(create_user_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.mqtt_broker_create_user(create_user_request).await?;
            Ok(MqttBrokerPlacementReply::CreateUser(reply.into_inner()))
        },
        DeleteUser(delete_user_request) => {
            let mut client = client_pool.mqtt_broker_admin_services_client(addr).await?;
            let reply = client.mqtt_broker_delete_user(delete_user_request).await?;
            Ok(MqttBrokerPlacementReply::DeleteUser(reply.into_inner()))
        },
    }
}

#[cfg(test)]
mod tests {}
