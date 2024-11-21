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
use std::time::Duration;

use admin::admin_interface_call;
use common_base::error::common::CommonError;
use log::error;
use placement::placement_interface_call;
use tokio::time::sleep;

use crate::mqtt::connection::connection_interface_call;
use crate::pool::ClientPool;
use crate::{retry_sleep_time, retry_times};

// modules(journal engine) -> service(admin) -> interface(list_user)
// modules(journal engine) -> interface(service functions) -> service(admin)  -> functions



#[derive(Clone)]
pub enum MqttBrokerService {
    Placement,
    Admin,
    Connection,
}

#[derive(Clone, Debug)]
pub enum MqttBrokerPlacementInterface {
    // placement
    DeleteSession,
    UpdateCache,
    SendLastWillMessage,

    // admin service  functions
    ClusterStatus,
    ListUser,
    CreateUser,
    DeleteUser,

    // connection
    ListConnection,
}

pub mod admin;
pub mod connection;
pub mod placement;

async fn retry_call(
    service: MqttBrokerService,
    interface: MqttBrokerPlacementInterface,
    client_pool: Arc<ClientPool>,
    addrs: Vec<String>,
    request: Vec<u8>,
) -> Result<Vec<u8>, CommonError> {
    if addrs.is_empty() {
        return Err(CommonError::CommonError(
            "Call address list cannot be empty".to_string(),
        ));
    }
    let mut times = 1;
    loop {
        let index = times % addrs.len();
        let addr = addrs.get(index).unwrap().clone();
        let result = match service {
            MqttBrokerService::Placement => {
                placement_interface_call(
                    interface.clone(),
                    client_pool.clone(),
                    addr,
                    request.clone(),
                )
                .await
            }
            MqttBrokerService::Admin => {
                admin_interface_call(
                    interface.clone(),
                    client_pool.clone(),
                    addr,
                    request.clone(),
                )
                .await
            }
            MqttBrokerService::Connection => {
                connection_interface_call(
                    interface.clone(),
                    client_pool.clone(),
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
                times += 1;
            }
        }
        sleep(Duration::from_secs(retry_sleep_time(times))).await;
    }
}

#[cfg(test)]
mod tests {}
