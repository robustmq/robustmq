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

use common_base::error::common::CommonError;
use mobc::Manager;
use protocol::broker::broker_mqtt_inner::mqtt_broker_inner_service_client::MqttBrokerInnerServiceClient;
use protocol::broker::broker_mqtt_inner::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateMqttCacheReply, UpdateMqttCacheRequest,
};
use tonic::transport::Channel;
use tracing::{error, info};

use crate::macros::impl_retriable_request;

pub mod call;

#[derive(Clone)]
pub struct MqttBrokerPlacementServiceManager {
    pub addr: String,
}

impl MqttBrokerPlacementServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for MqttBrokerPlacementServiceManager {
    type Connection = MqttBrokerInnerServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let url = format!("http://{}", self.addr);
        match MqttBrokerInnerServiceClient::connect(url.clone()).await {
            Ok(client) => {
                info!("Successfully connected to MQTT Broker at {}", self.addr);
                return Ok(client);
            }
            Err(err) => {
                error!(
                    "Failed to connect to MQTT Broker at {}: {} (full error: {:?})",
                    self.addr, err, err
                );
                return Err(CommonError::CommonError(format!(
                    "Failed to connect to MQTT Broker at {}: {}",
                    self.addr, err
                )));
            }
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}

impl_retriable_request!(
    DeleteSessionRequest,
    MqttBrokerInnerServiceClient<Channel>,
    DeleteSessionReply,
    mqtt_broker_mqtt_services_client,
    delete_session
);

impl_retriable_request!(
    UpdateMqttCacheRequest,
    MqttBrokerInnerServiceClient<Channel>,
    UpdateMqttCacheReply,
    mqtt_broker_mqtt_services_client,
    update_cache
);

impl_retriable_request!(
    SendLastWillMessageRequest,
    MqttBrokerInnerServiceClient<Channel>,
    SendLastWillMessageReply,
    mqtt_broker_mqtt_services_client,
    send_last_will_message
);
