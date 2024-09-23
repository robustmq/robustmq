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
use self::inner::{inner_delete, inner_exists, inner_get, inner_set};
use super::PlacementCenterInterface;
use common_base::error::common::CommonError;
use mobc::{Connection, Manager};
use protocol::placement_center::generate::kv::kv_service_client::KvServiceClient;
use std::sync::Arc;
use tonic::transport::Channel;

pub mod call;
mod inner;

async fn kv_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<KvServiceManager>, CommonError> {
    match client_poll.placement_center_kv_services_client(addr).await {
        Ok(client) => {
            return Ok(client);
        }
        Err(e) => {
            return Err(e);
        }
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
                PlacementCenterInterface::Set => inner_set(client, request.clone()).await,
                PlacementCenterInterface::Delete => inner_delete(client, request.clone()).await,
                PlacementCenterInterface::Get => inner_get(client, request.clone()).await,
                PlacementCenterInterface::Exists => inner_exists(client, request.clone()).await,
                _ => return Err(CommonError::CommmonError(format!(
                    "kv service does not support service interfaces [{:?}]",
                    interface
                ))),
            };
            match result {
                Ok(data) => return Ok(data),
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Err(e) => {
            return Err(e);
        }
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
            Err(err) => return Err(CommonError::CommmonError(format!(
                "{},{}",
                err.to_string(),
                self.addr.clone()
            ))),
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

#[cfg(test)]
mod tests {}
