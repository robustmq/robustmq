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
use protocol::meta::placement_center_mqtt::ConnectorHeartbeatRaw;
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
    let mut heatbeats = Vec::with_capacity(connector_manager.connector_heartbeat.len());

    for entry in connector_manager.connector_heartbeat.iter() {
        heatbeats.push(ConnectorHeartbeatRaw {
            connector_name: entry.key().clone(),
            heartbeat_time: *entry.value(),
            broker_id: conf.broker_id,
        });
    }

    if let Err(e) = storage.connector_heartbeat(heatbeats).await {
        error!("report connector heartbeat error:{}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::manager::ConnectorManager;
    use common_base::tools::unique_id;
    use common_config::{broker::init_broker_conf_by_config, config::BrokerConfig};

    async fn setup() -> (Arc<ClientPool>, Arc<ConnectorManager>) {
        let namespace = unique_id();
        let config = BrokerConfig {
            cluster_name: namespace,
            broker_id: 1,
            ..Default::default()
        };
        init_broker_conf_by_config(config);

        let client_pool = Arc::new(ClientPool::new(1));
        let connector_manager = Arc::new(ConnectorManager::new());

        (client_pool, connector_manager)
    }

    #[tokio::test]
    async fn test_start_connector_report_heartbeat_thread() {
        let (client_pool, connector_manager) = setup().await;
        let (stop_send, _) = broadcast::channel::<bool>(1);

        connector_manager.report_heartbeat("test_connector_1");
        connector_manager.report_heartbeat("test_connector_2");

        let heartbeat_handle = tokio::spawn({
            let client_pool = client_pool.clone();
            let connector_manager = connector_manager.clone();
            let stop_send = stop_send.clone();
            async move {
                start_connector_report_heartbeat_thread(client_pool, connector_manager, stop_send)
                    .await;
            }
        });

        sleep(Duration::from_millis(100)).await;

        stop_send.send(true).unwrap();

        assert!(heartbeat_handle.await.is_ok());
    }

    #[tokio::test]
    async fn test_report_heartbeat() {
        let (client_pool, connector_manager) = setup().await;

        connector_manager.report_heartbeat("test_connector");

        assert!(connector_manager
            .connector_heartbeat
            .contains_key("test_connector"));

        report_heartbeat(&client_pool, &connector_manager).await;

        // Verify that the heartbeat data still exists
        assert!(connector_manager
            .connector_heartbeat
            .contains_key("test_connector"));

        // With no data
        let (client_pool, connector_manager) = setup().await;

        report_heartbeat(&client_pool, &connector_manager).await;

        // Verify that no heartbeat data exists
        assert!(connector_manager.connector_heartbeat.is_empty());
    }
}
