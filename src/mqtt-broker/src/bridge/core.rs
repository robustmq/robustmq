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

use crate::handler::error::MqttBrokerError;
use axum::async_trait;
use log::{error, info};
use metadata_struct::mqtt::bridge::{
    config_local_file::LocalFileConnectorConfig, connector::MQTTConnector,
    connector_type::ConnectorType,
};
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::StorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};

use super::{file::FileBridgePlugin, manager::ConnectorManager};

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
    async fn exec(&self, config: BridgePluginReadConfig) -> Result<(), MqttBrokerError>;
}

pub async fn start_connector_thread<S>(
    message_storage: Arc<S>,
    connector: MQTTConnector,
    connector_manager: Arc<ConnectorManager>,
    stop_send: broadcast::Sender<bool>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    let mut recv = stop_send.subscribe();
    loop {
        select! {
            val = recv.recv() =>{
                if let Ok(flag) = val {
                    if flag {
                        info!("{}","Connector thread exited successfully");
                        break;
                    }
                }
            }
            _ = check_connector(
                &message_storage,
                &connector,
                &connector_manager
            ) => {
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn check_connector<S>(
    message_storage: &Arc<S>,
    connector: &MQTTConnector,
    connector_manager: &Arc<ConnectorManager>,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    // Start connector thread
    for raw in connector_manager.get_all_connector() {
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
            connector.clone(),
            thread,
        );
    }

    // Gc connector thread
    for raw in connector_manager.get_all_connector_thread() {
        if connector_manager
            .get_connector(&raw.connector_name)
            .is_some()
        {
            continue;
        }

        if let Err(e) = stop_thread(raw) {
            error!("{}", e);
        }
    }
}

fn start_thread<S>(
    connector_manager: Arc<ConnectorManager>,
    message_storage: Arc<S>,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    tokio::spawn(async move {
        match connector.connector_type {
            ConnectorType::LocalFile => {
                let local_file_config =
                    match serde_json::from_str::<LocalFileConnectorConfig>(&connector.config) {
                        Ok(config) => config,
                        Err(e) => {
                            error!("{}", e);
                            return;
                        }
                    };

                let bridge = FileBridgePlugin::new(
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
                    error!("{}", e);
                }
            }
            ConnectorType::Kafka => {}
        }
    });
}

fn stop_thread(thread: BridgePluginThread) -> Result<(), MqttBrokerError> {
    thread.stop_send.send(true)?;
    Ok(())
}
