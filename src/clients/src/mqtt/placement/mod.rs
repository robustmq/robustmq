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

use super::MQTTBrokerPlacementInterface;
use crate::poll::ClientPool;
use common_base::error::common::CommonError;
use inner::{inner_delete_session, inner_send_last_will_message, inner_update_cache};
use mobc::Manager;
use protocol::broker_server::generate::placement::mqtt_broker_placement_service_client::MqttBrokerPlacementServiceClient;
use std::sync::Arc;
use tonic::transport::Channel;

pub mod call;
pub mod inner;

async fn placement_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<MqttBrokerPlacementServiceClient<Channel>, CommonError> {
    match client_poll.mqtt_broker_mqtt_services_client(addr).await {
        Ok(client) => {
            return Ok(client);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub(crate) async fn placement_interface_call(
    interface: MQTTBrokerPlacementInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match placement_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                MQTTBrokerPlacementInterface::DeleteSession => {
                    inner_delete_session(client, request).await
                }
                MQTTBrokerPlacementInterface::UpdateCache => {
                    inner_update_cache(client, request).await
                }
                MQTTBrokerPlacementInterface::SendLastWillMessage => {
                    inner_send_last_will_message(client, request).await
                }
                _ => {
                    return Err(CommonError::CommmonError(format!(
                        "kv service does not support service interfaces [{:?}]",
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
pub(crate) struct MqttBrokerPlacementServiceManager {
    pub addr: String,
}

impl MqttBrokerPlacementServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for MqttBrokerPlacementServiceManager {
    type Connection = MqttBrokerPlacementServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match MqttBrokerPlacementServiceClient::connect(format!("http://{}", self.addr.clone()))
            .await
        {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(CommonError::CommmonError(format!(
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
