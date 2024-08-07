// Copyright 2023 RobustMQ Team
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

use crate::poll::ClientPool;
use common_base::errors::RobustMQError;
use mobc::Manager;
use protocol::placement_center::generate::journal::engine_service_client::EngineServiceClient;
use tonic::transport::Channel;

use self::inner::{
    inner_create_segment, inner_create_shard, inner_delete_segment, inner_delete_shard,
};
use super::PlacementCenterInterface;

pub mod call;
pub mod inner;

pub async fn journal_interface_call(
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match journal_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                PlacementCenterInterface::CreateShard => {
                    inner_create_shard(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteShard => {
                    inner_delete_shard(client, request.clone()).await
                }
                PlacementCenterInterface::CreateSegment => {
                    inner_create_segment(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteSegment => {
                    inner_delete_segment(client, request.clone()).await
                }
                _ => {
                    return Err(RobustMQError::CommmonError(format!(
                        "journal service does not support service interfaces [{:?}]",
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

pub async fn journal_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<EngineServiceClient<Channel>, RobustMQError> {
    match client_poll.placement_center_journal_services_client(addr).await {
        Ok(client) => {
            return Ok(client);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

#[derive(Clone)]
pub(crate) struct JournalServiceManager {
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
    type Error = RobustMQError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match EngineServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(RobustMQError::CommmonError(format!(
                    "{},{}",
                    err.to_string(),
                    self.addr.clone()
                )))
            }
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}
