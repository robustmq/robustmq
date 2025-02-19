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

use super::core::{BridgePlugin, BridgePluginReadConfig};
use crate::{handler::error::MqttBrokerError, storage::message::MessageStorage};
use axum::async_trait;
use log::error;
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_local_file::LocalFileConnectorConfig,
};
use storage_adapter::storage::StorageAdapter;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::{fs::OpenOptions, select, sync::broadcast, time::sleep};

pub struct FileBridgePlugin<S> {
    message_storage: Arc<S>,
    connector_name: String,
    config: LocalFileConnectorConfig,
    stop_send: broadcast::Sender<bool>,
}

impl<S> FileBridgePlugin<S>
where
    S: StorageAdapter + Sync + Send + 'static + Clone,
{
    pub fn new(
        message_storage: Arc<S>,
        connector_name: String,
        config: LocalFileConnectorConfig,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        FileBridgePlugin {
            message_storage,
            connector_name,
            config,
            stop_send,
        }
    }

    pub async fn append(
        &self,
        records: &Vec<Record>,
        writer: &mut BufWriter<File>,
    ) -> Result<(), MqttBrokerError> {
        for record in records {
            let data = serde_json::to_string(record)?;
            writer.write_all(data.as_ref()).await?;
        }
        writer.flush().await?;
        Ok(())
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
        let file = OpenOptions::new()
            .append(true)
            .open(self.config.local_file_path.clone())
            .await?;
        let mut writer = tokio::io::BufWriter::new(file);

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

                            if let Err(e) = self.append(&data,&mut writer).await{
                                error!("Connector {} failed to write data to {}, error message :{}", self.connector_name,self.config.local_file_path, e);
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
