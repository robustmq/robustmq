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
use protocol::broker::broker_mqtt::mqtt_broker_inner_service_client::MqttBrokerInnerServiceClient;
use protocol::broker::broker_mqtt::{
    DeleteSessionReply, DeleteSessionRequest, SendLastWillMessageReply, SendLastWillMessageRequest,
    UpdateMqttCacheReply, UpdateMqttCacheRequest,
};
use tonic::transport::Channel;
use tracing::{debug, error, info, warn};

use crate::macros::impl_retriable_request;

pub mod call;

#[derive(Clone)]
pub struct BrokerMQTTServiceManager {
    pub addr: String,
}

impl BrokerMQTTServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for BrokerMQTTServiceManager {
    type Connection = MqttBrokerInnerServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let url = format!("http://{}", self.addr);
        debug!(
            "Attempting to connect to MQTT Broker at {} (url: {})",
            self.addr, url
        );

        let start = std::time::Instant::now();
        match MqttBrokerInnerServiceClient::connect(url.clone()).await {
            Ok(client) => {
                let duration = start.elapsed();
                info!(
                    "Successfully connected to MQTT Broker at {} (took {:?})",
                    self.addr, duration
                );

                if duration.as_secs() > 2 {
                    warn!(
                        "Connection to MQTT Broker at {} took longer than expected: {:?}",
                        self.addr, duration
                    );
                }

                return Ok(client);
            }
            Err(err) => {
                let duration = start.elapsed();
                error!(
                    "Failed to connect to MQTT Broker at {} after {:?}: {} (full error: {:?})",
                    self.addr, duration, err, err
                );
                return Err(CommonError::CommonError(format!(
                    "Failed to connect to MQTT Broker at {} after {:?}: {}",
                    self.addr, duration, err
                )));
            }
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        // Basic connection validation
        // Note: gRPC connections are lazy, so we just validate the connection object exists
        // The actual health check happens when the first RPC is made
        debug!("Checking MQTT Broker connection health for {}", self.addr);
        Ok(conn)
    }
}

impl_retriable_request!(
    DeleteSessionRequest,
    MqttBrokerInnerServiceClient<Channel>,
    DeleteSessionReply,
    mqtt_broker_mqtt_services_client,
    delete_session,
    "MqttBrokerInnerService",
    "DeleteSession"
);

impl_retriable_request!(
    UpdateMqttCacheRequest,
    MqttBrokerInnerServiceClient<Channel>,
    UpdateMqttCacheReply,
    mqtt_broker_mqtt_services_client,
    update_cache,
    "MqttBrokerInnerService",
    "UpdateCache"
);

impl_retriable_request!(
    SendLastWillMessageRequest,
    MqttBrokerInnerServiceClient<Channel>,
    SendLastWillMessageReply,
    mqtt_broker_mqtt_services_client,
    send_last_will_message,
    "MqttBrokerInnerService",
    "SendLastWillMessage"
);
