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
};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};
use tokio_postgres::{Client, NoTls};
use tracing::{error, info};

use crate::common::types::ResultMqttBrokerError;
use crate::storage::message::MessageStorage;

use super::{
    core::{BridgePlugin, BridgePluginReadConfig},
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

    pub async fn connect(&self) -> Result<Client, common_base::error::common::CommonError> {
        let connection_string = self.config.connection_string();

        let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
            .await
            .map_err(|e| {
                error!("Failed to connect to PostgreSQL: {}", e);
                common_base::error::common::CommonError::CommonError(format!(
                    "PostgreSQL connection error: {}",
                    e
                ))
            })?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                error!("PostgreSQL connection error: {}", e);
            }
        });

        Ok(client)
    }

    pub async fn append(&self, records: &Vec<Record>, client: &Client) -> ResultMqttBrokerError {
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

            let sql = format!(
                "INSERT INTO {} (client_id, topic, timestamp, payload, data) VALUES ($1, $2, $3, $4, $5)",
                self.config.table
            );

            client
                .execute(
                    &sql,
                    &[&client_id, &topic, &timestamp, &payload_str, &record.data],
                )
                .await
                .map_err(|e| {
                    error!("Failed to execute PostgreSQL query: {}", e);
                    common_base::error::common::CommonError::CommonError(format!(
                        "PostgreSQL execution error: {}",
                        e
                    ))
                })?;
        }
        Ok(())
    }
}

#[async_trait]
impl BridgePlugin for PostgresBridgePlugin {
    async fn exec(&self, config: BridgePluginReadConfig) -> ResultMqttBrokerError {
        let message_storage = MessageStorage::new(self.message_storage.clone());
        let group_name = self.connector_name.clone();
        let mut recv = self.stop_send.subscribe();

        let client = match self.connect().await {
            Ok(client) => client,
            Err(e) => {
                error!(
                    "Connector {} failed to connect to PostgreSQL: {}",
                    self.connector_name, e
                );
                return Err(e.into());
            }
        };

        info!(
            "Connector {} successfully connected to PostgreSQL database: {}",
            self.connector_name, self.config.database
        );

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

                            if let Err(e) = self.append(&data, &client).await {
                                error!(
                                    "Connector {} failed to write data to PostgreSQL table {}, error: {}",
                                    self.connector_name, self.config.table, e
                                );
                                sleep(Duration::from_millis(100)).await;
                            } else {
                                info!(
                                    "Connector {} successfully wrote {} records to PostgreSQL table {}",
                                    self.connector_name, data.len(), self.config.table
                                );
                            }

                            // commit offset
                            message_storage
                                .commit_group_offset(&group_name, &config.topic_id, offset + data.len() as u64)
                                .await?;
                        }
                        Err(e) => {
                            error!(
                                "Connector {} failed to read Topic {} data with error: {}",
                                self.connector_name, config.topic_id, e
                            );
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
