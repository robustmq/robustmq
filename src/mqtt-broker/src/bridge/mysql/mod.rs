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
use metadata_struct::{adapter::record::Record, mqtt::bridge::config_mysql::MySQLConnectorConfig};
use sqlx::{mysql::MySqlPoolOptions, MySql, Pool};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};
use tracing::{error, info, warn};

use crate::common::types::ResultMqttBrokerError;
use crate::storage::message::MessageStorage;

use super::{
    core::{BridgePlugin, BridgePluginReadConfig},
    manager::ConnectorManager,
};

pub struct MySQLBridgePlugin {
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector_name: String,
    config: MySQLConnectorConfig,
    stop_send: broadcast::Sender<bool>,
}

impl MySQLBridgePlugin {
    pub fn new(
        connector_manager: Arc<ConnectorManager>,
        message_storage: ArcStorageAdapter,
        connector_name: String,
        config: MySQLConnectorConfig,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        MySQLBridgePlugin {
            connector_manager,
            message_storage,
            connector_name,
            config,
            stop_send,
        }
    }

    async fn create_pool(&self) -> Result<Pool<MySql>, sqlx::Error> {
        MySqlPoolOptions::new()
            .max_connections(self.config.get_pool_size())
            .connect(&self.config.connection_url())
            .await
    }

    pub async fn append(&self, records: &Vec<Record>, pool: &Pool<MySql>) -> ResultMqttBrokerError {
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
        pool: &Pool<MySql>,
    ) -> ResultMqttBrokerError {
        for record in records {
            let payload = serde_json::to_string(record)?;

            let base_sql = if let Some(template) = &self.config.sql_template {
                let placeholder_count = template.matches('?').count();
                if placeholder_count != 3 {
                    warn!(
                        "Connector {}: sql_template expects 3 placeholders, but found {}. Using provided template with binds (key, payload, timestamp) in order.",
                        self.connector_name,
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

    async fn batch_insert(
        &self,
        records: &Vec<Record>,
        pool: &Pool<MySql>,
    ) -> ResultMqttBrokerError {
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
impl BridgePlugin for MySQLBridgePlugin {
    async fn exec(&self, config: BridgePluginReadConfig) -> ResultMqttBrokerError {
        let message_storage = MessageStorage::new(self.message_storage.clone());
        let group_name = self.connector_name.clone();
        let mut recv = self.stop_send.subscribe();
        let pool = match self.create_pool().await {
            Ok(p) => p,
            Err(e) => {
                error!(
                    "Connector {} failed to create MySQL connection pool: {}",
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

                val = message_storage.read_topic_message(&config.topic_id, offset, config.record_num) => {
                    match val {
                        Ok(data) => {
                            self.connector_manager.report_heartbeat(&self.connector_name);
                            if data.is_empty() {
                                sleep(Duration::from_millis(100)).await;
                                continue;
                            }

                            if let Err(e) = self.append(&data, &pool).await {
                                error!("Connector {} failed to write data to MySQL table {}, error message: {}", self.connector_name, self.config.table, e);
                                sleep(Duration::from_millis(100)).await;
                            }

                            // commit offset
                            message_storage.commit_group_offset(&group_name, &config.topic_id, offset + data.len() as u64).await?;
                        }
                        Err(e) => {
                            error!("Connector {} failed to read Topic {} data with error message :{}", self.connector_name, config.topic_id, e);
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
