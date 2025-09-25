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

use crate::storage::message::MessageStorage;
use crate::{
    bridge::{
        core::{BridgePlugin, BridgePluginReadConfig},
        manager::ConnectorManager,
    },
    common::types::ResultMqttBrokerError,
};
use axum::async_trait;
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_pulsar::PulsarConnectorConfig,
};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};
use tracing::{error, info};
mod pulsar_producer;

pub struct PulsarBridgePlugin {
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector_name: String,
    config: PulsarConnectorConfig,
    stop_send: broadcast::Sender<bool>,
}

impl PulsarBridgePlugin {
    pub fn new(
        connector_manager: Arc<ConnectorManager>,
        message_storage: ArcStorageAdapter,
        connector_name: String,
        config: PulsarConnectorConfig,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        PulsarBridgePlugin {
            connector_manager,
            message_storage,
            connector_name,
            config,
            stop_send,
        }
    }

    pub async fn append(&self, records: &Vec<Record>) -> ResultMqttBrokerError {
        let mut producer = pulsar_producer::Producer::new(&self.config)
            .build_producer()
            .await?;
        for record in records {
            producer.send_non_blocking(record.clone()).await?;
        }

        Ok(())
    }
}

#[async_trait]
impl BridgePlugin for PulsarBridgePlugin {
    async fn exec(&self, config: BridgePluginReadConfig) -> ResultMqttBrokerError {
        let message_storage = MessageStorage::new(self.message_storage.clone());
        let group_name = self.connector_name.clone();
        let offset = message_storage.get_group_offset(&group_name).await?;
        let mut recv = self.stop_send.subscribe();

        loop {
            select! {
                val = recv.recv() => {
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

                            if let Err(e) = self.append(&data).await{
                                error!("Connector {} failed to write data to Pulsar topic {}, error message: {}", self.connector_name, self.config.topic, e);
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
