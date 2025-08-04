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

use std::{sync::Arc, time::Duration};

use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use protocol::placement_center::placement_center_mqtt::ConnectorHeartbeatRaw;
use tokio::{select, sync::broadcast, time::sleep};
use tracing::{error, info};

use crate::storage::connector::ConnectorStorage;

use super::manager::ConnectorManager;

pub async fn start_connector_report_heartbeat_thread(
    client_pool: Arc<ClientPool>,
    connector_manager: Arc<ConnectorManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let mut recv = stop_send.subscribe();
    loop {
        select! {
            val = recv.recv() =>{
                if let Ok(flag) = val {
                    if flag {
                        info!("{}","Connector heartbeat report thread exited successfully");
                        break;
                    }
                }
            }
            _ = report_heartbeat(
                &client_pool,
                &connector_manager
            ) => {
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn report_heartbeat(
    client_pool: &Arc<ClientPool>,
    connector_manager: &Arc<ConnectorManager>,
) {
    let storage = ConnectorStorage::new(client_pool.clone());
    let conf = broker_config();
    let mut heatbeats = Vec::new();

    for (connector_name, heartbeat_time) in connector_manager.connector_heartbeat.clone() {
        heatbeats.push(ConnectorHeartbeatRaw {
            connector_name,
            heartbeat_time,
            broker_id: conf.broker_id,
        });
    }

    if let Err(e) = storage.connector_heartbeat(heatbeats).await {
        error!("report connector heartbeat error:{}", e);
    }
}
