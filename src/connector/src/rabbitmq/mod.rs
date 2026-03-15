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

use async_trait::async_trait;
use grpc_clients::pool::ClientPool;
use lapin::{
    options::{BasicPublishOptions, ConfirmSelectOptions},
    BasicProperties, Channel, Connection, ConnectionProperties,
};
use metadata_struct::{
    connector::config_rabbitmq::RabbitMQConnectorConfig, connector::MQTTConnector,
    storage::adapter_record::AdapterWriteRecord,
};
use rule_engine::apply_rule_engine;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error, info, warn};

use common_base::error::common::CommonError;

use super::{
    core::{BridgePluginReadConfig, BridgePluginThread},
    failure::FailureRecordInfo,
    loops::run_connector_loop,
    manager::ConnectorManager,
    traits::ConnectorSink,
};

pub struct RabbitMQBridgePlugin {
    connector: MQTTConnector,
    config: RabbitMQConnectorConfig,
}

impl RabbitMQBridgePlugin {
    #[allow(clippy::result_large_err)]
    pub fn new(connector: MQTTConnector) -> Result<Self, CommonError> {
        let config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::RabbitMQ(config) => config.clone(),
            _ => {
                return Err(CommonError::CommonError(
                    "invalid connector type for rabbitmq plugin".to_string(),
                ));
            }
        };
        Ok(RabbitMQBridgePlugin { connector, config })
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

    async fn validate(&self) -> Result<(), CommonError> {
        info!(
            "Validating RabbitMQ connector configuration for {}:{}/{}",
            self.config.server, self.config.port, self.config.virtual_host
        );

        let uri = self.build_connection_uri();
        let connection = Connection::connect(&uri, ConnectionProperties::default())
            .await
            .map_err(|e| {
                CommonError::CommonError(format!(
                    "Failed to connect to RabbitMQ at {}:{}: {}",
                    self.config.server, self.config.port, e
                ))
            })?;

        let channel = connection
            .create_channel()
            .await
            .map_err(|e| CommonError::CommonError(format!("Failed to create channel: {}", e)))?;

        channel.close(200, "Validation complete").await.ok();
        connection.close(200, "Validation complete").await.ok();

        info!(
            "Successfully validated RabbitMQ connection to {}:{}/{}, exchange: {}",
            self.config.server, self.config.port, self.config.virtual_host, self.config.exchange
        );

        Ok(())
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
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
    ) -> Result<Vec<FailureRecordInfo>, CommonError> {
        if records.is_empty() {
            return Ok(vec![]);
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
            let processed_data =
                match apply_rule_engine(&self.connector.etl_rule, &record.data).await {
                    Ok(data) => data,
                    Err(e) => {
                        warn!(
                            "Failed to apply rule for record {}/{} (key: '{:?}'): {}",
                            idx + 1,
                            records.len(),
                            record.key,
                            e
                        );
                        failed_records.push((idx, e.to_string()));
                        continue;
                    }
                };

            let mut processed_record = record.clone();
            processed_record.data = processed_data;

            let data = match serde_json::to_string(&processed_record) {
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
            return Err(CommonError::CommonError(format!(
                "All {} records failed to send to RabbitMQ",
                records.len()
            )));
        }

        let mut fail_messages = Vec::with_capacity(failed_records.len());
        for (idx, error_message) in failed_records {
            if let Some(record) = records.get(idx) {
                fail_messages.push(FailureRecordInfo {
                    tenant: self.connector.tenant.clone(),
                    connector_name: self.connector.connector_name.clone(),
                    connector_type: self.connector.connector_type.to_string(),
                    source_topic: self.connector.topic_name.clone(),
                    error_message,
                    records: vec![record.clone()],
                });
            }
        }
        Ok(fail_messages)
    }

    async fn cleanup_sink(&self, channel: Channel) -> Result<(), CommonError> {
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
    stop_recv: Receiver<bool>,
) {
    tokio::spawn(Box::pin(async move {
        let connector_name = connector.connector_name.clone();
        let connector_type = connector.connector_type.to_string();
        let bridge = match RabbitMQBridgePlugin::new(connector.clone()) {
            Ok(bridge) => bridge,
            Err(e) => {
                error!(
                    "Invalid connector config type for RabbitMQ connector, connector_name='{}', connector_type='{}', error={}",
                    connector_name, connector_type, e
                );
                return;
            }
        };
        let batch_size = bridge.config.batch_size as u64;
        connector_manager.add_connector_thread(
            &connector.tenant,
            &connector.connector_name,
            thread,
        );

        if let Err(e) = run_connector_loop(
            &bridge,
            &client_pool,
            &connector_manager,
            &storage_driver_manager,
            connector.connector_name.clone(),
            BridgePluginReadConfig {
                tenant: connector.tenant,
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
                "Failed to start RabbitMQBridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}
