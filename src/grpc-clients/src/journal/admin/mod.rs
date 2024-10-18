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
use protocol::journal_server::journal_admin::journal_server_admin_service_client::JournalServerAdminServiceClient;
use tonic::transport::Channel;

use super::JournalEngineInterface;
use crate::poll::ClientPool;

pub mod call;

pub(crate) async fn admin_interface_call(
    _interface: JournalEngineInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    _request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match admin_client(client_poll.clone(), addr.clone()).await {
        Ok(_client) => Ok(Vec::new()),
        Err(e) => Err(e),
    }
}

async fn admin_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<JournalAdminServiceManager>, CommonError> {
    match client_poll.journal_admin_services_client(addr).await {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

#[derive(Clone)]
pub struct JournalAdminServiceManager {
    pub addr: String,
}

impl JournalAdminServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for JournalAdminServiceManager {
    type Connection = JournalServerAdminServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match JournalServerAdminServiceClient::connect(format!("http://{}", self.addr.clone()))
            .await
        {
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

pub(crate) async fn _client_call<R, Resp, ClientFunction, Fut, DecodeFunction, EncodeFunction>(
    client: Connection<JournalAdminServiceManager>,
    request: Vec<u8>,
    decode_fn: DecodeFunction,
    client_fn: ClientFunction,
    encode_fn: EncodeFunction,
) -> Result<Vec<u8>, CommonError>
where
    R: prost::Message + Default,
    Resp: prost::Message,
    DecodeFunction: FnOnce(&[u8]) -> Result<R, prost::DecodeError>,
    ClientFunction: FnOnce(Connection<JournalAdminServiceManager>, R) -> Fut,
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
