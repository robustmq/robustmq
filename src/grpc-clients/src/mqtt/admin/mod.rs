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
use inner::{inner_cluster_status, inner_create_user, inner_delete_user, inner_list_user};
use mobc::{Connection, Manager};
use protocol::broker_mqtt::broker_mqtt_admin::mqtt_broker_admin_service_client::MqttBrokerAdminServiceClient;
use tonic::transport::Channel;

use super::MqttBrokerPlacementInterface;
use crate::pool::ClientPool;

pub mod call;
pub mod inner;

async fn admin_client(
    client_pool: Arc<ClientPool>,
    addr: String,
) -> Result<Connection<MqttBrokerAdminServiceManager>, CommonError> {
    match client_pool.mqtt_broker_admin_services_client(addr).await {
        Ok(client) => Ok(client),
        Err(e) => Err(e),
    }
}

pub(crate) async fn admin_interface_call(
    interface: MqttBrokerPlacementInterface,
    client_pool: Arc<ClientPool>,
    addr: String,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    match admin_client(client_pool.clone(), addr.clone()).await {
        Ok(client) => {
            let result = match interface {
                MqttBrokerPlacementInterface::ClusterStatus => {
                    inner_cluster_status(client, request).await
                }
                MqttBrokerPlacementInterface::ListUser => inner_list_user(client, request).await,
                MqttBrokerPlacementInterface::CreateUser => {
                    inner_create_user(client, request).await
                }
                MqttBrokerPlacementInterface::DeleteUser => {
                    inner_delete_user(client, request).await
                }
                _ => {
                    return Err(CommonError::CommmonError(format!(
                        "admin service does not support service interfaces [{:?}]",
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
pub struct MqttBrokerAdminServiceManager {
    pub addr: String,
}

impl MqttBrokerAdminServiceManager {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}
#[tonic::async_trait]
impl Manager for MqttBrokerAdminServiceManager {
    type Connection = MqttBrokerAdminServiceClient<Channel>;
    type Error = CommonError;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        match MqttBrokerAdminServiceClient::connect(format!("http://{}", self.addr.clone())).await {
            Ok(client) => {
                return Ok(client);
            }
            Err(err) => {
                return Err(CommonError::CommmonError(format!(
                    "{},{}",
                    err,
                    self.addr.clone()
                )))
            }
        };
    }

    async fn check(&self, conn: Self::Connection) -> Result<Self::Connection, Self::Error> {
        Ok(conn)
    }
}
