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

use crate::macros::impl_retriable_request;
use common_base::error::common::CommonError;
use mobc::Manager;
use protocol::broker::broker_common::{
    broker_common_service_client::BrokerCommonServiceClient, UpdateCacheReply, UpdateCacheRequest,
};
use tonic::transport::Channel;
use tracing::{debug, error, info, warn};

pub mod call;

#[derive(Clone)]
pub struct BrokerCommonServiceManager {
    pub addr: String,
}

impl BrokerCommonServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for BrokerCommonServiceManager {
    type Connection = BrokerCommonServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let url = format!("http://{}", self.addr);
        debug!(
            "Attempting to connect to MQTT Broker at {} (url: {})",
            self.addr, url
        );

        let start = std::time::Instant::now();
        match BrokerCommonServiceClient::connect(url.clone()).await {
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
        debug!("Checking Broker Common connection health for {}", self.addr);
        Ok(conn)
    }
}

impl_retriable_request!(
    UpdateCacheRequest,
    BrokerCommonServiceClient<Channel>,
    UpdateCacheReply,
    broker_common_services_client,
    update_cache,
    "MqttBrokerInnerService",
    "UpdateCache"
);
