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
use protocol::placement_center::generate::openraft::open_raft_service_client::OpenRaftServiceClient;
use protocol::placement_center::generate::openraft::{
    AppendReply, AppendRequest, SnapshotReply, SnapshotRequest, VoteReply, VoteRequest,
};
use tonic::transport::Channel;

use super::PlacementCenterInterface;
use crate::poll::ClientPool;

pub mod call;

pub(crate) async fn openraft_interface_call(
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match openraft_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                PlacementCenterInterface::Vote => {
                    client_call(
                        client,
                        request.clone(),
                        |data| VoteRequest::decode(data),
                        |mut client, request| async move { client.vote(request).await },
                        VoteReply::encode_to_vec,
                    )
                    .await
                }
                PlacementCenterInterface::Append => {
                    client_call(
                        client,
                        request.clone(),
                        |data| AppendRequest::decode(data),
                        |mut client, request| async move { client.append(request).await },
                        AppendReply::encode_to_vec,
                    )
                    .await
                }
                PlacementCenterInterface::Snapshot => {
                    client_call(
                        client,
                        request.clone(),
                        |data| SnapshotRequest::decode(data),
                        |mut client, request| async move { client.snapshot(request).await },
                        SnapshotReply::encode_to_vec,
                    )
                    .await
                }
                _ => {
                    return Err(CommonError::CommmonError(format!(
                        "openraft service does not support service interfaces [{:?}]",
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

async fn openraft_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<OpenRaftServiceManager>, CommonError> {
    match client_poll
        .placement_center_openraft_services_client(addr)
        .await
    {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

#[derive(Clone)]
pub struct OpenRaftServiceManager {
    pub addr: String,
}

impl OpenRaftServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for OpenRaftServiceManager {
    type Connection = OpenRaftServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let addr = format!("http://{}", self.addr.clone());

        match OpenRaftServiceClient::connect(addr.clone()).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => return Err(CommonError::CommmonError(format!("{},{}", err, addr))),
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

pub(crate) async fn client_call<R, Resp, ClientFunction, Fut, DecodeFunction, EncodeFunction>(
    client: Connection<OpenRaftServiceManager>,
    request: Vec<u8>,
    decode_fn: DecodeFunction,
    client_fn: ClientFunction,
    encode_fn: EncodeFunction,
) -> Result<Vec<u8>, CommonError>
where
    R: prost::Message + Default,
    Resp: prost::Message,
    DecodeFunction: FnOnce(&[u8]) -> Result<R, prost::DecodeError>,
    ClientFunction: FnOnce(Connection<OpenRaftServiceManager>, R) -> Fut,
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
