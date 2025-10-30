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
    adapter::record::Record, mqtt::bridge::config_kafka::KafkaConnectorConfig,
    mqtt::bridge::connector::MQTTConnector,
};
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use storage_adapter::storage::ArcStorageAdapter;
use tracing::error;

use crate::common::types::ResultMqttBrokerError;

use super::{
    core::{run_connector_loop, BridgePluginReadConfig, BridgePluginThread, ConnectorSink},
    manager::ConnectorManager,
};

pub struct KafkaBridgePlugin {
    config: KafkaConnectorConfig,
}

impl KafkaBridgePlugin {
    pub fn new(config: KafkaConnectorConfig) -> Self {
        KafkaBridgePlugin { config }
    }
}

#[async_trait]
impl ConnectorSink for KafkaBridgePlugin {
    type SinkResource = FutureProducer;

    async fn validate(&self) -> ResultMqttBrokerError {
        Ok(())
    }

    async fn init_sink(
        &self,
    ) -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError> {
        let producer: FutureProducer = rdkafka::ClientConfig::new()
            .set("bootstrap.servers", self.config.bootstrap_servers.as_str())
            .set("message.timeout.ms", "5000")
            .create()?;
        Ok(producer)
    }

    async fn send_batch(
        &self,
        records: &[Record],
        producer: &mut FutureProducer,
    ) -> ResultMqttBrokerError {
        for record in records {
            let data = serde_json::to_string(record)?;
            producer
                .send(
                    FutureRecord::to(self.config.topic.as_str())
                        .key(self.config.key.as_str())
                        .payload(&data),
                    Duration::from_secs(0),
                )
                .await
                .map_err(|(e, _)| e)?;
        }

        producer.flush(Duration::from_secs(0))?;
        Ok(())
    }

    async fn cleanup_sink(&self, producer: FutureProducer) -> ResultMqttBrokerError {
        producer.flush(Duration::from_secs(5))?;
        Ok(())
    }
}

pub fn start_kafka_connector(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        let kafka_config = match serde_json::from_str::<KafkaConnectorConfig>(&connector.config) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to parse KafkaConnectorConfig with error message: {}, configuration contents: {}", e, connector.config);
                return;
            }
        };

        let bridge = KafkaBridgePlugin::new(kafka_config);

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
            },
            stop_recv,
        )
        .await
        {
            connector_manager.remove_connector_thread(&connector.connector_name);
            error!(
                "Failed to start KafkaBridgePlugin with error message: {:?}",
                e
            );
        }
    });
}
