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

use axum::async_trait;
use log::error;
use metadata_struct::mqtt::bridge::config_kafka::KafkaConnectorConfig;
use std::{sync::Arc, time::Duration};
use storage_adapter::storage::StorageAdapter;
use tokio::{select, sync::broadcast, time::sleep};

use crate::{handler::error::MqttBrokerError, storage::message::MessageStorage};

use super::core::{BridgePlugin, BridgePluginReadConfig};

pub struct FileBridgePlugin<S> {
    message_storage: Arc<S>,
    connector_name: String,
    config: KafkaConnectorConfig,
    stop_send: broadcast::Sender<bool>,
}

impl<S> FileBridgePlugin<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        message_storage: Arc<S>,
        connector_name: String,
        config: KafkaConnectorConfig,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        FileBridgePlugin {
            message_storage,
            connector_name,
            config,
            stop_send,
        }
    }
}

#[async_trait]
impl<S> BridgePlugin for FileBridgePlugin<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    async fn exec(&self, config: BridgePluginReadConfig) -> Result<(), MqttBrokerError> {
        let message_storage = MessageStorage::new(self.message_storage.clone());
        let group_name = self.connector_name.clone();
        let offset = message_storage.get_group_offset(&group_name).await?;
        let mut recv = self.stop_send.subscribe();

        loop {
            select! {
                val = recv.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            break;
                        }
                    }
                },

                val = message_storage.read_topic_message(&config.topic_id, offset, config.record_num) => {
                    match val {
                        Ok(data) => {

                            if data.is_empty() {
                                sleep(Duration::from_millis(100)).await;
                                continue;
                            }

                            println!("{}", self.config.bootstrap_servers);
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
