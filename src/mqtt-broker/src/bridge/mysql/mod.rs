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
use std::time::Duration;

use axum::async_trait;
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_mysql::MySQLConnectorConfig,
    mqtt::bridge::connector::MQTTConnector,
};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use storage_adapter::storage::ArcStorageAdapter;
use tracing::{debug, error, info, warn};

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
        debug!(
            "Creating MySQL connection pool: {}:{}/{}",
            self.config.host, self.config.port, self.config.database
        );

        MySqlPoolOptions::new()
            .max_connections(self.config.get_pool_size())
            .min_connections(self.config.min_pool_size)
            .acquire_timeout(Duration::from_secs(self.config.acquire_timeout_secs))
            .idle_timeout(Some(Duration::from_secs(self.config.idle_timeout_secs)))
            .max_lifetime(Some(Duration::from_secs(self.config.max_lifetime_secs)))
            .connect(&self.config.connection_url())
            .await
    }

    async fn single_insert(&self, records: &[Record], pool: &Pool<MySql>) -> ResultMqttBrokerError {
        let mut success_count = 0;
        let mut failed_records = Vec::new();

        for (idx, record) in records.iter().enumerate() {
            let payload = match serde_json::to_string(record) {
                Ok(p) => p,
                Err(e) => {
                    warn!(
                        "Failed to serialize record {}/{} (key: '{}', timestamp: {}): {}",
                        idx + 1,
                        records.len(),
                        record.key,
                        record.timestamp,
                        e
                    );
                    failed_records.push((idx, e.to_string()));
                    continue;
                }
            };

            let base_sql = if let Some(template) = &self.config.sql_template {
                template.clone()
            } else {
                format!(
                    "INSERT INTO `{}` (record_key, payload, timestamp) VALUES (?, ?, ?)",
                    self.config.table
                )
            };

            let sql = if self.config.is_upsert_enabled() {
                format!(
                    "{} AS new_vals ON DUPLICATE KEY UPDATE payload = new_vals.payload, timestamp = new_vals.timestamp",
                    base_sql
                )
            } else {
                base_sql
            };

            match sqlx::query(&sql)
                .bind(&record.key)
                .bind(&payload)
                .bind(record.timestamp as i64)
                .execute(pool)
                .await
            {
                Ok(_) => {
                    success_count += 1;
                    debug!("Successfully inserted record {}/{}", idx + 1, records.len());
                }
                Err(e) => {
                    warn!(
                        "Failed to insert record {}/{} (key: '{}', timestamp: {}): {}",
                        idx + 1,
                        records.len(),
                        record.key,
                        record.timestamp,
                        e
                    );
                    failed_records.push((idx, e.to_string()));
                }
            }
        }

        if success_count > 0 {
            info!(
                "Inserted {}/{} records successfully into table {}",
                success_count,
                records.len(),
                self.config.table
            );
        }

        if failed_records.len() == records.len() {
            return Err(crate::handler::error::MqttBrokerError::CommonError(
                format!("All {} records failed to insert into MySQL", records.len()),
            ));
        }

        Ok(())
    }

    async fn batch_insert(&self, records: &[Record], pool: &Pool<MySql>) -> ResultMqttBrokerError {
        debug!("Processing batch insert of {} records", records.len());

        let mut values_placeholders = Vec::with_capacity(records.len());
        let mut bindings = Vec::with_capacity(records.len());
        let mut failed_serializations = Vec::new();

        for (idx, record) in records.iter().enumerate() {
            let payload = match serde_json::to_string(record) {
                Ok(p) => p,
                Err(e) => {
                    warn!(
                        "Failed to serialize record {}/{} (key: '{}', timestamp: {}): {}",
                        idx + 1,
                        records.len(),
                        record.key,
                        record.timestamp,
                        e
                    );
                    failed_serializations.push((idx, e.to_string()));
                    continue;
                }
            };
            values_placeholders.push("(?, ?, ?)");
            bindings.push((record.key.clone(), payload, record.timestamp as i64));
        }

        if bindings.is_empty() {
            return Err(crate::handler::error::MqttBrokerError::CommonError(
                format!("All {} records failed to serialize", records.len()),
            ));
        }

        if !failed_serializations.is_empty() {
            warn!(
                "{}/{} records failed serialization and will not be inserted",
                failed_serializations.len(),
                records.len()
            );
        }

        let base_sql = format!(
            "INSERT INTO `{}` (record_key, payload, timestamp) VALUES {}",
            self.config.table,
            values_placeholders.join(", ")
        );

        let sql = if self.config.is_upsert_enabled() {
            format!(
                "{} AS new_vals ON DUPLICATE KEY UPDATE payload = new_vals.payload, timestamp = new_vals.timestamp",
                base_sql
            )
        } else {
            base_sql
        };

        debug!(
            "Batch inserting {} records into table {}",
            bindings.len(),
            self.config.table
        );

        let mut query = sqlx::query(&sql);
        for (key, payload, timestamp) in bindings.iter() {
            query = query.bind(key).bind(payload).bind(timestamp);
        }

        match query.execute(pool).await {
            Ok(result) => {
                info!(
                    "Successfully batch inserted {} records into table {} (affected rows: {})",
                    bindings.len(),
                    self.config.table,
                    result.rows_affected()
                );
                Ok(())
            }
            Err(e) => {
                let error_msg = format!(
                    "Failed to batch insert {} records into table {}: {}",
                    bindings.len(),
                    self.config.table,
                    e
                );
                error!("{}", error_msg);
                Err(crate::handler::error::MqttBrokerError::CommonError(
                    error_msg,
                ))
            }
        }
    }
}

#[async_trait]
impl ConnectorSink for MySQLBridgePlugin {
    type SinkResource = Pool<MySql>;

    async fn validate(&self) -> ResultMqttBrokerError {
        info!(
            "Validating MySQL connector configuration for {}:{}/{}",
            self.config.host, self.config.port, self.config.database
        );

        let pool = self.create_pool().await.map_err(|e| {
            crate::handler::error::MqttBrokerError::CommonError(format!(
                "Failed to create MySQL connection pool: {}",
                e
            ))
        })?;

        sqlx::query("SELECT 1").execute(&pool).await.map_err(|e| {
            crate::handler::error::MqttBrokerError::CommonError(format!(
                "Failed to ping MySQL server at {}:{}/{}: {}",
                self.config.host, self.config.port, self.config.database, e
            ))
        })?;

        let table_check_sql =
            "SELECT 1 FROM information_schema.tables WHERE table_schema = ? AND table_name = ?"
                .to_string();

        let table_name = if self.config.table.contains('.') {
            let parts: Vec<&str> = self.config.table.split('.').collect();
            parts.get(1).unwrap_or(&parts[0]).to_string()
        } else {
            self.config.table.clone()
        };

        let table_exists = sqlx::query(&table_check_sql)
            .bind(&self.config.database)
            .bind(&table_name)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                crate::handler::error::MqttBrokerError::CommonError(format!(
                    "Failed to check if table {} exists: {}",
                    self.config.table, e
                ))
            })?;

        if table_exists.is_none() {
            warn!(
                "Table {} does not exist in database {}. It must be created before starting the connector.",
                self.config.table, self.config.database
            );
        }

        pool.close().await;

        info!(
            "Successfully validated MySQL connection to {}:{}/{}, table: {}",
            self.config.host, self.config.port, self.config.database, self.config.table
        );

        Ok(())
    }

    async fn init_sink(
        &self,
    ) -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError> {
        info!(
            "Initializing MySQL sink: {}:{}/{}.{}",
            self.config.host, self.config.port, self.config.database, self.config.table
        );
        let pool = self.create_pool().await?;
        info!(
            "MySQL sink initialized successfully (pool size: {}-{})",
            self.config.min_pool_size,
            self.config.get_pool_size()
        );
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

        let mode = if self.config.is_batch_insert_enabled() {
            "batch"
        } else {
            "single"
        };

        debug!(
            "Sending {} records to MySQL in {} mode",
            records.len(),
            mode
        );

        if self.config.is_batch_insert_enabled() {
            self.batch_insert(records, pool).await
        } else {
            self.single_insert(records, pool).await
        }
    }

    async fn cleanup_sink(&self, pool: Pool<MySql>) -> ResultMqttBrokerError {
        info!(
            "Closing MySQL connection pool for {}:{}/{}",
            self.config.host, self.config.port, self.config.database
        );
        pool.close().await;
        info!("MySQL connection pool closed successfully");
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

        let batch_size = mysql_config.batch_size as u64;
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
                record_num: batch_size,
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
