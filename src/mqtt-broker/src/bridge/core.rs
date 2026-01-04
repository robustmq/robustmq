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

use crate::{
    bridge::failure::failure_message_process, handler::tool::ResultMqttBrokerError,
    storage::connector::ConnectorStorage,
};
use axum::async_trait;

use common_base::{
    error::ResultCommonError,
    tools::{loop_select_ticket, now_millis},
};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    mqtt::bridge::{
        connector::{FailureHandlingStrategy, MQTTConnector},
        connector_type::ConnectorType,
        status::MQTTStatus,
    },
    storage::adapter_record::AdapterWriteRecord,
};
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};
use tracing::{debug, error, info, warn};

use super::{
    elasticsearch::start_elasticsearch_connector,
    file::start_local_file_connector,
    greptimedb::start_greptimedb_connector,
    kafka::start_kafka_connector,
    manager::{update_last_active, ConnectorManager},
    mongodb::start_mongodb_connector,
    mysql::start_mysql_connector,
    postgres::start_postgres_connector,
    pulsar::start_pulsar_connector,
    rabbitmq::start_rabbitmq_connector,
    redis::start_redis_connector,
};

use crate::storage::message::MessageStorage;

#[derive(Clone)]
pub struct BridgePluginReadConfig {
    pub topic_name: String,
    pub record_num: u64,
    pub strategy: FailureHandlingStrategy,
}

#[derive(Clone)]
pub struct BridgePluginThread {
    pub connector_name: String,
    pub last_send_time: u64,
    pub send_success_total: u64,
    pub send_fail_total: u64,
    pub stop_send: broadcast::Sender<bool>,
    pub last_msg: Option<String>,
}

#[async_trait]
pub trait BridgePlugin {
    async fn exec(&self, config: BridgePluginReadConfig) -> ResultMqttBrokerError;
}

#[async_trait]
pub trait ConnectorSink: Send + Sync {
    type SinkResource: Send;

    async fn validate(&self) -> ResultMqttBrokerError;

    async fn init_sink(&self)
        -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError>;

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
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
    sink.validate().await?;

    let mut resource = sink.init_sink().await?;
    let message_storage = MessageStorage::new(message_storage);
    let group_name = connector_name.clone();

    loop {
        let offset = message_storage
            .get_group_offset(&group_name, &config.topic_name)
            .await?;
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

                        let start_time = now_millis();
                        let message_count = data.len() as u64;
                        let mut retry_times = 0;
                        loop{
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
                                    break;
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
                                    if failure_message_process(config.strategy.clone(),retry_times).await{
                                        sleep(Duration::from_millis(100)).await;
                                        break
                                    }
                                    retry_times +=1;
                                }
                            }
                        }

                    },
                    Err(e) => {
                        update_last_active(connector_manager, &connector_name, now_millis(), 0, false);
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

    loop_select_ticket(ac_fn, 1000, &stop_send).await;
    info!("Connector thread exited successfully");
}

async fn check_connector(
    message_storage: &ArcStorageAdapter,
    connector_manager: &Arc<ConnectorManager>,
    client_pool: &Arc<ClientPool>,
) {
    let config = broker_config();
    let current_broker_id = config.broker_id;

    let all_connectors = connector_manager.get_all_connector();
    let all_threads = connector_manager.get_all_connector_thread();

    debug!(
        "Checking connectors: current_broker_id={}, total_connectors={}, running_threads={}",
        current_broker_id,
        all_connectors.len(),
        all_threads.len()
    );

    let mut started_count = 0;
    let mut skipped_count = 0;
    let mut stopped_count = 0;

    // Start connector thread
    for raw in all_connectors {
        // Skip if broker_id is not assigned
        if raw.broker_id.is_none() {
            debug!(
                "Skipping connector '{}': broker_id not assigned (status: {:?})",
                raw.connector_name, raw.status
            );
            skipped_count += 1;
            continue;
        }

        // Skip if not assigned to current broker
        if let Some(broker_id) = raw.broker_id {
            if broker_id != current_broker_id {
                debug!(
                    "Skipping connector '{}': assigned to broker {} (current: {})",
                    raw.connector_name, broker_id, current_broker_id
                );
                skipped_count += 1;
                continue;
            }
        }

        // Skip if thread already running
        if connector_manager
            .get_connector_thread(&raw.connector_name)
            .is_some()
        {
            debug!(
                "Connector '{}' thread already running, skipping",
                raw.connector_name
            );
            skipped_count += 1;
            continue;
        }

        // Start new connector thread
        info!(
            "Starting connector '{}' (type: {:?}, topic: {}, broker_id: {})",
            raw.connector_name, raw.connector_type, raw.topic_name, current_broker_id
        );

        let (stop_send, _) = broadcast::channel::<bool>(1);
        let thread = BridgePluginThread {
            connector_name: raw.connector_name.clone(),
            last_send_time: 0,
            send_fail_total: 0,
            send_success_total: 0,
            stop_send,
            last_msg: None,
        };

        start_thread(
            connector_manager.clone(),
            message_storage.clone(),
            raw.clone(),
            thread,
        );
        started_count += 1;
    }

    // Gc connector thread
    for raw in all_threads {
        let (is_stop_thread, stop_reason) =
            if let Some(connector) = connector_manager.get_connector(&raw.connector_name) {
                if let Some(broker_id) = connector.broker_id {
                    if broker_id != current_broker_id {
                        (
                            true,
                            format!(
                                "broker reassigned from {} to {}",
                                current_broker_id, broker_id
                            ),
                        )
                    } else {
                        (false, String::new())
                    }
                } else {
                    (true, "broker_id is None".to_string())
                }
            } else {
                (
                    true,
                    "connector not found in manager, possibly deleted".to_string(),
                )
            };

        if is_stop_thread {
            info!(
                "Stopping connector '{}' thread: {}",
                raw.connector_name, stop_reason
            );

            match stop_thread(raw.clone()) {
                Ok(_) => {
                    info!(
                        "Successfully sent stop signal to connector '{}'",
                        raw.connector_name
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to send stop signal to connector '{}': {}",
                        raw.connector_name, e
                    );
                }
            }

            connector_manager.remove_connector_thread(&raw.connector_name);

            if let Some(mut connector) = connector_manager.get_connector(&raw.connector_name) {
                let old_status = connector.status.clone();
                connector.status = MQTTStatus::Idle;

                info!(
                    "Updating connector '{}' status: {:?} -> {:?}",
                    raw.connector_name, old_status, connector.status
                );

                let storage = ConnectorStorage::new(client_pool.clone());
                if let Err(e) = storage.update_connector(connector).await {
                    error!(
                        "Failed to update connector '{}' status to Idle: {}",
                        raw.connector_name, e
                    );
                }
            }

            stopped_count += 1;
        }
    }

    if started_count > 0 || stopped_count > 0 {
        debug!(
            "Connector check completed: started={}, stopped={}, skipped={}",
            started_count, stopped_count, skipped_count
        );
    }
}

fn start_thread(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    let connector_name = connector.connector_name.clone();
    let connector_type = connector.connector_type.clone();

    debug!(
        "Dispatching connector '{}' to type-specific handler: {:?}",
        connector_name, connector_type
    );

    match connector_type {
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
        ConnectorType::Elasticsearch => {
            start_elasticsearch_connector(connector_manager, message_storage, connector, thread);
        }
        ConnectorType::Redis => {
            start_redis_connector(connector_manager, message_storage, connector, thread);
        }
    }
}

fn stop_thread(thread: BridgePluginThread) -> ResultMqttBrokerError {
    debug!(
        "Sending stop signal to connector '{}' thread",
        thread.connector_name
    );
    thread.stop_send.send(true)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bridge::manager::ConnectorManager;
    use common_base::tools::{now_second, unique_id};
    use common_config::{broker::init_broker_conf_by_config, config::BrokerConfig};
    use metadata_struct::{
        mqtt::bridge::connector::FailureHandlingStrategy, storage::adapter_offset::AdapterShardInfo,
    };
    use storage_adapter::storage::{build_storage_driver_manager, ArcStorageAdapter};

    fn setup() -> (ArcStorageAdapter, Arc<ConnectorManager>) {
        let namespace = unique_id();
        let config = BrokerConfig {
            cluster_name: namespace.clone(),
            broker_id: 1,
            ..Default::default()
        };
        init_broker_conf_by_config(config);

        let storage_adapter = build_storage_driver_manager();
        let connector_manager = Arc::new(ConnectorManager::new());
        (storage_adapter, connector_manager)
    }

    fn create_test_connector() -> MQTTConnector {
        use metadata_struct::mqtt::bridge::{
            config_local_file::LocalFileConnectorConfig, ConnectorConfig,
        };

        MQTTConnector {
            connector_name: "test_connector".to_string(),
            connector_type: ConnectorType::LocalFile,
            topic_name: "test_topic".to_string(),
            config: ConnectorConfig::LocalFile(LocalFileConnectorConfig::default()),
            failure_strategy: FailureHandlingStrategy::Discard,
            status: MQTTStatus::Running,
            broker_id: Some(1),
            create_time: now_second(),
            update_time: now_second(),
        }
    }

    #[test]
    fn test_bridge_plugin_read_config_creation() {
        let config = BridgePluginReadConfig {
            topic_name: "test_topic".to_string(),
            record_num: 100,
            strategy: FailureHandlingStrategy::Discard,
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
            last_msg: None,
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
        use metadata_struct::mqtt::bridge::{
            config_local_file::LocalFileConnectorConfig, ConnectorConfig,
        };

        let (storage_adapter, connector_manager) = setup();

        let mut connector = create_test_connector();
        connector.broker_id = Some(1);
        connector.config = ConnectorConfig::LocalFile(LocalFileConnectorConfig {
            local_file_path: "/tmp/test.txt".to_string(),
            ..Default::default()
        });
        connector_manager.add_connector(&connector);

        let shard_name = connector.topic_name.clone();
        storage_adapter
            .create_shard(&AdapterShardInfo {
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
            .is_some());
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
            last_msg: None,
        };

        assert!(stop_thread(thread).is_ok());
        assert!(stop_recv.recv().await.unwrap());
    }
}
