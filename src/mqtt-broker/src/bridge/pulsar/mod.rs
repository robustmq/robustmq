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
        manager::ConnectorManager,
    },
    handler::tool::ResultMqttBrokerError,
};
use axum::async_trait;
use metadata_struct::{
    mqtt::bridge::config_pulsar::PulsarConnectorConfig, mqtt::bridge::connector::MQTTConnector,
    storage::adapter_record::AdapterWriteRecord,
};
use storage_adapter::storage::ArcStorageAdapter;
use tracing::error;
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
        Ok(())
    }

    async fn init_sink(
        &self,
    ) -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError> {
        let producer = pulsar_producer::Producer::new(&self.config)
            .build_producer()
            .await?;
        Ok(producer)
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        producer: &mut pulsar::producer::Producer<pulsar::TokioExecutor>,
    ) -> ResultMqttBrokerError {
        for record in records {
            producer.send_non_blocking(record.clone()).await?;
        }
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
        let pulsar_config = match &connector.config {
            metadata_struct::mqtt::bridge::ConnectorConfig::Pulsar(config) => config.clone(),
            _ => {
                error!("Invalid connector config type, expected Pulsar config");
                return;
            }
        };

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
                record_num: 100,
                strategy: connector.failure_strategy,
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
