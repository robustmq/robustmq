use common_base::errors::RobustMQError;
use inner::{inner_delete_resource_config, inner_get_resource_config, inner_set_resource_config};
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
) -> Result<Vec<u8>, RobustMQError> {
    match placement_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
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
                _ => {
                    return Err(RobustMQError::CommmonError(format!(
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
) -> Result<PlacementCenterServiceClient<Channel>, RobustMQError> {
    match client_poll.get_placement_center_inner_services_client(addr).await {
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
    type Error = RobustMQError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match PlacementCenterServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(RobustMQError::CommmonError(format!(
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
