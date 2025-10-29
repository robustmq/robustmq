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

use super::core::{run_connector_loop, BridgePluginReadConfig, BridgePluginThread, ConnectorSink};
use super::manager::ConnectorManager;
use crate::common::types::ResultMqttBrokerError;
use axum::async_trait;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_local_file::LocalFileConnectorConfig,
    mqtt::bridge::connector::MQTTConnector,
};
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tracing::error;

pub struct FileBridgePlugin {
    config: LocalFileConnectorConfig,
}

impl FileBridgePlugin {
    pub fn new(config: LocalFileConnectorConfig) -> Self {
        FileBridgePlugin { config }
    }
}

#[async_trait]
impl ConnectorSink for FileBridgePlugin {
    type SinkResource = BufWriter<File>;

    async fn init_sink(
        &self,
    ) -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError> {
        let file_path = std::path::Path::new(&self.config.local_file_path);
        if let Some(parent) = file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.config.local_file_path.clone())
            .await?;
        Ok(BufWriter::new(file))
    }

    async fn send_batch(
        &self,
        records: &[Record],
        writer: &mut BufWriter<File>,
    ) -> ResultMqttBrokerError {
        for record in records {
            let msg = serde_json::from_slice::<MqttMessage>(&record.data)?;
            let data = serde_json::to_string(&msg)?;
            writer.write_all(data.as_ref()).await?;
            writer.write_all(b"\n").await?;
        }
        writer.flush().await?;
        Ok(())
    }

    async fn cleanup_sink(&self, mut writer: BufWriter<File>) -> ResultMqttBrokerError {
        writer.flush().await?;
        writer.shutdown().await?;
        Ok(())
    }
}

pub fn start_local_file_connector(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        let local_file_config = match serde_json::from_str::<LocalFileConnectorConfig>(
            &connector.config,
        ) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to parse LocalFileConnectorConfig file with error message :{}, configuration contents: {}", e, connector.config);
                return;
            }
        };

        let bridge = FileBridgePlugin::new(local_file_config);

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
                "Failed to start FileBridgePlugin with error message: {:?}",
                e
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use common_base::{
        tools::{now_second, unique_id},
        utils::crc::calc_crc32,
    };
    use common_config::{broker::init_broker_conf_by_config, config::BrokerConfig};
    use metadata_struct::{
        adapter::record::{Header, Record},
        mqtt::bridge::config_local_file::LocalFileConnectorConfig,
    };
    use std::{fs, path::PathBuf, sync::Arc, time::Duration};
    use storage_adapter::storage::{build_memory_storage_driver, ShardInfo};
    use tokio::{fs::File, io::AsyncReadExt, sync::broadcast, time::sleep};

    use crate::bridge::{
        core::{run_connector_loop, BridgePluginReadConfig},
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

        let file_bridge_plugin = FileBridgePlugin::new(config.clone());

        let read_config = BridgePluginReadConfig {
            topic_name: shard_name.clone(),
            record_num: 100,
        };

        let record_config_clone = read_config.clone();
        let connector_manager_clone = connector_manager.clone();
        let storage_adapter_clone = storage_adapter.clone();
        let connector_name_clone = connector_name.clone();
        let stop_recv = stop_send.subscribe();

        let handle = tokio::spawn(async move {
            run_connector_loop(
                &file_bridge_plugin,
                &connector_manager_clone,
                storage_adapter_clone,
                connector_name_clone,
                record_config_clone,
                stop_recv,
            )
            .await
            .unwrap();
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
