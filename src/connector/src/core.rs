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

use common_base::error::common::CommonError;
use common_base::{error::ResultCommonError, tools::loop_select_ticket};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::connector::{
    status::MQTTStatus, ConnectorType, FailureHandlingStrategy, MQTTConnector,
};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::{
    broadcast,
    mpsc::{self, Receiver},
};
use tracing::{debug, error, info, warn};

use super::{
    elasticsearch::start_elasticsearch_connector, file::start_local_file_connector,
    greptimedb::start_greptimedb_connector, kafka::start_kafka_connector,
    manager::ConnectorManager, mongodb::start_mongodb_connector, mysql::start_mysql_connector,
    postgres::start_postgres_connector, pulsar::start_pulsar_connector,
    rabbitmq::start_rabbitmq_connector, redis::start_redis_connector,
};
use crate::storage::connector::ConnectorStorage;

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
    pub stop_send: mpsc::Sender<bool>,
    pub last_msg: Option<String>,
}

pub(crate) async fn start_connector_thread(
    client_pool: Arc<ClientPool>,
    storage_driver_manager: Arc<StorageDriverManager>,
    connector_manager: Arc<ConnectorManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let ac_fn = async || -> ResultCommonError {
        let current_broker_id = broker_config().broker_id;
        start_connectors(
            &storage_driver_manager,
            &connector_manager,
            &client_pool,
            current_broker_id,
        );
        gc_connectors(&connector_manager, &client_pool, current_broker_id).await;
        Ok(())
    };

    loop_select_ticket(ac_fn, 1000, &stop_send).await;
}

fn start_connectors(
    storage_driver_manager: &Arc<StorageDriverManager>,
    connector_manager: &Arc<ConnectorManager>,
    client_pool: &Arc<ClientPool>,
    current_broker_id: u64,
) {
    for raw in connector_manager.get_all_connector() {
        if raw.broker_id != Some(current_broker_id) {
            continue;
        }

        if connector_manager
            .get_connector_thread(&raw.connector_name)
            .is_some()
        {
            continue;
        }

        info!(
            "Starting connector '{}' (type: {:?}, topic: {})",
            raw.connector_name, raw.connector_type, raw.topic_name
        );

        let (stop_send, stop_rx) = mpsc::channel::<bool>(1);
        let thread = BridgePluginThread {
            connector_name: raw.connector_name.clone(),
            last_send_time: 0,
            send_fail_total: 0,
            send_success_total: 0,
            stop_send,
            last_msg: None,
        };

        start_thread(
            client_pool.clone(),
            connector_manager.clone(),
            storage_driver_manager.clone(),
            raw.clone(),
            thread,
            stop_rx,
        );
    }
}

async fn gc_connectors(
    connector_manager: &Arc<ConnectorManager>,
    client_pool: &Arc<ClientPool>,
    current_broker_id: u64,
) {
    for raw in connector_manager.get_all_connector_thread() {
        let should_stop = match connector_manager.get_connector(&raw.connector_name) {
            Some(connector) => connector.broker_id != Some(current_broker_id),
            None => true,
        };

        if !should_stop {
            continue;
        }

        if let Err(e) = stop_thread(raw.clone()).await {
            warn!("Failed to stop connector '{}': {}", raw.connector_name, e);
        }
        connector_manager.remove_connector_thread(&raw.connector_name);

        if let Some(mut connector) = connector_manager.get_connector(&raw.connector_name) {
            connector.status = MQTTStatus::Idle;
            let storage = ConnectorStorage::new(client_pool.clone());
            if let Err(e) = storage.update_connector(connector).await {
                error!(
                    "Failed to update connector '{}' to Idle: {}",
                    raw.connector_name, e
                );
            }
        }
    }
}

fn start_thread(
    client_pool: Arc<ClientPool>,
    connector_manager: Arc<ConnectorManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    connector: MQTTConnector,
    thread: BridgePluginThread,
    stop_rx: Receiver<bool>,
) {
    let connector_name = connector.connector_name.clone();
    let connector_type = connector.connector_type.clone();

    debug!(
        "Dispatching connector '{}' to type-specific handler: {:?}",
        connector_name, connector_type
    );

    match connector_type {
        ConnectorType::LocalFile(_) => {
            start_local_file_connector(
                client_pool,
                connector_manager,
                storage_driver_manager,
                connector,
                thread,
                stop_rx,
            );
        }
        ConnectorType::Kafka(_) => {
            start_kafka_connector(
                client_pool,
                connector_manager,
                storage_driver_manager,
                connector,
                thread,
                stop_rx,
            );
        }
        ConnectorType::GreptimeDB(_) => {
            start_greptimedb_connector(
                client_pool,
                connector_manager,
                storage_driver_manager,
                connector,
                thread,
                stop_rx,
            );
        }
        ConnectorType::Pulsar(_) => {
            start_pulsar_connector(
                client_pool,
                connector_manager,
                storage_driver_manager,
                connector,
                thread,
                stop_rx,
            );
        }
        ConnectorType::Postgres(_) => {
            start_postgres_connector(
                client_pool,
                connector_manager,
                storage_driver_manager,
                connector,
                thread,
                stop_rx,
            );
        }
        ConnectorType::MongoDB(_) => {
            start_mongodb_connector(
                client_pool,
                connector_manager,
                storage_driver_manager,
                connector,
                thread,
                stop_rx,
            );
        }
        ConnectorType::RabbitMQ(_) => {
            start_rabbitmq_connector(
                client_pool,
                connector_manager,
                storage_driver_manager,
                connector,
                thread,
                stop_rx,
            );
        }
        ConnectorType::MySQL(_) => {
            start_mysql_connector(
                client_pool,
                connector_manager,
                storage_driver_manager,
                connector,
                thread,
                stop_rx,
            );
        }
        ConnectorType::Elasticsearch(_) => {
            start_elasticsearch_connector(
                client_pool,
                connector_manager,
                storage_driver_manager,
                connector,
                thread,
                stop_rx,
            );
        }
        ConnectorType::Redis(_) => {
            start_redis_connector(
                client_pool,
                connector_manager,
                storage_driver_manager,
                connector,
                thread,
                stop_rx,
            );
        }
    }
}

async fn stop_thread(thread: BridgePluginThread) -> Result<(), CommonError> {
    debug!(
        "Sending stop signal to connector '{}' thread",
        thread.connector_name
    );
    thread.stop_send.send(true).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::ConnectorManager;
    use common_base::uuid::unique_id;
    use common_config::{broker::init_broker_conf_by_config, config::BrokerConfig};
    use metadata_struct::connector::FailureHandlingStrategy;
    use std::time::Duration;
    use storage_adapter::storage::test_build_storage_driver_manager;
    use tokio::time::sleep;

    async fn setup() -> (Arc<StorageDriverManager>, Arc<ConnectorManager>) {
        let namespace = unique_id();
        let config = BrokerConfig {
            cluster_name: namespace.clone(),
            broker_id: 1,
            ..Default::default()
        };
        init_broker_conf_by_config(config);

        let storage_adapter = test_build_storage_driver_manager().await.unwrap();
        let connector_manager = Arc::new(ConnectorManager::new());
        (storage_adapter, connector_manager)
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
        let (stop_send, _) = mpsc::channel::<bool>(1);
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
        let (storage_driver_manager, connector_manager) = setup().await;
        let (stop_send, _) = broadcast::channel::<bool>(1);
        let client_pool = Arc::new(ClientPool::new(1));
        let start_handle = tokio::spawn(async move {
            start_connector_thread(
                client_pool,
                storage_driver_manager,
                connector_manager,
                stop_send,
            )
            .await;
        });

        sleep(Duration::from_millis(100)).await;

        start_handle.abort();
        assert!(start_handle.await.unwrap_err().is_cancelled());
    }

    #[tokio::test]
    async fn test_stop_thread() {
        let (stop_send, mut stop_recv) = mpsc::channel::<bool>(1);
        let thread = BridgePluginThread {
            connector_name: "test_connector".to_string(),
            last_send_time: 0,
            send_fail_total: 0,
            send_success_total: 0,
            stop_send: stop_send.clone(),
            last_msg: None,
        };

        assert!(stop_thread(thread).await.is_ok());
        assert!(stop_recv.recv().await.unwrap());
    }
}
