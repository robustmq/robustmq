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
        use tracing::info;

        let mut client_config = rdkafka::ClientConfig::new();

        client_config
            .set("bootstrap.servers", &self.config.bootstrap_servers)
            .set(
                "message.timeout.ms",
                self.config.message_timeout_ms.to_string(),
            )
            .set("compression.type", &self.config.compression_type)
            .set("batch.size", self.config.batch_size.to_string())
            .set("linger.ms", self.config.linger_ms.to_string())
            .set("acks", &self.config.acks)
            .set("retries", self.config.retries.to_string())
            .set("queue.buffering.max.messages", "100000")
            .set("queue.buffering.max.kbytes", "1048576");

        info!(
            "Kafka producer initialized: servers={}, topic={}, compression={}, batch_size={}, acks={}",
            self.config.bootstrap_servers,
            self.config.topic,
            self.config.compression_type,
            self.config.batch_size,
            self.config.acks
        );

        let producer: FutureProducer = client_config.create()?;
        Ok(producer)
    }

    async fn send_batch(
        &self,
        records: &[Record],
        producer: &mut FutureProducer,
    ) -> ResultMqttBrokerError {
        use futures::future::join_all;

        let mut serialized_data = Vec::with_capacity(records.len());
        let mut keys = Vec::with_capacity(records.len());

        for record in records {
            let data = serde_json::to_string(record)?;
            serialized_data.push(data);

            let key = if self.config.key.is_empty() {
                if record.key.is_empty() {
                    String::new()
                } else {
                    record.key.clone()
                }
            } else {
                self.config.key.clone()
            };
            keys.push(key);
        }

        let mut send_futures = Vec::with_capacity(serialized_data.len());

        for (data, key) in serialized_data.iter().zip(keys.iter()) {
            let future = producer.send(
                FutureRecord::to(self.config.topic.as_str())
                    .key(key)
                    .payload(data),
                Duration::from_secs(0),
            );
            send_futures.push(future);
        }

        let results = join_all(send_futures).await;

        if results.iter().all(|r| r.is_err()) {
            return Err(crate::handler::error::MqttBrokerError::CommonError(
                "All records failed to send to Kafka".to_string(),
            ));
        }

        Ok(())
    }

    async fn cleanup_sink(&self, producer: FutureProducer) -> ResultMqttBrokerError {
        use tracing::info;

        info!(
            "Flushing Kafka producer with timeout of {}s",
            self.config.cleanup_timeout_secs
        );
        producer.flush(Duration::from_secs(self.config.cleanup_timeout_secs))?;
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
                strategy: connector.failure_strategy,
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
