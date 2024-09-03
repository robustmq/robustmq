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

use common_base::error::common::CommonError;
use inner::{
    inner_delete_idempotent, inner_delete_resource_config, inner_exist_idempotent, inner_get_resource_config, inner_node_list, inner_set_idempotent, inner_set_resource_config
};
use mobc::Manager;
use protocol::placement_center::generate::placement::placement_center_service_client::PlacementCenterServiceClient;
use std::sync::Arc;
use tonic::transport::Channel;

use crate::poll::ClientPool;

use self::inner::{
    inner_heartbeat, inner_register_node, inner_send_raft_conf_change, inner_send_raft_message,
    inner_unregister_node,
};

use super::PlacementCenterInterface;

pub mod call;
mod inner;

pub(crate) async fn placement_interface_call(
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match placement_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                PlacementCenterInterface::ListNode => {
                    inner_node_list(client, request.clone()).await
                }
                PlacementCenterInterface::RegisterNode => {
                    inner_register_node(client, request.clone()).await
                }
                PlacementCenterInterface::UnRegisterNode => {
                    inner_unregister_node(client, request.clone()).await
                }
                PlacementCenterInterface::Heartbeat => {
                    inner_heartbeat(client, request.clone()).await
                }
                PlacementCenterInterface::SendRaftMessage => {
                    inner_send_raft_message(client, request.clone()).await
                }
                PlacementCenterInterface::SendRaftConfChange => {
                    inner_send_raft_conf_change(client, request.clone()).await
                }
                PlacementCenterInterface::SetReourceConfig => {
                    inner_set_resource_config(client, request.clone()).await
                }
                PlacementCenterInterface::GetReourceConfig => {
                    inner_get_resource_config(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteReourceConfig => {
                    inner_delete_resource_config(client, request.clone()).await
                }
                PlacementCenterInterface::SetIdempotentData => {
                    inner_set_idempotent(client, request.clone()).await
                }
                PlacementCenterInterface::ExistsIdempotentData => {
                    inner_exist_idempotent(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteIdempotentData => {
                    inner_delete_idempotent(client, request.clone()).await
                }
                _ => {
                    return Err(CommonError::CommmonError(format!(
                        "placement service does not support service interfaces [{:?}]",
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

async fn placement_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<PlacementCenterServiceClient<Channel>, CommonError> {
    match client_poll
        .get_placement_center_inner_services_client(addr)
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

pub(crate) struct PlacementServiceManager {
    pub addr: String,
}

impl PlacementServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for PlacementServiceManager {
    type Connection = PlacementCenterServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match PlacementCenterServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(CommonError::CommmonError(format!(
                    "manager connect error:{},{}",
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
