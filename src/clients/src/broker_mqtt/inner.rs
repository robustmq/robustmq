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
use prost::Message;
use protocol::broker_server::generate::mqtt::{
    mqtt_broker_service_client::MqttBrokerServiceClient, CommonReply, DeleteSessionRequest,
    SendLastWillMessageRequest, UpdateCacheRequest,
};
use tonic::transport::Channel;

pub(crate) async fn inner_delete_session(
    mut client: MqttBrokerServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match DeleteSessionRequest::decode(request.as_ref()) {
        Ok(request) => match client.delete_session(request).await {
            Ok(result) => {
                return Ok(CommonReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(CommonError::GrpcServerStatus(e)),
        },
        Err(e) => {
            return Err(CommonError::CommmonError(e.to_string()));
        }
    }
}

pub(crate) async fn inner_update_cache(
    mut client: MqttBrokerServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match UpdateCacheRequest::decode(request.as_ref()) {
        Ok(request) => match client.update_cache(request).await {
            Ok(result) => {
                return Ok(CommonReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(CommonError::GrpcServerStatus(e)),
        },
        Err(e) => {
            return Err(CommonError::CommmonError(e.to_string()));
        }
    }
}

pub(crate) async fn inner_send_last_will_message(
    mut client: MqttBrokerServiceClient<Channel>,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match SendLastWillMessageRequest::decode(request.as_ref()) {
        Ok(request) => match client.send_last_will_message(request).await {
            Ok(result) => {
                return Ok(CommonReply::encode_to_vec(&result.into_inner()));
            }
            Err(e) => return Err(CommonError::GrpcServerStatus(e)),
        },
        Err(e) => {
            return Err(CommonError::CommmonError(e.to_string()));
        }
    }
}
