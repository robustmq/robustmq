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

use crate::poll::ClientPool;
use common_base::error::common::CommonError;
use prost::Message as _;
use protocol::broker_server::generate::placement::{
    CommonReply, DeleteSessionRequest, SendLastWillMessageRequest, UpdateCacheRequest
};
use std::sync::Arc;

use super::{retry_call, MQTTBrokerInterface, MQTTBrokerService};

pub async fn broker_mqtt_delete_session(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: DeleteSessionRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = DeleteSessionRequest::encode_to_vec(&request);
    match retry_call(
        MQTTBrokerService::Mqtt,
        MQTTBrokerInterface::DeleteSession,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}

pub async fn broker_mqtt_update_cache(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: UpdateCacheRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = UpdateCacheRequest::encode_to_vec(&request);
    match retry_call(
        MQTTBrokerService::Mqtt,
        MQTTBrokerInterface::UpdateCache,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}


pub async fn send_last_will_message(
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: SendLastWillMessageRequest,
) -> Result<CommonReply, CommonError> {
    let request_data = SendLastWillMessageRequest::encode_to_vec(&request);
    match retry_call(
        MQTTBrokerService::Mqtt,
        MQTTBrokerInterface::SendLastWillMessage,
        client_poll,
        addrs,
        request_data,
    )
    .await
    {
        Ok(data) => match CommonReply::decode(data.as_ref()) {
            Ok(da) => return Ok(da),
            Err(e) => return Err(CommonError::CommmonError(e.to_string())),
        },
        Err(e) => {
            return Err(e);
        }
    }
}
