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

use crate::{poll::ClientPool, retry_sleep_time, retry_times};
use common_base::error::robustmq::RobustMQError;
use inner::{inner_delete_session, inner_send_last_will_message, inner_update_cache};
use log::error;
use mobc::Manager;
use protocol::broker_server::generate::mqtt::mqtt_broker_service_client::MqttBrokerServiceClient;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use tonic::transport::Channel;

#[derive(Clone)]
pub enum MQTTBrokerService {
    Mqtt,
}

#[derive(Clone, Debug)]
pub enum MQTTBrokerInterface {
    DeleteSession,
    UpdateCache,
    SendLastWillMessage,
}

pub mod call;
pub mod inner;

async fn retry_call(
    service: MQTTBrokerService,
    interface: MQTTBrokerInterface,
    client_poll: Arc<ClientPool>,
    addrs: Vec<String>,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    let mut times = 1;
    loop {
        let index = times % addrs.len();
        let addr = addrs.get(index).unwrap().clone();
        let result = match service {
            MQTTBrokerService::Mqtt => {
                mqtt_interface_call(
                    interface.clone(),
                    client_poll.clone(),
                    addr,
                    request.clone(),
                )
                .await
            }
        };

        match result {
            Ok(data) => {
                return Ok(data);
            }
            Err(e) => {
                error!("{}", e);
                if times > retry_times() {
                    return Err(e);
                }
                times = times + 1;
            }
        }
        sleep(Duration::from_secs(retry_sleep_time(times) as u64)).await;
    }
}

async fn mqtt_client(
    client_poll: Arc<ClientPool>,
    addr: String,
) -> Result<MqttBrokerServiceClient<Channel>, RobustMQError> {
    match client_poll.mqtt_broker_mqtt_services_client(addr).await {
        Ok(client) => {
            return Ok(client);
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub(crate) async fn mqtt_interface_call(
    interface: MQTTBrokerInterface,
    client_poll: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, RobustMQError> {
    match mqtt_client(client_poll.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                MQTTBrokerInterface::DeleteSession => inner_delete_session(client, request).await,
                MQTTBrokerInterface::UpdateCache => inner_update_cache(client, request).await,
                MQTTBrokerInterface::SendLastWillMessage => {
                    inner_send_last_will_message(client, request).await
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
pub(crate) struct MqttBrokerMqttServiceManager {
    pub addr: String,
}

impl MqttBrokerMqttServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for MqttBrokerMqttServiceManager {
    type Connection = MqttBrokerServiceClient<Channel>;
    type Error = RobustMQError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match MqttBrokerServiceClient::connect(format!("http://{}", self.addr.clone())).await {
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

#[cfg(test)]
mod tests {}
