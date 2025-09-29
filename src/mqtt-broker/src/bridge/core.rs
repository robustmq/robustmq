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
    config_local_file::LocalFileConnectorConfig, config_postgres::PostgresConnectorConfig,
    config_pulsar::PulsarConnectorConfig, connector::MQTTConnector, connector_type::ConnectorType,
    status::MQTTStatus,
};
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{sync::broadcast, time::sleep};
use tracing::{error, info};

use super::{
    file::FileBridgePlugin, manager::ConnectorManager, postgres::PostgresBridgePlugin,
    pulsar::PulsarBridgePlugin,
};

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
            ConnectorType::Pulsar => {
                let pulsar_config = match serde_json::from_str::<PulsarConnectorConfig>(
                    &connector.config,
                ) {
                    Ok(config) => config,
                    Err(e) => {
                        error!("Failed to parse PulsarConnectorConfig file with error message :{}, configuration contents: {}", e, connector.config);
                        return;
                    }
                };

                let bridge = PulsarBridgePlugin::new(
                    connector_manager.clone(),
                    message_storage.clone(),
                    connector.connector_name.clone(),
                    pulsar_config,
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
                        "Failed to start PulsarBridgePlugin with error message: {:?}",
                        e
                    );
                }
            }
            ConnectorType::Postgres => {
                let postgres_config = match serde_json::from_str::<PostgresConnectorConfig>(
                    &connector.config,
                ) {
                    Ok(config) => config,
                    Err(e) => {
                        error!("Failed to parse PostgresConnectorConfig with error message: {}, configuration contents: {}", e, connector.config);
                        return;
                    }
                };

                let bridge = PostgresBridgePlugin::new(
                    connector_manager.clone(),
                    message_storage.clone(),
                    connector.connector_name.clone(),
                    postgres_config,
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
                        "Failed to start PostgresBridgePlugin with error message: {:?}",
                        e
                    );
                }
            }
        }
    });
}

fn stop_thread(thread: BridgePluginThread) -> ResultMqttBrokerError {
    thread.stop_send.send(true)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::manager::ConnectorManager;
    use common_base::tools::{now_second, unique_id};
    use common_config::{broker::init_broker_conf_by_config, config::BrokerConfig};
    use storage_adapter::storage::{build_memory_storage_driver, ArcStorageAdapter, ShardInfo};

    fn setup() -> (ArcStorageAdapter, Arc<ConnectorManager>) {
        let namespace = unique_id();
        let config = BrokerConfig {
            cluster_name: namespace.clone(),
            broker_id: 1,
            ..Default::default()
        };
        init_broker_conf_by_config(config);

        let storage_adapter = build_memory_storage_driver();
        let connector_manager = Arc::new(ConnectorManager::new());
        (storage_adapter, connector_manager)
    }

    fn create_test_connector() -> MQTTConnector {
        MQTTConnector {
            connector_name: "test_connector".to_string(),
            connector_type: ConnectorType::LocalFile,
            topic_id: "test_topic".to_string(),
            config: "{}".to_string(),
            status: MQTTStatus::Running,
            broker_id: Some(1),
            cluster_name: "test_cluster".to_string(),
            create_time: now_second(),
            update_time: now_second(),
        }
    }

    #[test]
    fn test_bridge_plugin_read_config_creation() {
        let config = BridgePluginReadConfig {
            topic_id: "test_topic".to_string(),
            record_num: 100,
        };

        assert_eq!(config.topic_id, "test_topic");
        assert_eq!(config.record_num, 100);
    }

    #[test]
    fn test_bridge_plugin_thread_creation() {
        let (stop_send, _) = broadcast::channel::<bool>(1);
        let thread = BridgePluginThread {
            connector_name: "test_connector".to_string(),
            stop_send: stop_send.clone(),
        };

        assert_eq!(thread.connector_name, "test_connector");
        assert!(thread.stop_send.same_channel(&stop_send));
    }

    #[tokio::test]
    async fn test_start_connector_thread() {
        let (storage_adapter, connector_manager) = setup();
        let (stop_send, _) = broadcast::channel::<bool>(1);

        let start_handle = tokio::spawn(async move {
            start_connector_thread(storage_adapter, connector_manager, stop_send).await;
        });

        sleep(Duration::from_millis(100)).await;

        start_handle.abort();
        assert!(start_handle.await.unwrap_err().is_cancelled());
    }

    #[tokio::test]
    async fn test_check_connector() {
        let (storage_adapter, connector_manager) = setup();

        let mut connector = create_test_connector();
        connector.broker_id = Some(1);
        connector.config = r#"{"local_file_path": "/tmp/test.txt"}"#.to_string();
        connector_manager.add_connector(&connector);

        let shard_name = connector.topic_id.clone();
        storage_adapter
            .create_shard(ShardInfo {
                namespace: "default".to_string(),
                shard_name,
                ..Default::default()
            })
            .await
            .unwrap();

        check_connector(&storage_adapter, &connector_manager).await;

        sleep(Duration::from_millis(100)).await;

        assert!(connector_manager
            .get_connector_thread(&connector.connector_name)
            .is_none());
    }

    #[tokio::test]
    async fn test_stop_thread() {
        let (stop_send, mut stop_recv) = broadcast::channel::<bool>(1);
        let thread = BridgePluginThread {
            connector_name: "test_connector".to_string(),
            stop_send: stop_send.clone(),
        };

        assert!(stop_thread(thread).is_ok());
        assert!(stop_recv.recv().await.unwrap());
    }
}
