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
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_client::MqttBrokerAdminServiceClient;
use protocol::broker_mqtt::broker_mqtt_admin::{
    ClusterStatusReply, ClusterStatusRequest, CreateUserReply, CreateUserRequest, DeleteUserReply,
    DeleteUserRequest, ListConnectionReply, ListConnectionRequest, ListUserReply, ListUserRequest,
};
use tonic::transport::Channel;

use super::MqttBrokerInterface;
use crate::pool::ClientPool;

pub mod call;

async fn admin_client(
    client_pool: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<MqttBrokerAdminServiceManager>, CommonError> {
    match client_pool.mqtt_broker_admin_services_client(addr).await {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

pub(crate) async fn admin_interface_call(
    interface: MqttBrokerInterface,
    client_pool: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match admin_client(client_pool.clone(), addr.clone()).await {
        Ok(client) => {
            let result =
                match interface {
                    // admin cluster status
                    MqttBrokerInterface::ClusterStatus => client_call(
                        client,
                        request.clone(),
                        |data| ClusterStatusRequest::decode(data),
                        |mut client, request| async move { client.cluster_status(request).await },
                        ClusterStatusReply::encode_to_vec,
                    )
                    .await,

                    // admin user
                    MqttBrokerInterface::ListUser => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ListUserRequest::decode(data),
                            |mut client, request| async move {
                                client.mqtt_broker_list_user(request).await
                            },
                            ListUserReply::encode_to_vec,
                        )
                        .await
                    }

                    MqttBrokerInterface::CreateUser => {
                        client_call(
                            client,
                            request.clone(),
                            |data| CreateUserRequest::decode(data),
                            |mut client, request| async move {
                                client.mqtt_broker_create_user(request).await
                            },
                            CreateUserReply::encode_to_vec,
                        )
                        .await
                    }

                    MqttBrokerInterface::DeleteUser => {
                        client_call(
                            client,
                            request.clone(),
                            |data| DeleteUserRequest::decode(data),
                            |mut client, request| async move {
                                client.mqtt_broker_delete_user(request).await
                            },
                            DeleteUserReply::encode_to_vec,
                        )
                        .await
                    }

                    // admin connection
                    MqttBrokerInterface::ListConnection => {
                        client_call(
                            client,
                            request.clone(),
                            |data| ListConnectionRequest::decode(data),
                            |mut client, request| async move {
                                client.mqtt_broker_list_connection(request).await
                            },
                            ListConnectionReply::encode_to_vec,
                        )
                        .await
                    }
                    _ => {
                        return Err(CommonError::CommonError(format!(
                            "admin service does not support service interfaces [{:?}]",
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
pub struct MqttBrokerAdminServiceManager {
    pub addr: String,
}

impl MqttBrokerAdminServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for MqttBrokerAdminServiceManager {
    type Connection = MqttBrokerAdminServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match MqttBrokerAdminServiceClient::connect(format!("http://{}", self.addr.clone())).await {
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

/// Performs an asynchronous client call to the MQTT broker admin service.
///
/// This function takes a connection to the MQTT broker admin service, a request payload,
/// a decode function to parse the request, a client function to perform the actual call,
/// and an encode function to serialize the response. It returns the encoded response
/// or a common error if any step fails.
///
/// # Parameters
/// - `client`: A connection to the MQTT broker admin service.
/// - `request`: The raw request data as a byte vector.
/// - `decode_fn`: A function that decodes the request data into a Protobuf message.
/// - `client_fn`: A function that takes the client and decoded request to perform the service call.
/// - `encode_fn`: A function that encodes the response Protobuf message into a byte vector.
///
/// # Returns
/// The function returns a `Result` with a byte vector containing the encoded response
/// on success, or a `CommonError` on failure.
///
/// # Type Parameters
/// - `R`: The type of the decoded request message. It must be a Protobuf message and implement `Default`.
/// - `Resp`: The type of the response message. It must be a Protobuf message.
/// - `ClientFunction`: The type of the function that performs the service call. It must return a Future.
/// - `Fut`: The type of the Future returned by `ClientFunction`. It must resolve to a `Result` with a `tonic::Response` containing the response message.
/// - `DecodeFunction`: The type of the function that decodes the request data.
/// - `EncodeFunction`: The type of the function that encodes the response message.
///
/// # Errors
/// This function will return a `CommonError` if:
/// - The request data cannot be decoded.
/// - The service call fails with a gRPC status error.
///
/// # Panics
/// This function does not panic; all errors are handled via the returned `Result`.
pub(crate) async fn client_call<R, Resp, ClientFunction, Fut, DecodeFunction, EncodeFunction>(
    client: Connection<MqttBrokerAdminServiceManager>,
    request: Vec<u8>,
    decode_fn: DecodeFunction,
    client_fn: ClientFunction,
    encode_fn: EncodeFunction,
) -> Result<Vec<u8>, CommonError>
where
    R: prost::Message + Default,
    Resp: prost::Message,
    DecodeFunction: FnOnce(&[u8]) -> Result<R, prost::DecodeError>,
    ClientFunction: FnOnce(Connection<MqttBrokerAdminServiceManager>, R) -> Fut,
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
