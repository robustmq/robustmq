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
    ClusterStatusReply, ClusterStatusRequest, CreateAclReply, CreateAclRequest,
    CreateBlacklistReply, CreateBlacklistRequest, CreateUserReply, CreateUserRequest,
    DeleteAclReply, DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest,
    DeleteUserReply, DeleteUserRequest, EnableSlowSubScribeReply, EnableSlowSubscribeRequest,
    ListAclReply, ListAclRequest, ListBlacklistReply, ListBlacklistRequest, ListConnectionReply,
    ListConnectionRequest, ListSlowSubscribeReply, ListSlowSubscribeRequest, ListTopicReply,
    ListTopicRequest, ListUserReply, ListUserRequest,
};

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

pub async fn mqtt_broker_list_acl(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: ListAclRequest,
) -> Result<ListAclReply, CommonError> {
    let request = MqttBrokerPlacementRequest::ListAcl(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::ListAcl(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_create_acl(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: CreateAclRequest,
) -> Result<CreateAclReply, CommonError> {
    let request = MqttBrokerPlacementRequest::CreateAcl(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::CreateAcl(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_delete_acl(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: DeleteAclRequest,
) -> Result<DeleteAclReply, CommonError> {
    let request = MqttBrokerPlacementRequest::DeleteAcl(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::DeleteAcl(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_list_blacklist(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: ListBlacklistRequest,
) -> Result<ListBlacklistReply, CommonError> {
    let request = MqttBrokerPlacementRequest::ListBlacklist(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::ListBlacklist(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_create_blacklist(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: CreateBlacklistRequest,
) -> Result<CreateBlacklistReply, CommonError> {
    let request = MqttBrokerPlacementRequest::CreateBlacklist(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::CreateBlacklist(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_delete_blacklist(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: DeleteBlacklistRequest,
) -> Result<DeleteBlacklistReply, CommonError> {
    let request = MqttBrokerPlacementRequest::DeleteBlacklist(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::DeleteBlacklist(reply) => Ok(reply),
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

// --------- observability --------
// --------- slow subscribe features ------
pub async fn mqtt_broker_enable_slow_subscribe(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: EnableSlowSubscribeRequest,
) -> Result<EnableSlowSubScribeReply, CommonError> {
    let request = MqttBrokerPlacementRequest::EnableSlowSubscribe(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::EnableSlowSubscribe(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_list_slow_subscribe(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: ListSlowSubscribeRequest,
) -> Result<ListSlowSubscribeReply, CommonError> {
    let request = MqttBrokerPlacementRequest::ListSlowSubscribe(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::ListSlowSubscribe(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn mqtt_broker_list_topic(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: ListTopicRequest,
) -> Result<ListTopicReply, CommonError> {
    let request = MqttBrokerPlacementRequest::ListTopic(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::ListTopic(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}
