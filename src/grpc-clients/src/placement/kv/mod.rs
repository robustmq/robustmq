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
use protocol::placement_center::placement_center_kv::kv_service_client::KvServiceClient;
use protocol::placement_center::placement_center_kv::{
    DeleteReply, DeleteRequest, ExistsReply, ExistsRequest, GetReply, GetRequest, SetReply, SetRequest
};
use tonic::transport::Channel;

use super::PlacementCenterInterface;
use crate::poll::ClientPool;

pub mod call;

async fn kv_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<KvServiceManager>, CommonError> {
    match client_poll.placement_center_kv_services_client(addr).await {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

pub(crate) async fn kv_interface_call(
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match kv_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                PlacementCenterInterface::Set => {
                    client_call(
                        client,
                        request.clone(),
                        |data| SetRequest::decode(data),
                        |mut client, request| async move { client.set(request).await },
                        SetReply::encode_to_vec,
                    )
                    .await
                }
                PlacementCenterInterface::Delete => {
                    client_call(
                        client,
                        request.clone(),
                        |data| DeleteRequest::decode(data),
                        |mut client, request| async move { client.delete(request).await },
                        DeleteReply::encode_to_vec,
                    )
                    .await
                }
                PlacementCenterInterface::Get => {
                    client_call(
                        client,
                        request.clone(),
                        |data| GetRequest::decode(data),
                        |mut client, request| async move { client.get(request).await },
                        GetReply::encode_to_vec,
                    )
                    .await
                }
                PlacementCenterInterface::Exists => {
                    client_call(
                        client,
                        request.clone(),
                        |data| ExistsRequest::decode(data),
                        |mut client, request| async move { client.exists(request).await },
                        ExistsReply::encode_to_vec,
                    )
                    .await
                }
                _ => {
                    return Err(CommonError::CommmonError(format!(
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
pub struct KvServiceManager {
    pub addr: String,
}

impl KvServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for KvServiceManager {
    type Connection = KvServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match KvServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(CommonError::CommmonError(format!(
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
    client: Connection<KvServiceManager>,
    request: Vec<u8>,
    decode_fn: DecodeFunction,
    client_fn: ClientFunction,
    encode_fn: EncodeFunction,
) -> Result<Vec<u8>, CommonError>
where
    R: prost::Message + Default,
    Resp: prost::Message,
    DecodeFunction: FnOnce(&[u8]) -> Result<R, prost::DecodeError>,
    ClientFunction: FnOnce(Connection<KvServiceManager>, R) -> Fut,
    Fut: std::future::Future<Output = Result<tonic::Response<Resp>, tonic::Status>>,
    EncodeFunction: FnOnce(&Resp) -> Vec<u8>,
{
    match decode_fn(request.as_ref()) {
        Ok(decoded_request) => match client_fn(client, decoded_request).await {
            Ok(result) => Ok(encode_fn(&result.into_inner())),
            Err(e) => Err(CommonError::GrpcServerStatus(e)),
        },
        Err(e) => Err(CommonError::CommmonError(e.to_string())),
    }
}

#[cfg(test)]
mod tests {}
