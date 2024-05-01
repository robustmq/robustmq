use crate::poll::ClientPool;

use self::inner::{inner_delete_share_sub, inner_get_share_sub};
use super::PlacementCenterInterface;
use common_base::errors::RobustMQError;
use mobc::Manager;
use protocol::placement_center::generate::mqtt::mqtt_service_client::MqttServiceClient;
use std::sync::Arc;
use tonic::transport::Channel;

pub mod call;
mod inner;

async fn mqtt_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<MqttServiceClient<Channel>, RobustMQError> {
    match client_poll.get_mqtt_services_client(addr).await {
        Ok(client) => {
            return Ok(client);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub(crate) async fn mqtt_interface_call(
    interface: PlacementCenterInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match mqtt_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                PlacementCenterInterface::GetShareSub => {
                    inner_get_share_sub(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteShareSub => {
                    inner_delete_share_sub(client, request.clone()).await
                }
                _ => {
                    return Err(RobustMQError::CommmonError(format!(
                        "mqtt service does not support service interfaces [{:?}]",
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

#[derive(Clone)]
pub(crate) struct MQTTServiceManager {
    pub addr: String,
}

impl MQTTServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for MQTTServiceManager {
    type Connection = MqttServiceClient<Channel>;
    type Error = RobustMQError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match MqttServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => return Err(RobustMQError::TonicTransport(err)),
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}