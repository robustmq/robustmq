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
use protocol::placement_center::placement_center_journal::engine_service_client::EngineServiceClient;
use protocol::placement_center::placement_center_journal::{
    CreateNextSegmentReply, CreateNextSegmentRequest, CreateShardReply, CreateShardRequest,
    DeleteSegmentReply, DeleteSegmentRequest, DeleteShardReply, DeleteShardRequest,
    ListSegmentReply, ListSegmentRequest, ListShardReply, ListShardRequest,
};
use tonic::transport::Channel;

use super::PlacementCenterInterface;
use crate::poll::ClientPool;

pub mod call;

pub async fn journal_interface_call(
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match journal_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                PlacementCenterInterface::ListShard => {
                    client_call(
                        client,
                        request.clone(),
                        |data| ListShardRequest::decode(data),
                        |mut client, request| async move { client.list_shard(request).await },
                        ListShardReply::encode_to_vec,
                    )
                    .await
                }
                PlacementCenterInterface::CreateShard => {
                    client_call(
                        client,
                        request.clone(),
                        |data| CreateShardRequest::decode(data),
                        |mut client, request| async move { client.create_shard(request).await },
                        CreateShardReply::encode_to_vec,
                    )
                    .await
                }
                PlacementCenterInterface::DeleteShard => {
                    client_call(
                        client,
                        request.clone(),
                        |data| DeleteShardRequest::decode(data),
                        |mut client, request| async move { client.delete_shard(request).await },
                        DeleteShardReply::encode_to_vec,
                    )
                    .await
                }

                PlacementCenterInterface::ListSegment => {
                    client_call(
                        client,
                        request.clone(),
                        |data| ListSegmentRequest::decode(data),
                        |mut client, request| async move { client.list_segment(request).await },
                        ListSegmentReply::encode_to_vec,
                    )
                    .await
                }

                PlacementCenterInterface::CreateSegment => client_call(
                    client,
                    request.clone(),
                    |data| CreateNextSegmentRequest::decode(data),
                    |mut client, request| async move { client.create_next_segment(request).await },
                    CreateNextSegmentReply::encode_to_vec,
                )
                .await,
                PlacementCenterInterface::DeleteSegment => {
                    client_call(
                        client,
                        request.clone(),
                        |data| DeleteSegmentRequest::decode(data),
                        |mut client, request| async move { client.delete_segment(request).await },
                        DeleteSegmentReply::encode_to_vec,
                    )
                    .await
                }
                _ => {
                    return Err(CommonError::CommmonError(format!(
                        "journal service does not support service interfaces [{:?}]",
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

pub async fn journal_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<JournalServiceManager>, CommonError> {
    match client_poll
        .placement_center_journal_services_client(addr)
        .await
    {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

#[derive(Clone)]
pub struct JournalServiceManager {
    pub addr: String,
}

impl JournalServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for JournalServiceManager {
    type Connection = EngineServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match EngineServiceClient::connect(format!("http://{}", self.addr.clone())).await {
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
    client: Connection<JournalServiceManager>,
    request: Vec<u8>,
    decode_fn: DecodeFunction,
    client_fn: ClientFunction,
    encode_fn: EncodeFunction,
) -> Result<Vec<u8>, CommonError>
where
    R: prost::Message + Default,
    Resp: prost::Message,
    DecodeFunction: FnOnce(&[u8]) -> Result<R, prost::DecodeError>,
    ClientFunction: FnOnce(Connection<JournalServiceManager>, R) -> Fut,
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
