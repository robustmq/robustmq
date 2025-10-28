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
use lapin::{
    options::{BasicPublishOptions, ConfirmSelectOptions},
    BasicProperties, Connection, ConnectionProperties,
};
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_rabbitmq::RabbitMQConnectorConfig,
};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};
use tracing::{error, info};

use crate::common::types::ResultMqttBrokerError;
use crate::storage::message::MessageStorage;

use super::{
    core::{BridgePlugin, BridgePluginReadConfig},
    manager::ConnectorManager,
};

pub struct RabbitMQBridgePlugin {
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector_name: String,
    config: RabbitMQConnectorConfig,
    stop_send: broadcast::Sender<bool>,
}

impl RabbitMQBridgePlugin {
    pub fn new(
        connector_manager: Arc<ConnectorManager>,
        message_storage: ArcStorageAdapter,
        connector_name: String,
        config: RabbitMQConnectorConfig,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        RabbitMQBridgePlugin {
            connector_manager,
            message_storage,
            connector_name,
            config,
            stop_send,
        }
    }

    pub async fn append(
        &self,
        records: &Vec<Record>,
        connection: &Connection,
    ) -> ResultMqttBrokerError {
        let channel = connection.create_channel().await?;

        channel
            .confirm_select(ConfirmSelectOptions::default())
            .await?;

        for record in records {
            let data = serde_json::to_string(record)?;

            let properties = BasicProperties::default()
                .with_delivery_mode(self.config.delivery_mode.to_amqp_value());

            let confirm = channel
                .basic_publish(
                    &self.config.exchange,
                    &self.config.routing_key,
                    BasicPublishOptions::default(),
                    data.as_bytes(),
                    properties,
                )
                .await?;

            confirm.await?;
        }

        Ok(())
    }

    fn build_connection_uri(&self) -> String {
        let protocol = if self.config.enable_tls {
            "amqps"
        } else {
            "amqp"
        };
        format!(
            "{}://{}:{}@{}:{}/{}",
            protocol,
            self.config.username,
            self.config.password,
            self.config.server,
            self.config.port,
            self.config.virtual_host
        )
    }
}

#[async_trait]
impl BridgePlugin for RabbitMQBridgePlugin {
    async fn exec(&self, config: BridgePluginReadConfig) -> ResultMqttBrokerError {
        let message_storage = MessageStorage::new(self.message_storage.clone());
        let group_name = self.connector_name.clone();
        let mut recv = self.stop_send.subscribe();

        let uri = self.build_connection_uri();

        info!(
            "Connecting to RabbitMQ at {}:{} (exchange: {}, routing_key: {})",
            self.config.server, self.config.port, self.config.exchange, self.config.routing_key
        );

        let connection = Connection::connect(&uri, ConnectionProperties::default()).await?;

        info!("Successfully connected to RabbitMQ");

        loop {
            let offset = message_storage.get_group_offset(&group_name).await?;

            select! {
                val = recv.recv() => {
                    if let Ok(flag) = val {
                        if flag {
                            info!("RabbitMQ connector thread exited successfully");
                            if let Err(e) = connection.close(200, "Normal shutdown").await {
                                error!("Error closing RabbitMQ connection: {}", e);
                            }
                            break;
                        }
                    }
                }

                val = message_storage.read_topic_message(&config.topic_name, offset, config.record_num) => {
                    match val {
                        Ok(data) => {
                            self.connector_manager.report_heartbeat(&self.connector_name);

                            if data.is_empty() {
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                continue;
                            }

                            if let Err(e) = self.append(&data, &connection).await {
                                error!(
                                    "Connector {} failed to write data to RabbitMQ exchange {}, error: {}",
                                    self.connector_name, self.config.exchange, e
                                );
                                sleep(Duration::from_millis(100)).await;
                            }
                        },
                        Err(e) => {
                            error!(
                                "Connector {} failed to read Topic {} data with error: {}",
                                self.connector_name, config.topic_name, e
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
