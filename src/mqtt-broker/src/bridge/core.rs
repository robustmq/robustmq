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

use crate::{common::types::ResultMqttBrokerError, storage::connector::ConnectorStorage};
use axum::async_trait;

use common_base::{
    error::ResultCommonError,
    tools::{loop_select_ticket, now_mills},
};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    adapter::record::Record,
    mqtt::bridge::{connector::MQTTConnector, connector_type::ConnectorType, status::MQTTStatus},
};
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};
use tracing::{error, info};

use super::{
    file::start_local_file_connector,
    greptimedb::start_greptimedb_connector,
    kafka::start_kafka_connector,
    manager::{update_last_active, ConnectorManager},
    mongodb::start_mongodb_connector,
    mysql::start_mysql_connector,
    postgres::start_postgres_connector,
    pulsar::start_pulsar_connector,
    rabbitmq::start_rabbitmq_connector,
};

use crate::storage::message::MessageStorage;

#[derive(Clone)]
pub struct BridgePluginReadConfig {
    pub topic_name: String,
    pub record_num: u64,
}

#[derive(Clone)]
pub struct BridgePluginThread {
    pub connector_name: String,
    pub last_send_time: u64,
    pub send_success_total: u64,
    pub send_fail_total: u64,
    pub stop_send: broadcast::Sender<bool>,
}

#[async_trait]
pub trait BridgePlugin {
    async fn exec(&self, config: BridgePluginReadConfig) -> ResultMqttBrokerError;
}

#[async_trait]
pub trait ConnectorSink: Send + Sync {
    type SinkResource: Send;

    async fn init_sink(&self)
        -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError>;

    async fn send_batch(
        &self,
        records: &[Record],
        resource: &mut Self::SinkResource,
    ) -> ResultMqttBrokerError;

    async fn cleanup_sink(&self, _resource: Self::SinkResource) -> ResultMqttBrokerError {
        Ok(())
    }
}

pub async fn run_connector_loop<S: ConnectorSink>(
    sink: &S,
    connector_manager: &Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector_name: String,
    config: BridgePluginReadConfig,
    mut stop_recv: broadcast::Receiver<bool>,
) -> ResultMqttBrokerError {
    let mut resource = sink.init_sink().await?;
    let message_storage = MessageStorage::new(message_storage);
    let group_name = connector_name.clone();

    loop {
        let offset = message_storage.get_group_offset(&group_name).await?;

        select! {
            val = stop_recv.recv() => {
                if let Ok(flag) = val {
                    if flag {
                        sink.cleanup_sink(resource).await?;
                        break;
                    }
                }
            },

            val = message_storage.read_topic_message(&config.topic_name, offset, config.record_num) => {
                match val {
                    Ok(data) => {
                        connector_manager.report_heartbeat(&connector_name);

                        if data.is_empty() {
                            sleep(Duration::from_millis(100)).await;
                            continue;
                        }

                        let start_time = now_mills();
                        let message_count = data.len() as u64;

                        match sink.send_batch(&data, &mut resource).await {
                            Ok(_) => {
                                message_storage.commit_group_offset(
                                    &group_name,
                                    &config.topic_name,
                                    offset + message_count
                                ).await?;

                                update_last_active(
                                    connector_manager,
                                    &connector_name,
                                    start_time,
                                    message_count,
                                    true
                                );
                            },
                            Err(e) => {
                                update_last_active(
                                    connector_manager,
                                    &connector_name,
                                    start_time,
                                    message_count,
                                    false
                                );
                                error!("Connector {} failed to send batch: {}", connector_name, e);
                                sleep(Duration::from_millis(100)).await;
                            }
                        }
                    },
                    Err(e) => {
                        update_last_active(connector_manager, &connector_name, now_mills(), 0, false);
                        error!("Connector {} failed to read Topic {} data: {}", connector_name, config.topic_name, e);
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn start_connector_thread(
    client_pool: Arc<ClientPool>,
    message_storage: ArcStorageAdapter,
    connector_manager: Arc<ConnectorManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        check_connector(&message_storage, &connector_manager, &client_pool).await;
        sleep(Duration::from_secs(1)).await;
        Ok(())
    };

    loop_select_ticket(ac_fn, 1, &stop_send).await;
    info!("Connector thread exited successfully");
}

async fn check_connector(
    message_storage: &ArcStorageAdapter,
    connector_manager: &Arc<ConnectorManager>,
    client_pool: &Arc<ClientPool>,
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
            last_send_time: 0,
            send_fail_total: 0,
            send_success_total: 0,
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
                let storage = ConnectorStorage::new(client_pool.clone());
                if let Err(e) = storage.update_connector(connector).await {
                    error!("update connector fail,error:{}", e);
                }
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
    match connector.connector_type {
        ConnectorType::LocalFile => {
            start_local_file_connector(connector_manager, message_storage, connector, thread);
        }
        ConnectorType::Kafka => {
            start_kafka_connector(connector_manager, message_storage, connector, thread);
        }
        ConnectorType::GreptimeDB => {
            start_greptimedb_connector(connector_manager, message_storage, connector, thread);
        }
        ConnectorType::Pulsar => {
            start_pulsar_connector(connector_manager, message_storage, connector, thread);
        }
        ConnectorType::Postgres => {
            start_postgres_connector(connector_manager, message_storage, connector, thread);
        }
        ConnectorType::MongoDB => {
            start_mongodb_connector(connector_manager, message_storage, connector, thread);
        }
        ConnectorType::RabbitMQ => {
            start_rabbitmq_connector(connector_manager, message_storage, connector, thread);
        }
        ConnectorType::MySQL => {
            start_mysql_connector(connector_manager, message_storage, connector, thread);
        }
    }
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
            topic_name: "test_topic".to_string(),
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
            topic_name: "test_topic".to_string(),
            record_num: 100,
        };

        assert_eq!(config.topic_name, "test_topic");
        assert_eq!(config.record_num, 100);
    }

    #[test]
    fn test_bridge_plugin_thread_creation() {
        let (stop_send, _) = broadcast::channel::<bool>(1);
        let thread = BridgePluginThread {
            connector_name: "test_connector".to_string(),
            last_send_time: 0,
            send_fail_total: 0,
            send_success_total: 0,
            stop_send: stop_send.clone(),
        };

        assert_eq!(thread.connector_name, "test_connector");
        assert!(thread.stop_send.same_channel(&stop_send));
    }

    #[tokio::test]
    async fn test_start_connector_thread() {
        let (storage_adapter, connector_manager) = setup();
        let (stop_send, _) = broadcast::channel::<bool>(1);
        let client_pool = Arc::new(ClientPool::new(1));
        let start_handle = tokio::spawn(async move {
            start_connector_thread(client_pool, storage_adapter, connector_manager, stop_send)
                .await;
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

        let shard_name = connector.topic_name.clone();
        storage_adapter
            .create_shard(ShardInfo {
                namespace: "default".to_string(),
                shard_name,
                ..Default::default()
            })
            .await
            .unwrap();

        let client_pool = Arc::new(ClientPool::new(1));
        check_connector(&storage_adapter, &connector_manager, &client_pool).await;

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
            last_send_time: 0,
            send_fail_total: 0,
            send_success_total: 0,
            stop_send: stop_send.clone(),
        };

        assert!(stop_thread(thread).is_ok());
        assert!(stop_recv.recv().await.unwrap());
    }
}
