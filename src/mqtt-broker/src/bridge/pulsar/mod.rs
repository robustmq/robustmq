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

use crate::{
    bridge::{
        core::{run_connector_loop, BridgePluginReadConfig, BridgePluginThread, ConnectorSink},
        failure::FailureHandlingStrategy,
        manager::ConnectorManager,
    },
    common::types::ResultMqttBrokerError,
};
use axum::async_trait;
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_pulsar::PulsarConnectorConfig,
    mqtt::bridge::connector::MQTTConnector,
};
use storage_adapter::storage::ArcStorageAdapter;
use tracing::{debug, error, info, warn};
mod pulsar_producer;

pub struct PulsarBridgePlugin {
    config: PulsarConnectorConfig,
}

impl PulsarBridgePlugin {
    pub fn new(config: PulsarConnectorConfig) -> Self {
        PulsarBridgePlugin { config }
    }
}

#[async_trait]
impl ConnectorSink for PulsarBridgePlugin {
    type SinkResource = pulsar::producer::Producer<pulsar::TokioExecutor>;

    async fn validate(&self) -> ResultMqttBrokerError {
        info!(
            "Validating Pulsar connector configuration for {} topic: {}",
            self.config.server, self.config.topic
        );

        let producer = pulsar_producer::Producer::new(&self.config)
            .build_producer()
            .await
            .map_err(|e| {
                crate::handler::error::MqttBrokerError::CommonError(format!(
                    "Failed to connect to Pulsar at {}: {}",
                    self.config.server, e
                ))
            })?;

        drop(producer);

        info!(
            "Successfully validated Pulsar connection to {}, topic: {}",
            self.config.server, self.config.topic
        );

        Ok(())
    }

    async fn init_sink(
        &self,
    ) -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError> {
        info!(
            "Initializing Pulsar producer: {} topic: {}",
            self.config.server, self.config.topic
        );
        let producer = pulsar_producer::Producer::new(&self.config)
            .build_producer()
            .await?;
        info!("Pulsar producer initialized successfully");
        Ok(producer)
    }

    async fn send_batch(
        &self,
        records: &[Record],
        producer: &mut pulsar::producer::Producer<pulsar::TokioExecutor>,
    ) -> ResultMqttBrokerError {
        if records.is_empty() {
            return Ok(());
        }

        debug!("Sending {} records to Pulsar", records.len());

        let mut success_count = 0;
        let mut failed_records = Vec::new();

        for (idx, record) in records.iter().enumerate() {
            match producer.send_non_blocking(record.clone()).await {
                Ok(_) => {
                    success_count += 1;
                    debug!("Successfully sent record {}/{}", idx + 1, records.len());
                }
                Err(e) => {
                    warn!(
                        "Failed to send record {}/{} (key: '{}', timestamp: {}): {}",
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
                "Sent {}/{} records successfully to Pulsar topic {}",
                success_count,
                records.len(),
                self.config.topic
            );
        }

        if failed_records.len() == records.len() {
            return Err(crate::handler::error::MqttBrokerError::CommonError(
                format!("All {} records failed to send to Pulsar", records.len()),
            ));
        }

        Ok(())
    }

    async fn cleanup_sink(&self, mut producer: Self::SinkResource) -> ResultMqttBrokerError {
        info!("Closing Pulsar producer for topic {}", self.config.topic);

        match producer.send_batch().await {
            Ok(_) => {
                info!("Pulsar producer flushed successfully");
            }
            Err(e) => {
                warn!("Failed to flush Pulsar producer: {}", e);
            }
        }

        info!("Pulsar producer closed successfully");
        Ok(())
    }
}

pub fn start_pulsar_connector(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        let pulsar_config = match serde_json::from_str::<PulsarConnectorConfig>(&connector.config) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to parse PulsarConnectorConfig file with error message :{}, configuration contents: {}", e, connector.config);
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

        let batch_size = pulsar_config.batch_size as u64;
        let bridge = PulsarBridgePlugin::new(pulsar_config);

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
                "Failed to start PulsarBridgePlugin with error message: {:?}",
                e
            );
        }
    });
}
