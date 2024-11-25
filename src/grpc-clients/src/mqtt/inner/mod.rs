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
use mobc::{Connection, Manager};
use prost::Message;
use protocol::broker_mqtt::broker_mqtt_inner::mqtt_broker_inner_service_client::MqttBrokerInnerServiceClient;
use protocol::broker_mqtt::broker_mqtt_inner::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateCacheReply, UpdateCacheRequest,
};
use tonic::transport::Channel;

use super::MqttBrokerInterface;
use crate::pool::ClientPool;

pub mod call;

async fn placement_client(
    client_pool: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<MqttBrokerInnerServiceManager>, CommonError> {
    match client_pool.mqtt_broker_mqtt_services_client(addr).await {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

pub(crate) async fn placement_interface_call(
    interface: MqttBrokerInterface,
    client_pool: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match placement_client(client_pool.clone(), addr.clone()).await {
        Ok(client) => {
            let result =
                match interface {
                    MqttBrokerInterface::DeleteSession => client_call(
                        client,
                        request.clone(),
                        |data| DeleteSessionRequest::decode(data),
                        |mut client, request| async move { client.delete_session(request).await },
                        DeleteSessionReply::encode_to_vec,
                    )
                    .await,
                    MqttBrokerInterface::UpdateCache => {
                        client_call(
                            client,
                            request.clone(),
                            |data| UpdateCacheRequest::decode(data),
                            |mut client, request| async move { client.update_cache(request).await },
                            UpdateCacheReply::encode_to_vec,
                        )
                        .await
                    }
                    MqttBrokerInterface::SendLastWillMessage => {
                        client_call(
                            client,
                            request.clone(),
                            |data| SendLastWillMessageRequest::decode(data),
                            |mut client, request| async move {
                                client.send_last_will_message(request).await
                            },
                            SendLastWillMessageReply::encode_to_vec,
                        )
                        .await
                    }
                    _ => {
                        return Err(CommonError::CommonError(format!(
                            "kv service does not support service interfaces [{:?}]",
                            interface
                        )))
                    }
                };
            match result {
                Ok(data) => Ok(data),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

#[derive(Clone)]
pub struct MqttBrokerInnerServiceManager {
    pub addr: String,
}

impl MqttBrokerInnerServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for MqttBrokerInnerServiceManager {
    type Connection = MqttBrokerInnerServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match MqttBrokerInnerServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(CommonError::CommonError(format!(
                    "{},{}",
                    err,
                    self.addr.clone()
                )))
            }
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

pub(crate) async fn client_call<R, Resp, ClientFunction, Fut, DecodeFunction, EncodeFunction>(
    client: Connection<MqttBrokerInnerServiceManager>,
    request: Vec<u8>,
    decode_fn: DecodeFunction,
    client_fn: ClientFunction,
    encode_fn: EncodeFunction,
) -> Result<Vec<u8>, CommonError>
where
    R: prost::Message + Default,
    Resp: prost::Message,
    DecodeFunction: FnOnce(&[u8]) -> Result<R, prost::DecodeError>,
    ClientFunction: FnOnce(Connection<MqttBrokerInnerServiceManager>, R) -> Fut,
    Fut: std::future::Future<Output = Result<tonic::Response<Resp>, tonic::Status>>,
    EncodeFunction: FnOnce(&Resp) -> Vec<u8>,
{
    match decode_fn(request.as_ref()) {
        Ok(decoded_request) => match client_fn(client, decoded_request).await {
            Ok(result) => Ok(encode_fn(&result.into_inner())),
            Err(e) => Err(CommonError::GrpcServerStatus(e)),
        },
        Err(e) => Err(CommonError::CommonError(e.to_string())),
    }
}
