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

use common_base::error::common::CommonError;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, ClusterStatusRequest, CreateUserReply, CreateUserRequest, DeleteUserReply,
    DeleteUserRequest, ListConnectionReply, ListConnectionRequest, ListUserReply, ListUserRequest,
};
use protocol::placement_center::placement_center_mqtt::ListTopicReply;

use crate::mqtt::{call_once, MqttBrokerPlacementReply, MqttBrokerPlacementRequest};
use crate::pool::ClientPool;
use crate::utils::retry_call;

// ---- cluster ------
pub async fn cluster_status(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: ClusterStatusRequest,
) -> Result<ClusterStatusReply, CommonError> {
    let request = MqttBrokerPlacementRequest::ClusterStatus(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::ClusterStatus(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

// ------ user -------
pub async fn mqtt_broker_list_user(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: ListUserRequest,
) -> Result<ListUserReply, CommonError> {
    // let reply = retry_call(client_pool, addrs, MqttBrokerPlacementRequest::ListUser(request)).await?;
    let request = MqttBrokerPlacementRequest::ListUser(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::ListUser(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_create_user(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: CreateUserRequest,
) -> Result<CreateUserReply, CommonError> {
    let request = MqttBrokerPlacementRequest::CreateUser(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::CreateUser(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_delete_user(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: DeleteUserRequest,
) -> Result<DeleteUserReply, CommonError> {
    let request = MqttBrokerPlacementRequest::DeleteUser(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::DeleteUser(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

// ------- connection  -----------
pub async fn mqtt_broker_list_connection(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: ListConnectionRequest,
) -> Result<ListConnectionReply, CommonError> {
    let request = MqttBrokerPlacementRequest::ListConnection(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::ListConnection(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_list_user(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: ListTopicReply,
) -> Result<ListTopicReply, CommonError> {
    let request = MqttBrokerPlacementRequest::ListTopic(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::ListTopic(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}