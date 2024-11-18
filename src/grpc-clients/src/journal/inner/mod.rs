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
use protocol::journal_server::journal_inner::journal_server_inner_service_client::JournalServerInnerServiceClient;
use protocol::journal_server::journal_inner::{
    DeleteSegmentFileReply, DeleteSegmentFileRequest, DeleteShardFileReply, DeleteShardFileRequest,
    GetSegmentDeleteStatusReply, GetSegmentDeleteStatusRequest, GetShardDeleteStatusReply,
    GetShardDeleteStatusRequest, UpdateJournalCacheReply, UpdateJournalCacheRequest,
};
use tonic::transport::Channel;

use super::JournalEngineInterface;
use crate::pool::ClientPool;

pub mod call;

pub(crate) async fn inner_interface_call(
    interface: JournalEngineInterface,
    client_pool: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match inner_client(client_pool.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                JournalEngineInterface::UpdateCache => {
                    client_call(
                        client,
                        request.clone(),
                        |data| UpdateJournalCacheRequest::decode(data),
                        |mut client, request| async move { client.update_cache(request).await },
                        UpdateJournalCacheReply::encode_to_vec,
                    )
                    .await
                }
                JournalEngineInterface::DeleteShardFile => client_call(
                    client,
                    request.clone(),
                    |data| DeleteShardFileRequest::decode(data),
                    |mut client, request| async move { client.delete_shard_file(request).await },
                    DeleteShardFileReply::encode_to_vec,
                )
                .await,
                JournalEngineInterface::GetShardDeleteStatus => {
                    client_call(
                        client,
                        request.clone(),
                        |data| GetShardDeleteStatusRequest::decode(data),
                        |mut client, request| async move {
                            client.get_shard_delete_status(request).await
                        },
                        GetShardDeleteStatusReply::encode_to_vec,
                    )
                    .await
                }
                JournalEngineInterface::DeleteSegmentFile => client_call(
                    client,
                    request.clone(),
                    |data| DeleteSegmentFileRequest::decode(data),
                    |mut client, request| async move { client.delete_segment_file(request).await },
                    DeleteSegmentFileReply::encode_to_vec,
                )
                .await,
                JournalEngineInterface::GetSegmentDeleteStatus => {
                    client_call(
                        client,
                        request.clone(),
                        |data| GetSegmentDeleteStatusRequest::decode(data),
                        |mut client, request| async move {
                            client.get_segment_delete_status(request).await
                        },
                        GetSegmentDeleteStatusReply::encode_to_vec,
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

async fn inner_client(
    client_pool: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<JournalInnerServiceManager>, CommonError> {
    match client_pool.journal_inner_services_client(addr).await {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

#[derive(Clone)]
pub struct JournalInnerServiceManager {
    pub addr: String,
}

impl JournalInnerServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for JournalInnerServiceManager {
    type Connection = JournalServerInnerServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match JournalServerInnerServiceClient::connect(format!("http://{}", self.addr.clone()))
            .await
        {
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
    client: Connection<JournalInnerServiceManager>,
    request: Vec<u8>,
    decode_fn: DecodeFunction,
    client_fn: ClientFunction,
    encode_fn: EncodeFunction,
) -> Result<Vec<u8>, CommonError>
where
    R: prost::Message + Default,
    Resp: prost::Message,
    DecodeFunction: FnOnce(&[u8]) -> Result<R, prost::DecodeError>,
    ClientFunction: FnOnce(Connection<JournalInnerServiceManager>, R) -> Fut,
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
