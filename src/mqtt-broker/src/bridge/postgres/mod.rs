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
    mqtt::bridge::config_postgres::PostgresConnectorConfig, mqtt::bridge::connector::MQTTConnector,
    storage::adapter_record::AdapterWriteRecord,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use storage_adapter::driver::{ArcStorageAdapter, StorageDriverManager};
use tracing::{error, warn};

use crate::handler::tool::ResultMqttBrokerError;

use super::{
    core::{run_connector_loop, BridgePluginReadConfig, BridgePluginThread, ConnectorSink},
    manager::ConnectorManager,
};

pub struct PostgresBridgePlugin {
    config: PostgresConnectorConfig,
}

impl PostgresBridgePlugin {
    pub fn new(config: PostgresConnectorConfig) -> Self {
        PostgresBridgePlugin { config }
    }

    async fn create_pool(&self) -> Result<Pool<Postgres>, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(self.config.get_pool_size())
            .connect(&self.config.connection_url())
            .await
    }

    async fn single_insert(
        &self,
        records: &[AdapterWriteRecord],
        pool: &Pool<Postgres>,
    ) -> ResultMqttBrokerError {
        for record in records {
            let client_id = &record.key;
            let timestamp = record.timestamp as i64;
            let topic = record
                .header
                .as_ref()
                .and_then(|headers| headers.iter().find(|h| h.name == "topic"))
                .map(|h| h.value.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let payload_str = String::from_utf8_lossy(&record.data).to_string();

            let base_sql = if let Some(template) = &self.config.sql_template {
                let placeholder_count = (1..=10)
                    .filter(|i| template.contains(&format!("${}", i)))
                    .count();
                if placeholder_count != 5 {
                    warn!(
                        "sql_template expects 5 placeholders, but found {}. Using provided template with binds (client_id, topic, timestamp, payload, data) in order.",
                        placeholder_count
                    );
                }
                template.clone()
            } else {
                format!(
                    "INSERT INTO {} (client_id, topic, timestamp, payload, data) VALUES ($1, $2, $3, $4, $5)",
                    self.config.table
                )
            };

            let sql = if self.config.is_upsert_enabled() {
                let conflict_columns = self
                    .config
                    .conflict_columns
                    .as_deref()
                    .unwrap_or("client_id, topic");

                format!(
                    "{} ON CONFLICT ({}) DO UPDATE SET timestamp = EXCLUDED.timestamp, payload = EXCLUDED.payload, data = EXCLUDED.data",
                    base_sql, conflict_columns
                )
            } else {
                base_sql
            };

            sqlx::query(&sql)
                .bind(client_id)
                .bind(&topic)
                .bind(timestamp)
                .bind(&payload_str)
                .bind(record.data.as_ref())
                .execute(pool)
                .await?;
        }

        Ok(())
    }

    async fn batch_insert(
        &self,
        records: &[AdapterWriteRecord],
        pool: &Pool<Postgres>,
    ) -> ResultMqttBrokerError {
        let mut value_placeholders = Vec::with_capacity(records.len());
        let mut param_index = 1;

        let mut client_ids = Vec::with_capacity(records.len());
        let mut topics = Vec::with_capacity(records.len());
        let mut timestamps = Vec::with_capacity(records.len());
        let mut payloads = Vec::with_capacity(records.len());
        let mut data_vec = Vec::with_capacity(records.len());

        for record in records {
            let client_id = record.key.clone();
            let timestamp = record.timestamp as i64;
            let topic = record
                .header
                .as_ref()
                .and_then(|headers| headers.iter().find(|h| h.name == "topic"))
                .map(|h| h.value.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let payload_str = String::from_utf8_lossy(&record.data).to_string();
            let data = record.data.clone();

            client_ids.push(client_id);
            topics.push(topic);
            timestamps.push(timestamp);
            payloads.push(payload_str);
            data_vec.push(data);

            value_placeholders.push(format!(
                "(${}, ${}, ${}, ${}, ${})",
                param_index,
                param_index + 1,
                param_index + 2,
                param_index + 3,
                param_index + 4
            ));
            param_index += 5;
        }

        let base_sql = format!(
            "INSERT INTO {} (client_id, topic, timestamp, payload, data) VALUES {}",
            self.config.table,
            value_placeholders.join(", ")
        );

        let sql = if self.config.is_upsert_enabled() {
            let conflict_columns = self
                .config
                .conflict_columns
                .as_deref()
                .unwrap_or("client_id, topic");

            format!(
                "{} ON CONFLICT ({}) DO UPDATE SET timestamp = EXCLUDED.timestamp, payload = EXCLUDED.payload, data = EXCLUDED.data",
                base_sql, conflict_columns
            )
        } else {
            base_sql
        };

        let mut query = sqlx::query(&sql);
        for i in 0..records.len() {
            query = query
                .bind(&client_ids[i])
                .bind(&topics[i])
                .bind(timestamps[i])
                .bind(&payloads[i])
                .bind(data_vec[i].as_ref());
        }

        query.execute(pool).await?;

        Ok(())
    }
}

#[async_trait]
impl ConnectorSink for PostgresBridgePlugin {
    type SinkResource = Pool<Postgres>;

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
        records: &[AdapterWriteRecord],
        pool: &mut Pool<Postgres>,
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

    async fn cleanup_sink(&self, pool: Pool<Postgres>) -> ResultMqttBrokerError {
        pool.close().await;
        Ok(())
    }
}

pub fn start_postgres_connector(
    connector_manager: Arc<ConnectorManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        let postgres_config = match &connector.config {
            metadata_struct::mqtt::bridge::ConnectorConfig::Postgres(config) => config.clone(),
            _ => {
                error!("Invalid connector config type, expected Postgres config");
                return;
            }
        };
        let bridge = PostgresBridgePlugin::new(postgres_config);

        let stop_recv = thread.stop_send.subscribe();
        connector_manager.add_connector_thread(&connector.connector_name, thread);

        if let Err(e) = run_connector_loop(
            &bridge,
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
                "Failed to start PostgresBridgePlugin with error message: {:?}",
                e
            );
        }
    });
}
