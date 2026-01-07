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
use grpc_clients::pool::ClientPool;
use lapin::{
    options::{BasicPublishOptions, ConfirmSelectOptions},
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use metadata_struct::{
    mqtt::bridge::config_rabbitmq::RabbitMQConnectorConfig, mqtt::bridge::connector::MQTTConnector,
    storage::adapter_record::AdapterWriteRecord,
};
use storage_adapter::driver::StorageDriverManager;
use tracing::{debug, error, info, warn};

use crate::handler::tool::ResultMqttBrokerError;

use super::{
    core::{run_connector_loop, BridgePluginReadConfig, BridgePluginThread, ConnectorSink},
    manager::ConnectorManager,
};

pub struct RabbitMQBridgePlugin {
    config: RabbitMQConnectorConfig,
}

impl RabbitMQBridgePlugin {
    pub fn new(config: RabbitMQConnectorConfig) -> Self {
        RabbitMQBridgePlugin { config }
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
impl ConnectorSink for RabbitMQBridgePlugin {
    type SinkResource = Channel;

    async fn validate(&self) -> ResultMqttBrokerError {
        info!(
            "Validating RabbitMQ connector configuration for {}:{}/{}",
            self.config.server, self.config.port, self.config.virtual_host
        );

        let uri = self.build_connection_uri();
        let connection = Connection::connect(&uri, ConnectionProperties::default())
            .await
            .map_err(|e| {
                crate::handler::error::MqttBrokerError::CommonError(format!(
                    "Failed to connect to RabbitMQ at {}:{}: {}",
                    self.config.server, self.config.port, e
                ))
            })?;

        let channel = connection.create_channel().await.map_err(|e| {
            crate::handler::error::MqttBrokerError::CommonError(format!(
                "Failed to create channel: {}",
                e
            ))
        })?;

        channel.close(200, "Validation complete").await.ok();
        connection.close(200, "Validation complete").await.ok();

        info!(
            "Successfully validated RabbitMQ connection to {}:{}/{}, exchange: {}",
            self.config.server, self.config.port, self.config.virtual_host, self.config.exchange
        );

        Ok(())
    }

    async fn init_sink(
        &self,
    ) -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError> {
        info!(
            "Initializing RabbitMQ channel: {}:{}/{} exchange: {} (timeout: {}s, heartbeat: {}s)",
            self.config.server,
            self.config.port,
            self.config.virtual_host,
            self.config.exchange,
            self.config.connection_timeout_secs,
            self.config.heartbeat_secs
        );

        let uri = self.build_connection_uri();

        let connection = Connection::connect(&uri, ConnectionProperties::default()).await?;

        let channel = connection.create_channel().await?;

        if self.config.publisher_confirms {
            channel
                .confirm_select(ConfirmSelectOptions::default())
                .await?;
            debug!("Publisher confirms enabled");
        }

        info!(
            "RabbitMQ channel initialized successfully (publisher_confirms: {})",
            self.config.publisher_confirms
        );

        Ok(channel)
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        channel: &mut Channel,
    ) -> ResultMqttBrokerError {
        if records.is_empty() {
            return Ok(());
        }

        debug!(
            "Sending {} records to RabbitMQ exchange {}",
            records.len(),
            self.config.exchange
        );

        let mut success_count = 0;
        let mut failed_records = Vec::new();
        let mut confirms = Vec::new();

        for (idx, record) in records.iter().enumerate() {
            let data = match serde_json::to_string(record) {
                Ok(d) => d,
                Err(e) => {
                    warn!(
                        "Failed to serialize record {}/{} (key: '{:?}'): {}",
                        idx + 1,
                        records.len(),
                        record.key,
                        e
                    );
                    failed_records.push((idx, e.to_string()));
                    continue;
                }
            };

            let properties = BasicProperties::default()
                .with_delivery_mode(self.config.delivery_mode.to_amqp_value());

            match channel
                .basic_publish(
                    &self.config.exchange,
                    &self.config.routing_key,
                    BasicPublishOptions::default(),
                    data.as_bytes(),
                    properties,
                )
                .await
            {
                Ok(confirm) => {
                    if self.config.publisher_confirms {
                        confirms.push((idx, confirm));
                    } else {
                        success_count += 1;
                        debug!(
                            "Published record {}/{} (no confirm)",
                            idx + 1,
                            records.len()
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to publish record {}/{} (key: '{:?}') to exchange {}: {}",
                        idx + 1,
                        records.len(),
                        record.key,
                        self.config.exchange,
                        e
                    );
                    failed_records.push((idx, e.to_string()));
                }
            }
        }

        if self.config.publisher_confirms && !confirms.is_empty() {
            debug!("Waiting for {} publisher confirms", confirms.len());

            for (idx, confirm) in confirms {
                match tokio::time::timeout(
                    Duration::from_secs(self.config.confirm_timeout_secs),
                    confirm,
                )
                .await
                {
                    Ok(Ok(_)) => {
                        success_count += 1;
                        debug!("Confirmed record {}/{}", idx + 1, records.len());
                    }
                    Ok(Err(e)) => {
                        warn!(
                            "Failed to confirm record {}/{}: {}",
                            idx + 1,
                            records.len(),
                            e
                        );
                        failed_records.push((idx, e.to_string()));
                    }
                    Err(_) => {
                        warn!(
                            "Timeout waiting for confirm of record {}/{} (timeout: {}s)",
                            idx + 1,
                            records.len(),
                            self.config.confirm_timeout_secs
                        );
                        failed_records.push((idx, "Confirm timeout".to_string()));
                    }
                }
            }
        }

        if success_count > 0 {
            info!(
                "Sent {}/{} records successfully to RabbitMQ exchange {}",
                success_count,
                records.len(),
                self.config.exchange
            );
        }

        if failed_records.len() == records.len() {
            return Err(crate::handler::error::MqttBrokerError::CommonError(
                format!("All {} records failed to send to RabbitMQ", records.len()),
            ));
        }

        Ok(())
    }

    async fn cleanup_sink(&self, channel: Channel) -> ResultMqttBrokerError {
        info!(
            "Closing RabbitMQ channel for exchange {}",
            self.config.exchange
        );

        match channel.close(200, "Normal shutdown").await {
            Ok(_) => {
                info!("RabbitMQ channel closed successfully");
            }
            Err(e) => {
                warn!("Failed to close RabbitMQ channel cleanly: {}", e);
            }
        }

        Ok(())
    }
}

pub fn start_rabbitmq_connector(
    client_pool: Arc<ClientPool>,
    connector_manager: Arc<ConnectorManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        let rabbitmq_config = match &connector.config {
            metadata_struct::mqtt::bridge::ConnectorConfig::RabbitMQ(config) => config.clone(),
            _ => {
                error!("Invalid connector config type, expected RabbitMQ config");
                return;
            }
        };

        let batch_size = rabbitmq_config.batch_size as u64;
        let bridge = RabbitMQBridgePlugin::new(rabbitmq_config);

        let stop_recv = thread.stop_send.subscribe();
        connector_manager.add_connector_thread(&connector.connector_name, thread);

        if let Err(e) = run_connector_loop(
            &bridge,
            &client_pool,
            &connector_manager,
            storage_driver_manager.clone(),
            connector.connector_name.clone(),
            BridgePluginReadConfig {
                topic_name: connector.topic_name,
                record_num: batch_size,
                strategy: connector.failure_strategy,
            },
            stop_recv,
        )
        .await
        {
            connector_manager.remove_connector_thread(&connector.connector_name);
            error!(
                "Failed to start RabbitMQBridgePlugin with error message: {:?}",
                e
            );
        }
    });
}
