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
use log::{error, info};
use metadata_struct::{adapter::record::Record, mqtt::bridge::config_kafka::KafkaConnectorConfig};
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use storage_adapter::storage::StorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};

use crate::{handler::error::MqttBrokerError, storage::message::MessageStorage};

use super::{
    core::{BridgePlugin, BridgePluginReadConfig},
    manager::ConnectorManager,
};

pub struct KafkaBridgePlugin<S> {
    connector_manager: Arc<ConnectorManager>,
    message_storage: Arc<S>,
    connector_name: String,
    config: KafkaConnectorConfig,
    stop_send: broadcast::Sender<bool>,
}

impl<S> KafkaBridgePlugin<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        connector_manager: Arc<ConnectorManager>,
        message_storage: Arc<S>,
        connector_name: String,
        config: KafkaConnectorConfig,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        KafkaBridgePlugin {
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
        producer: FutureProducer,
    ) -> Result<(), MqttBrokerError> {
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
}

#[async_trait]
impl<S> BridgePlugin for KafkaBridgePlugin<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    async fn exec(&self, config: BridgePluginReadConfig) -> Result<(), MqttBrokerError> {
        let message_storage = MessageStorage::new(self.message_storage.clone());
        let group_name = self.connector_name.clone();
        let offset = message_storage.get_group_offset(&group_name).await?;
        let mut recv = self.stop_send.subscribe();
        let producer: FutureProducer = rdkafka::ClientConfig::new()
            .set("bootstrap.servers", self.config.bootstrap_servers.as_str())
            .set("message.timeout.ms", "5000")
            .create()?;

        loop {
            select! {
                val = recv.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            info!("{}","Connector thread exited successfully");
                            break;
                        }
                    }
                }

                val = message_storage.read_topic_message(&config.topic_id, offset, config.record_num) => {
                    match val {
                        Ok(data) => {
                            self.connector_manager.report_heartbeat(&self.connector_name);
                            if data.is_empty() {
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                continue;
                            }

                            if let Err(e) = self.append(&data, producer.clone()).await{
                                error!("Connector {} failed to write data to kafka topic {}, error message: {}", self.connector_name, self.config.topic, e);
                                sleep(Duration::from_millis(100)).await;
                            }
                        },
                        Err(e) => {
                            error!("Connector {} failed to read Topic {} data with error message :{}", self.connector_name,config.topic_id,e);
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
