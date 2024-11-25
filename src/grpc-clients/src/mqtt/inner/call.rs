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

use crate::mqtt::{retry_call, MqttBrokerInterface, MqttBrokerService};
use crate::pool::ClientPool;

pub async fn broker_mqtt_delete_session(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteSessionRequest,
) -> Result<DeleteSessionReply, CommonError> {
    let request_data = DeleteSessionRequest::encode_to_vec(&request);
    match retry_call(
        MqttBrokerService::Inner,
        MqttBrokerInterface::DeleteSession,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match DeleteSessionReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn broker_mqtt_update_cache(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: UpdateCacheRequest,
) -> Result<UpdateCacheReply, CommonError> {
    let request_data = UpdateCacheRequest::encode_to_vec(&request);
    match retry_call(
        MqttBrokerService::Inner,
        MqttBrokerInterface::UpdateCache,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match UpdateCacheReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}

pub async fn send_last_will_message(
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SendLastWillMessageRequest,
) -> Result<SendLastWillMessageReply, CommonError> {
    let request_data = SendLastWillMessageRequest::encode_to_vec(&request);
    match retry_call(
        MqttBrokerService::Inner,
        MqttBrokerInterface::SendLastWillMessage,
        client_pool,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match SendLastWillMessageReply::decode(data.as_ref()) {
            Ok(da) => Ok(da),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        },
        Err(e) => Err(e),
    }
}
