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
    CreateBlacklistReply, CreateBlacklistRequest, CreateTopicRewriteRuleReply,
    CreateTopicRewriteRuleRequest, CreateUserReply, CreateUserRequest, DeleteAclReply,
    DeleteAclRequest, DeleteBlacklistReply, DeleteBlacklistRequest, DeleteTopicRewriteRuleReply,
    DeleteTopicRewriteRuleRequest, DeleteUserReply, DeleteUserRequest, EnableConnectionJitterReply,
    EnableConnectionJitterRequest, EnableSlowSubScribeReply, EnableSlowSubscribeRequest,
    ListAclReply, ListAclRequest, ListBlacklistReply, ListBlacklistRequest, ListConnectionReply,
    ListConnectionRequest, ListSlowSubscribeReply, ListSlowSubscribeRequest, ListTopicReply,
    ListTopicRequest, ListUserReply, ListUserRequest,
};

use crate::pool::ClientPool;
use crate::utils::retry_call;

// ---- cluster ------
pub async fn cluster_status(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: ClusterStatusRequest,
) -> Result<ClusterStatusReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

// ------ user -------
pub async fn mqtt_broker_list_user(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: ListUserRequest,
) -> Result<ListUserReply, CommonError> {
    // let reply = retry_call(client_pool, addrs, MqttBrokerPlacementRequest::ListUser(request)).await?;
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_create_user(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: CreateUserRequest,
) -> Result<CreateUserReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_delete_user(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: DeleteUserRequest,
) -> Result<DeleteUserReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_list_acl(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: ListAclRequest,
) -> Result<ListAclReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_create_acl(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: CreateAclRequest,
) -> Result<CreateAclReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_delete_acl(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: DeleteAclRequest,
) -> Result<DeleteAclReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_list_blacklist(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: ListBlacklistRequest,
) -> Result<ListBlacklistReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_create_blacklist(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: CreateBlacklistRequest,
) -> Result<CreateBlacklistReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_delete_blacklist(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: DeleteBlacklistRequest,
) -> Result<DeleteBlacklistReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

// ------- connection  -----------
pub async fn mqtt_broker_list_connection(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: ListConnectionRequest,
) -> Result<ListConnectionReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

// ------- connection jitter feat  -----------
pub async fn mqtt_broker_enable_connection_jitter(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: EnableConnectionJitterRequest,
) -> Result<EnableConnectionJitterReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

// --------- observability --------
// --------- slow subscribe features ------
pub async fn mqtt_broker_enable_slow_subscribe(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: EnableSlowSubscribeRequest,
) -> Result<EnableSlowSubScribeReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_list_slow_subscribe(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: ListSlowSubscribeRequest,
) -> Result<ListSlowSubscribeReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_list_topic(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: ListTopicRequest,
) -> Result<ListTopicReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_create_topic_rewrite_rule(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: CreateTopicRewriteRuleRequest,
) -> Result<CreateTopicRewriteRuleReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}

pub async fn mqtt_broker_delete_topic_rewrite_rule(
    client_pool: &ClientPool,
    addrs: &[impl AsRef<str>],
    request: DeleteTopicRewriteRuleRequest,
) -> Result<DeleteTopicRewriteRuleReply, CommonError> {
    retry_call(client_pool, addrs, request).await
}
