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

use crate::common::types::ResultMqttBrokerError;
use axum::async_trait;

use common_base::{error::ResultCommonError, tools::loop_select};
use common_config::broker::broker_config;
use metadata_struct::mqtt::bridge::{
    config_local_file::LocalFileConnectorConfig, connector::MQTTConnector,
    connector_type::ConnectorType, status::MQTTStatus,
};
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{sync::broadcast, time::sleep};
use tracing::{error, info};

use super::{file::FileBridgePlugin, manager::ConnectorManager};

#[derive(Clone)]
pub struct BridgePluginReadConfig {
    pub topic_id: String,
    pub record_num: u64,
}

#[derive(Clone)]
pub struct BridgePluginThread {
    pub connector_name: String,
    pub stop_send: broadcast::Sender<bool>,
}

#[async_trait]
pub trait BridgePlugin {
    async fn exec(&self, config: BridgePluginReadConfig) -> ResultMqttBrokerError;
}

pub async fn start_connector_thread(
    message_storage: ArcStorageAdapter,
    connector_manager: Arc<ConnectorManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        check_connector(&message_storage, &connector_manager).await;
        sleep(Duration::from_secs(1)).await;
        Ok(())
    };

    loop_select(ac_fn, 1, &stop_send).await;
    info!("Connector thread exited successfully");
}

async fn check_connector(
    message_storage: &ArcStorageAdapter,
    connector_manager: &Arc<ConnectorManager>,
) {
    let config = broker_config();

    // Start connector thread
    for raw in connector_manager.get_all_connector() {
        if raw.broker_id.is_none() {
            continue;
        }

        if let Some(broker_id) = raw.broker_id {
            if broker_id != config.broker_id {
                continue;
            }
        }

        if connector_manager
            .get_connector_thread(&raw.connector_name)
            .is_some()
        {
            continue;
        }

        let (stop_send, _) = broadcast::channel::<bool>(1);
        let thread = BridgePluginThread {
            connector_name: raw.connector_name.clone(),
            stop_send,
        };

        start_thread(
            connector_manager.clone(),
            message_storage.clone(),
            raw.clone(),
            thread,
        );
    }

    // Gc connector thread
    for raw in connector_manager.get_all_connector_thread() {
        let is_stop_thread =
            if let Some(connecttor) = connector_manager.get_connector(&raw.connector_name) {
                if let Some(broker_id) = connecttor.broker_id {
                    broker_id != config.broker_id
                } else {
                    true
                }
            } else {
                true
            };

        if is_stop_thread {
            if let Err(e) = stop_thread(raw.clone()) {
                error!(
                    "Stopping connector {} Thread failed with error message: {}",
                    raw.connector_name, e
                );
            }
            if let Some(mut connector) = connector_manager.get_connector(&raw.connector_name) {
                connector.status = MQTTStatus::Idle;
            }
        }
    }
}

fn start_thread(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        match connector.connector_type {
            ConnectorType::LocalFile => {
                let local_file_config = match serde_json::from_str::<LocalFileConnectorConfig>(
                    &connector.config,
                ) {
                    Ok(config) => config,
                    Err(e) => {
                        error!("Failed to parse LocalFileConnectorConfig file with error message :{}, configuration contents: {}", e, connector.config);
                        return;
                    }
                };

                let bridge = FileBridgePlugin::new(
                    connector_manager.clone(),
                    message_storage.clone(),
                    connector.connector_name.clone(),
                    local_file_config,
                    thread.stop_send.clone(),
                );

                connector_manager.add_connector_thread(&connector.connector_name, thread);

                if let Err(e) = bridge
                    .exec(BridgePluginReadConfig {
                        topic_id: connector.topic_id,
                        record_num: 100,
                    })
                    .await
                {
                    connector_manager.remove_connector_thread(&connector.connector_name);
                    error!(
                        "Failed to start FileBridgePlugin with error message: {:?}",
                        e
                    );
                }
            }
            ConnectorType::Kafka => {}
            ConnectorType::GreptimeDB => {}
        }
    });
}

fn stop_thread(thread: BridgePluginThread) -> ResultMqttBrokerError {
    thread.stop_send.send(true)?;
    Ok(())
}
