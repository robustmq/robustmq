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

use std::{sync::Arc, time::Duration};

use axum::async_trait;
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_postgres::PostgresConnectorConfig,
    mqtt::bridge::connector::MQTTConnector,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};
use tracing::{error, info, warn};

use crate::common::types::ResultMqttBrokerError;
use crate::storage::message::MessageStorage;

use super::{
    core::{BridgePlugin, BridgePluginReadConfig, BridgePluginThread},
    manager::ConnectorManager,
};

pub struct PostgresBridgePlugin {
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector_name: String,
    config: PostgresConnectorConfig,
    stop_send: broadcast::Sender<bool>,
}

impl PostgresBridgePlugin {
    pub fn new(
        connector_manager: Arc<ConnectorManager>,
        message_storage: ArcStorageAdapter,
        connector_name: String,
        config: PostgresConnectorConfig,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        PostgresBridgePlugin {
            connector_manager,
            message_storage,
            connector_name,
            config,
            stop_send,
        }
    }

    async fn create_pool(&self) -> Result<Pool<Postgres>, sqlx::Error> {
        PgPoolOptions::new()
            .max_connections(self.config.get_pool_size())
            .connect(&self.config.connection_url())
            .await
    }

    pub async fn append(
        &self,
        records: &Vec<Record>,
        pool: &Pool<Postgres>,
    ) -> ResultMqttBrokerError {
        if records.is_empty() {
            return Ok(());
        }

        if self.config.is_batch_insert_enabled() {
            if self.config.sql_template.is_some() {
                warn!(
                    "Connector {}: sql_template is not applied in batch mode; default batch INSERT will be used",
                    self.connector_name
                );
            }
            self.batch_insert(records, pool).await
        } else {
            self.single_insert(records, pool).await
        }
    }

    async fn single_insert(
        &self,
        records: &Vec<Record>,
        pool: &Pool<Postgres>,
    ) -> ResultMqttBrokerError {
        for record in records {
            let client_id = &record.key;
            let timestamp = record.timestamp as i64;
            let topic = record
                .header
                .iter()
                .find(|h| h.name == "topic")
                .map(|h| h.value.clone())
                .unwrap_or_else(|| "unknown".to_string());
            let payload_str = String::from_utf8_lossy(&record.data).to_string();

            let base_sql = if let Some(template) = &self.config.sql_template {
                let placeholder_count = (1..=10)
                    .filter(|i| template.contains(&format!("${}", i)))
                    .count();
                if placeholder_count != 5 {
                    warn!(
                        "Connector {}: sql_template expects 5 placeholders, but found {}. Using provided template with binds (client_id, topic, timestamp, payload, data) in order.",
                        self.connector_name,
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
                .bind(&record.data)
                .execute(pool)
                .await?;
        }

        Ok(())
    }

    async fn batch_insert(
        &self,
        records: &Vec<Record>,
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
                .iter()
                .find(|h| h.name == "topic")
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
                .bind(&data_vec[i]);
        }

        query.execute(pool).await?;

        Ok(())
    }
}

pub fn start_postgres_connector(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
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
                topic_name: connector.topic_name,
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
    });
}

#[async_trait]
impl BridgePlugin for PostgresBridgePlugin {
    async fn exec(&self, config: BridgePluginReadConfig) -> ResultMqttBrokerError {
        let message_storage = MessageStorage::new(self.message_storage.clone());
        let group_name = self.connector_name.clone();
        let mut recv = self.stop_send.subscribe();

        let pool = match self.create_pool().await {
            Ok(p) => p,
            Err(e) => {
                error!(
                    "Connector {} failed to create PostgreSQL connection pool: {}",
                    self.connector_name, e
                );
                return Err(e.into());
            }
        };

        loop {
            let offset = message_storage.get_group_offset(&group_name).await?;

            select! {
                val = recv.recv() => {
                    if let Ok(flag) = val {
                        if flag {
                            info!("Connector {} thread exited successfully", self.connector_name);
                            break;
                        }
                    }
                }

                val = message_storage.read_topic_message(&config.topic_name, offset, config.record_num) => {
                    match val {
                        Ok(data) => {
                            self.connector_manager.report_heartbeat(&self.connector_name);

                            if data.is_empty() {
                                sleep(Duration::from_millis(100)).await;
                                continue;
                            }

                            if let Err(e) = self.append(&data, &pool).await {
                                error!("Connector {} failed to write data to PostgreSQL table {}, error message: {}", self.connector_name, self.config.table, e);
                                sleep(Duration::from_millis(100)).await;
                            }

                            // commit offset
                            message_storage
                                .commit_group_offset(&group_name, &config.topic_name, offset + data.len() as u64)
                                .await?;
                        }
                        Err(e) => {
                            error!("Connector {} failed to read Topic {} data with error message :{}", self.connector_name, config.topic_name, e);
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
