use self::inner::inner_get_share_sub_leader;
use super::PlacementCenterInterface;
use crate::poll::ClientPool;
use common_base::errors::RobustMQError;
use inner::{
    inner_create_session, inner_create_topic, inner_create_user, inner_delete_session,
    inner_delete_topic, inner_delete_user, inner_list_session, inner_list_topic, inner_list_user,
    inner_set_topic_retain_message,
};
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
    match client_poll.placement_center_mqtt_services_client(addr).await {
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
                    inner_get_share_sub_leader(client, request.clone()).await
                }
                PlacementCenterInterface::ListUser => {
                    inner_list_user(client, request.clone()).await
                }
                PlacementCenterInterface::CreateUser => {
                    inner_create_user(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteUser => {
                    inner_delete_user(client, request.clone()).await
                }
                PlacementCenterInterface::ListTopic => {
                    inner_list_topic(client, request.clone()).await
                }
                PlacementCenterInterface::CreateTopic => {
                    inner_create_topic(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteTopic => {
                    inner_delete_topic(client, request.clone()).await
                }
                PlacementCenterInterface::SetTopicRetainMessage => {
                    inner_set_topic_retain_message(client, request.clone()).await
                }
                PlacementCenterInterface::ListSession => {
                    inner_list_session(client, request.clone()).await
                }
                PlacementCenterInterface::CreateSession => {
                    inner_create_session(client, request.clone()).await
                }
                PlacementCenterInterface::DeleteSession => {
                    inner_delete_session(client, request.clone()).await
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
