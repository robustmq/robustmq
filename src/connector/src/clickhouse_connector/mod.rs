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
use clickhouse::Client;
use clickhouse::Row;
use common_base::error::common::CommonError;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    connector::config_clickhouse::ClickHouseConnectorConfig, connector::MQTTConnector,
    storage::adapter_record::AdapterWriteRecord,
};
use serde::Serialize;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error};

use super::{
    core::{BridgePluginReadConfig, BridgePluginThread},
    loops::run_connector_loop,
    manager::ConnectorManager,
    traits::ConnectorSink,
};

/// Standard row schema for ClickHouse ingestion.
///
/// Users should create a matching table:
/// ```sql
/// CREATE TABLE mqtt_messages (
///     data String,
///     key String,
///     timestamp UInt64
/// ) ENGINE = MergeTree()
/// ORDER BY timestamp;
/// ```
#[derive(Row, Serialize)]
struct MqttMessageRow {
    data: String,
    key: String,
    timestamp: u64,
}

pub struct ClickHouseBridgePlugin {
    config: ClickHouseConnectorConfig,
}

impl ClickHouseBridgePlugin {
    pub fn new(config: ClickHouseConnectorConfig) -> Self {
        ClickHouseBridgePlugin { config }
    }
}

#[async_trait]
impl ConnectorSink for ClickHouseBridgePlugin {
    type SinkResource = Client;

    async fn validate(&self) -> Result<(), CommonError> {
        self.config.validate()
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
        let mut client = Client::default()
            .with_url(&self.config.url)
            .with_database(&self.config.database);

        if !self.config.username.is_empty() {
            client = client.with_user(&self.config.username);
        }
        if !self.config.password.is_empty() {
            client = client.with_password(&self.config.password);
        }

        client.query("SELECT 1").execute().await.map_err(|e| {
            CommonError::CommonError(format!(
                "Failed to connect to ClickHouse at {}: {}",
                self.config.url, e
            ))
        })?;

        debug!(
            "Connected to ClickHouse: url={}, database={}, table={}",
            self.config.url, self.config.database, self.config.table
        );

        Ok(client)
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        client: &mut Client,
    ) -> Result<(), CommonError> {
        if records.is_empty() {
            return Ok(());
        }

        let mut insert = client
            .insert::<MqttMessageRow>(&self.config.table)
            .await
            .map_err(|e| {
                CommonError::CommonError(format!("Failed to prepare ClickHouse insert: {}", e))
            })?;

        for record in records {
            let row = MqttMessageRow {
                data: String::from_utf8_lossy(&record.data).to_string(),
                key: record.key.clone().unwrap_or_default(),
                timestamp: record.timestamp,
            };

            insert.write(&row).await.map_err(|e| {
                CommonError::CommonError(format!("Failed to write row to ClickHouse: {}", e))
            })?;
        }

        insert.end().await.map_err(|e| {
            CommonError::CommonError(format!("Failed to flush ClickHouse insert: {}", e))
        })?;

        Ok(())
    }
}

pub fn start_clickhouse_connector(
    client_pool: Arc<ClientPool>,
    connector_manager: Arc<ConnectorManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    connector: MQTTConnector,
    thread: BridgePluginThread,
    stop_recv: Receiver<bool>,
) {
    tokio::spawn(Box::pin(async move {
        let ch_config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::ClickHouse(config) => config.clone(),
            _ => {
                error!("Invalid connector config type, expected ClickHouse config");
                return;
            }
        };
        let bridge = ClickHouseBridgePlugin::new(ch_config);

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
                "Failed to start ClickHouseBridgePlugin with error message: {:?}",
                e
            );
        }
    }));
}
