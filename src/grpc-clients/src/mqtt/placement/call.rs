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
use protocol::broker_mqtt::broker_mqtt_inner::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateCacheReply, UpdateCacheRequest,
};

use crate::mqtt::{call_once, MqttBrokerPlacementReply, MqttBrokerPlacementRequest};
use crate::pool::ClientPool;
use crate::utils::retry_call;

pub async fn broker_mqtt_delete_session(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: DeleteSessionRequest,
) -> Result<DeleteSessionReply, CommonError> {
    let request = MqttBrokerPlacementRequest::DeleteSession(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::DeleteSession(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn broker_mqtt_update_cache(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: UpdateCacheRequest,
) -> Result<UpdateCacheReply, CommonError> {
    let request = MqttBrokerPlacementRequest::UpdateCache(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::UpdateCache(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn send_last_will_message(
    client_pool: Arc<ClientPool>,
    addrs: &[String],
    request: SendLastWillMessageRequest,
) -> Result<SendLastWillMessageReply, CommonError> {
    let request = MqttBrokerPlacementRequest::SendLastWillMessage(request);
    match retry_call(&client_pool, addrs, request, call_once).await? {
        MqttBrokerPlacementReply::SendLastWillMessage(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}
