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

use super::PlacementCenterInterface;
use crate::poll::ClientPool;
use common_base::error::common::CommonError;
use inner::{inner_append, inner_snapshot, inner_vote};
use mobc::{Connection, Manager};
use protocol::placement_center::generate::openraft::open_raft_service_client::OpenRaftServiceClient;
use std::sync::Arc;
use tonic::transport::Channel;

pub mod call;
mod inner;

pub(crate) async fn openraft_interface_call(
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match openraft_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                PlacementCenterInterface::Vote => inner_vote(client, request.clone()).await,
                PlacementCenterInterface::Append => inner_append(client, request.clone()).await,
                PlacementCenterInterface::Snapshot => inner_snapshot(client, request.clone()).await,
                _ => {
                    return Err(CommonError::CommmonError(format!(
                        "openraft service does not support service interfaces [{:?}]",
                        interface
                    )))
                }
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

async fn openraft_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<OpenRaftServiceManager>, CommonError> {
    match client_poll
        .placement_center_openraft_services_client(addr)
        .await
    {
        Ok(client) => {
            return Ok(client);
        }
        Err(e) => {
            return Err(e);
        }
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
        let mut addr = format!("http://{}", self.addr.clone());

        match OpenRaftServiceClient::connect(addr.clone()).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(CommonError::CommmonError(format!(
                    "{},{}",
                    err.to_string(),
                    addr
                )))
            }
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}
