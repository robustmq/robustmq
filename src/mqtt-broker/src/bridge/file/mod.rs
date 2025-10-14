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
use super::manager::ConnectorManager;
use crate::common::types::ResultMqttBrokerError;
use crate::storage::message::MessageStorage;
use axum::async_trait;
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_local_file::LocalFileConnectorConfig,
};
use storage_adapter::storage::ArcStorageAdapter;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::{fs::OpenOptions, select, sync::broadcast, time::sleep};
use tracing::error;

pub struct FileBridgePlugin {
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector_name: String,
    config: LocalFileConnectorConfig,
    stop_send: broadcast::Sender<bool>,
}

impl FileBridgePlugin {
    pub fn new(
        connector_manager: Arc<ConnectorManager>,
        message_storage: ArcStorageAdapter,
        connector_name: String,
        config: LocalFileConnectorConfig,
        stop_send: broadcast::Sender<bool>,
    ) -> Self {
        FileBridgePlugin {
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
        writer: &mut BufWriter<File>,
    ) -> ResultMqttBrokerError {
        for record in records {
            let data = serde_json::to_string(record)?;
            writer.write_all(data.as_ref()).await?;
        }
        writer.flush().await?;
        Ok(())
    }
}

#[async_trait]
impl BridgePlugin for FileBridgePlugin {
    async fn exec(&self, config: BridgePluginReadConfig) -> ResultMqttBrokerError {
        let message_storage = MessageStorage::new(self.message_storage.clone());
        let group_name = self.connector_name.clone();
        let mut recv = self.stop_send.subscribe();
        let file = OpenOptions::new()
            .append(true)
            .open(self.config.local_file_path.clone())
            .await?;
        let mut writer = tokio::io::BufWriter::new(file);

        loop {
            let offset = message_storage.get_group_offset(&group_name).await?;

            select! {
                val = recv.recv() =>{
                    if let Ok(flag) = val {
                        if flag {
                            break;
                        }
                    }
                },

                val = message_storage.read_topic_message(&config.topic_name, offset, config.record_num) => {
                    match val {
                        Ok(data) => {
                            self.connector_manager.report_heartbeat(&self.connector_name);
                            if data.is_empty() {
                                sleep(Duration::from_millis(100)).await;
                                continue;
                            }

                            if let Err(e) = self.append(&data,&mut writer).await{
                                error!("Connector {} failed to write data to {}, error message :{}", self.connector_name,self.config.local_file_path, e);
                                sleep(Duration::from_millis(100)).await;
                            }

                            // commit offset
                            message_storage.commit_group_offset(&group_name, &config.topic_name, offset + data.len() as u64).await?;
                        },
                        Err(e) => {
                            error!("Connector {} failed to read Topic {} data with error message :{}", self.connector_name,config.topic_name,e);
                            sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf, sync::Arc, time::Duration};

    use common_base::{
        tools::{now_second, unique_id},
        utils::crc::calc_crc32,
    };
    use common_config::{broker::init_broker_conf_by_config, config::BrokerConfig};
    use metadata_struct::{
        adapter::record::{Header, Record},
        mqtt::bridge::config_local_file::LocalFileConnectorConfig,
    };
    use storage_adapter::storage::{build_memory_storage_driver, ShardInfo};
    use tokio::{fs::File, io::AsyncReadExt, sync::broadcast, time::sleep};

    use crate::bridge::{
        core::{BridgePlugin, BridgePluginReadConfig},
        file::FileBridgePlugin,
        manager::ConnectorManager,
    };
    use tempfile::tempdir;

    #[ignore]
    #[tokio::test]
    async fn file_bridge_plugin_test() {
        // init a dummy mqtt broker config
        let namespace = unique_id();

        let mqtt_config = BrokerConfig {
            cluster_name: namespace.clone(),
            ..Default::default()
        };

        init_broker_conf_by_config(mqtt_config);

        let storage_adapter = build_memory_storage_driver();

        let shard_name = "test_topic".to_string();

        // prepare some data for testing
        storage_adapter
            .create_shard(ShardInfo {
                namespace: namespace.clone(),
                shard_name: shard_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let mut test_data = vec![];

        for i in 0..1000 {
            let record = Record {
                offset: Some(i),
                header: vec![Header {
                    name: "test_name".to_string(),
                    value: "test_value".to_string(),
                }],
                key: format!("test_key_{i}"),
                data: format!("test_data_{i}").as_bytes().to_vec(),
                tags: vec![],
                timestamp: now_second(),
                crc_num: calc_crc32(format!("test_data_{i}").as_bytes()),
            };

            test_data.push(record);
        }

        storage_adapter
            .batch_write(namespace.clone(), shard_name.clone(), test_data.clone())
            .await
            .unwrap();

        let connector_name = "test_file_connector".to_string();

        let connector_manager = Arc::new(ConnectorManager::new());

        let dir_path = tempdir().unwrap().path().to_str().unwrap().to_string();

        let config = LocalFileConnectorConfig {
            local_file_path: PathBuf::from(dir_path.clone())
                .join("test.txt")
                .to_str()
                .unwrap()
                .to_string(),
        };

        // create such file
        fs::create_dir_all(dir_path).unwrap();
        File::create(config.local_file_path.clone()).await.unwrap();

        let (stop_send, _) = broadcast::channel(1);

        let file_bridge_plugin = FileBridgePlugin::new(
            connector_manager.clone(),
            storage_adapter.clone(),
            connector_name.clone(),
            config.clone(),
            stop_send.clone(),
        );

        let read_config = BridgePluginReadConfig {
            topic_name: shard_name.clone(),
            record_num: 100,
        };

        let record_config_clone = read_config.clone();
        let handle = tokio::spawn(async move {
            file_bridge_plugin.exec(record_config_clone).await.unwrap();
        });

        // wait for a while
        sleep(Duration::from_secs(2)).await;

        stop_send.send(true).unwrap();

        handle.await.unwrap();

        // read the file and check the data
        let mut file = File::open(config.local_file_path.clone()).await.unwrap();

        let mut res: String = String::new();
        file.read_to_string(&mut res).await.unwrap();

        let expected = test_data.iter().fold(String::new(), |acc, record| {
            let data = serde_json::to_string(record).unwrap();
            acc + &data
        });

        assert_eq!(res, expected);
    }
}
