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

use std::sync::Arc;

use axum::async_trait;
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_mysql::MySQLConnectorConfig,
    mqtt::bridge::connector::MQTTConnector,
};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use storage_adapter::storage::ArcStorageAdapter;
use tracing::{error, warn};

use crate::{bridge::failure::FailureHandlingStrategy, common::types::ResultMqttBrokerError};

use super::{
    core::{run_connector_loop, BridgePluginReadConfig, BridgePluginThread, ConnectorSink},
    manager::ConnectorManager,
};

pub struct MySQLBridgePlugin {
    config: MySQLConnectorConfig,
}

impl MySQLBridgePlugin {
    pub fn new(config: MySQLConnectorConfig) -> Self {
        MySQLBridgePlugin { config }
    }

    async fn create_pool(&self) -> Result<Pool<MySql>, sqlx::Error> {
        MySqlPoolOptions::new()
            .max_connections(self.config.get_pool_size())
            .connect(&self.config.connection_url())
            .await
    }

    async fn single_insert(&self, records: &[Record], pool: &Pool<MySql>) -> ResultMqttBrokerError {
        for record in records {
            let payload = serde_json::to_string(record)?;

            let base_sql = if let Some(template) = &self.config.sql_template {
                let placeholder_count = template.matches('?').count();
                if placeholder_count != 3 {
                    warn!(
                        "sql_template expects 3 placeholders, but found {}. Using provided template with binds (key, payload, timestamp) in order.",
                        placeholder_count
                    );
                }
                template.clone()
            } else {
                format!(
                    "INSERT INTO {} (record_key, payload, timestamp) VALUES (?, ?, ?)",
                    self.config.table
                )
            };

            let sql = if self.config.is_upsert_enabled() {
                format!(
                    "{} ON DUPLICATE KEY UPDATE payload = VALUES(payload), timestamp = VALUES(timestamp)",
                    base_sql
                )
            } else {
                base_sql
            };

            sqlx::query(&sql)
                .bind(&record.key)
                .bind(&payload)
                .bind(record.timestamp as i64)
                .execute(pool)
                .await?;
        }

        Ok(())
    }

    async fn batch_insert(&self, records: &[Record], pool: &Pool<MySql>) -> ResultMqttBrokerError {
        let mut values_placeholders = Vec::with_capacity(records.len());
        let mut bindings = Vec::with_capacity(records.len());

        for record in records {
            values_placeholders.push("(?, ?, ?)");
            let payload = serde_json::to_string(record)?;
            bindings.push((record.key.clone(), payload, record.timestamp as i64));
        }

        let base_sql = format!(
            "INSERT INTO {} (record_key, payload, timestamp) VALUES {}",
            self.config.table,
            values_placeholders.join(", ")
        );

        let sql = if self.config.is_upsert_enabled() {
            format!(
                "{} ON DUPLICATE KEY UPDATE payload = VALUES(payload), timestamp = VALUES(timestamp)",
                base_sql
            )
        } else {
            base_sql
        };

        let mut query = sqlx::query(&sql);
        for (key, payload, timestamp) in bindings {
            query = query.bind(key).bind(payload).bind(timestamp);
        }

        query.execute(pool).await?;

        Ok(())
    }
}

#[async_trait]
impl ConnectorSink for MySQLBridgePlugin {
    type SinkResource = Pool<MySql>;

    async fn validate(&self) -> ResultMqttBrokerError {
        Ok(())
    }

    async fn init_sink(
        &self,
    ) -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError> {
        let pool = self.create_pool().await?;
        Ok(pool)
    }

    async fn send_batch(
        &self,
        records: &[Record],
        pool: &mut Pool<MySql>,
    ) -> ResultMqttBrokerError {
        if records.is_empty() {
            return Ok(());
        }
        if self.config.is_batch_insert_enabled() {
            if self.config.sql_template.is_some() {
                warn!(
                    "sql_template is not applied in batch mode; default batch INSERT will be used"
                );
            }
            self.batch_insert(records, pool).await
        } else {
            self.single_insert(records, pool).await
        }
    }

    async fn cleanup_sink(&self, pool: Pool<MySql>) -> ResultMqttBrokerError {
        pool.close().await;
        Ok(())
    }
}

pub fn start_mysql_connector(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        let mysql_config = match serde_json::from_str::<MySQLConnectorConfig>(&connector.config) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to parse MySQLConnectorConfig with error message: {}, configuration contents: {}", e, connector.config);
                return;
            }
        };

        let failure_strategy = match serde_json::from_str::<FailureHandlingStrategy>(
            &connector.failure_strategy,
        ) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to parse FailureHandlingStrategy file with error message :{}, configuration contents: {}", e, connector.failure_strategy);
                return;
            }
        };

        let bridge = MySQLBridgePlugin::new(mysql_config);

        let stop_recv = thread.stop_send.subscribe();
        connector_manager.add_connector_thread(&connector.connector_name, thread);

        if let Err(e) = run_connector_loop(
            &bridge,
            &connector_manager,
            message_storage.clone(),
            connector.connector_name.clone(),
            BridgePluginReadConfig {
                topic_name: connector.topic_name,
                record_num: 100,
                strategy: failure_strategy,
            },
            stop_recv,
        )
        .await
        {
            connector_manager.remove_connector_thread(&connector.connector_name);
            error!(
                "Failed to start MySQLBridgePlugin with error message: {:?}",
                e
            );
        }
    });
}
