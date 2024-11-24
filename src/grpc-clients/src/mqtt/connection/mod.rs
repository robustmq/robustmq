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

use std::sync::Arc;

use common_base::error::common::CommonError;
use mobc::{Connection, Manager};
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_client::MqttBrokerAdminServiceClient;
use tonic::transport::Channel;

use crate::mqtt::connection::inner::inner_list_connection;
use crate::mqtt::MqttBrokerPlacementInterface;
use crate::pool::ClientPool;

pub mod call;
pub mod inner;

async fn connection_client(
    client_pool: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<MqttBrokerConnectionServiceManager>, CommonError> {
    match client_pool
        .mqtt_broker_connection_services_client(addr)
        .await
    {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

pub(crate) async fn connection_interface_call(
    interface: MqttBrokerPlacementInterface,
    client_pool: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match connection_client(client_pool.clone(), addr).await {
        Ok(client) => {
            let result = match interface {
                MqttBrokerPlacementInterface::ListConnection => {
                    inner_list_connection(client, request).await
                }
                _ => {
                    return Err(CommonError::CommonError(format!(
                        "connection service does not support service interfaces [{:?}]",
                        interface
                    )))
                }
            };

            match result {
                Ok(data) => Ok(data),
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

#[derive(Clone)]
pub struct MqttBrokerConnectionServiceManager {
    pub addr: String,
}

impl MqttBrokerConnectionServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[tonic::async_trait]
impl Manager for MqttBrokerConnectionServiceManager {
    type Connection = MqttBrokerAdminServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match MqttBrokerAdminServiceClient::connect(format!("http://{}", self.addr.clone()))
            .await
        {
            Ok(client) => Ok(client),
            Err(e) => Err(CommonError::CommonError(format!(
                "{},{}",
                e,
                self.addr.clone()
            ))),
        }
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}
