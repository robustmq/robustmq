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
use prost::Message as _;
use protocol::broker_mqtt::broker_mqtt_inner::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateCacheReply, UpdateCacheRequest,
};

use crate::mqtt::{retry_call, MqttBrokerPlacementInterface, MqttBrokerPlacementReply, MqttBrokerPlacementRequest, MqttBrokerService};
use crate::pool::ClientPool;

pub async fn broker_mqtt_delete_session(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteSessionRequest,
) -> Result<DeleteSessionReply, CommonError> {
    let reply = retry_call(client_pool, addrs, MqttBrokerPlacementRequest::DeleteSession(request)).await?;
    match reply {
        MqttBrokerPlacementReply::DeleteSession(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn broker_mqtt_update_cache(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: UpdateCacheRequest,
) -> Result<UpdateCacheReply, CommonError> {
    let reply = retry_call(client_pool, addrs, MqttBrokerPlacementRequest::UpdateCache(request)).await?;
    match reply {
        MqttBrokerPlacementReply::UpdateCache(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}

pub async fn send_last_will_message(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SendLastWillMessageRequest,
) -> Result<SendLastWillMessageReply, CommonError> {
    let reply = retry_call(client_pool, addrs, MqttBrokerPlacementRequest::SendLastWillMessage(request)).await?;
    match reply {
        MqttBrokerPlacementReply::SendLastWillMessage(reply) => Ok(reply),
        _ => unreachable!("Reply type mismatch"),
    }
}
