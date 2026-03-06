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

use async_trait::async_trait;
use common_base::error::common::CommonError;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    connector::config_cassandra::CassandraConnectorConfig, connector::MQTTConnector,
    storage::adapter_record::AdapterWriteRecord,
};
use rule_engine::apply_rule_engine;
use scylla::{Session, SessionBuilder};
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error};

use super::{
    core::{BridgePluginReadConfig, BridgePluginThread},
    loops::run_connector_loop,
    manager::ConnectorManager,
    traits::ConnectorSink,
};

pub struct CassandraBridgePlugin {
    connector: MQTTConnector,
    config: CassandraConnectorConfig,
}

impl CassandraBridgePlugin {
    pub fn new(connector: MQTTConnector) -> Result<Self, CommonError> {
        let config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::Cassandra(config) => config.clone(),
            _ => {
                return Err(CommonError::CommonError(
                    "invalid connector type for cassandra plugin".to_string(),
                ));
            }
        };
        Ok(CassandraBridgePlugin { connector, config })
    }

    fn build_insert_cql(&self) -> String {
        format!(
            "INSERT INTO {}.{} (msgid, topic, qos, payload, arrived) VALUES (?, ?, ?, ?, ?)",
            self.config.keyspace, self.config.table
        )
    }
}

#[async_trait]
impl ConnectorSink for CassandraBridgePlugin {
    type SinkResource = Session;

    async fn validate(&self) -> Result<(), CommonError> {
        self.config.validate()
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
        let known_nodes = self.config.known_nodes();

        let mut builder = SessionBuilder::new()
            .known_nodes(&known_nodes)
            .connection_timeout(Duration::from_secs(self.config.timeout_secs));

        if !self.config.username.is_empty() {
            builder = builder.user(&self.config.username, &self.config.password);
        }

        let session = builder.build().await.map_err(|e| {
            CommonError::CommonError(format!(
                "Failed to connect to Cassandra at {:?}: {}",
                known_nodes, e
            ))
        })?;

        debug!(
            "Connected to Cassandra: nodes={:?}, keyspace={}, table={}",
            known_nodes, self.config.keyspace, self.config.table
        );

        Ok(session)
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        session: &mut Session,
    ) -> Result<(), CommonError> {
        if records.is_empty() {
            return Ok(());
        }

        let cql = self.build_insert_cql();
        let prepared = session.prepare(cql).await.map_err(|e| {
            CommonError::CommonError(format!("Failed to prepare CQL statement: {}", e))
        })?;

        for record in records {
            let processed_data = apply_rule_engine(&self.connector.rules, &record.data).await?;
            let payload = String::from_utf8_lossy(&processed_data).to_string();
            let key = record.key.clone().unwrap_or_default();
            let timestamp = record.timestamp as i64;

            session
                .execute_unpaged(&prepared, (&key, "", 0i32, &payload, timestamp))
                .await
                .map_err(|e| {
                    CommonError::CommonError(format!("Failed to execute CQL insert: {}", e))
                })?;
        }

        Ok(())
    }
}

pub fn start_cassandra_connector(
    client_pool: Arc<ClientPool>,
    connector_manager: Arc<ConnectorManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    connector: MQTTConnector,
    thread: BridgePluginThread,
    stop_recv: Receiver<bool>,
) {
    tokio::spawn(Box::pin(async move {
        let connector_name = connector.connector_name.clone();
        let connector_type = connector.connector_type.to_string();
        let bridge = match CassandraBridgePlugin::new(connector.clone()) {
            Ok(bridge) => bridge,
            Err(e) => {
                error!(
                    "Invalid connector config type for Cassandra connector, connector_name='{}', connector_type='{}', error={}",
                    connector_name, connector_type, e
                );
                return;
            }
        };

        connector_manager.add_connector_thread(&connector.connector_name, thread);

        if let Err(e) = run_connector_loop(
            &bridge,
            &client_pool,
            &connector_manager,
            storage_driver_manager.clone(),
            connector.connector_name.clone(),
            BridgePluginReadConfig {
                topic_name: connector.topic_name,
                record_num: 100,
                strategy: connector.failure_strategy,
            },
            stop_recv,
        )
        .await
        {
            connector_manager.remove_connector_thread(&connector.connector_name);
            error!(
                "Failed to start CassandraBridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}
