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
use crate::handler::error::MqttBrokerError;
use crate::handler::tool::ResultMqttBrokerError;
use axum::async_trait;
use chrono::{DateTime, Local, Timelike};
use metadata_struct::mqtt::bridge::config_local_file::RotationStrategy;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::{
    mqtt::bridge::config_local_file::LocalFileConnectorConfig,
    mqtt::bridge::connector::MQTTConnector, storage::adapter_record::AdapterWriteRecord,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use storage_adapter::storage::ArcStorageAdapter;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tracing::{error, info};

pub struct FileWriter {
    writer: BufWriter<File>,
    current_path: PathBuf,
    current_size: u64,
    last_rotation_time: DateTime<Local>,
    config: LocalFileConnectorConfig,
}

impl FileWriter {
    async fn new(config: LocalFileConnectorConfig) -> Result<Self, MqttBrokerError> {
        let path = PathBuf::from(&config.local_file_path);
        let writer = Self::create_file(&path).await?;
        let current_size = Self::get_file_size(&path).await?;

        Ok(FileWriter {
            writer,
            current_path: path,
            current_size,
            last_rotation_time: Local::now(),
            config,
        })
    }

    async fn create_file(path: &Path) -> Result<BufWriter<File>, MqttBrokerError> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;
        Ok(BufWriter::new(file))
    }

    async fn get_file_size(path: &Path) -> Result<u64, MqttBrokerError> {
        match tokio::fs::metadata(path).await {
            Ok(metadata) => Ok(metadata.len()),
            Err(_) => Ok(0),
        }
    }

    fn generate_rotated_filename(&self) -> PathBuf {
        let original_path = Path::new(&self.config.local_file_path);
        let parent = original_path.parent().unwrap_or_else(|| Path::new("."));
        let stem = original_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("file");
        let extension = original_path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("log");

        let now = Local::now();
        let timestamp = match self.config.rotation_strategy {
            RotationStrategy::Hourly => now.format("%Y%m%d_%H").to_string(),
            RotationStrategy::Daily => now.format("%Y%m%d").to_string(),
            RotationStrategy::Size => now.format("%Y%m%d_%H%M%S").to_string(),
            RotationStrategy::None => String::new(),
        };

        parent.join(format!("{}_{}.{}", stem, timestamp, extension))
    }

    async fn should_rotate(&self) -> bool {
        match self.config.rotation_strategy {
            RotationStrategy::None => false,
            RotationStrategy::Size => {
                let max_size_bytes = self.config.max_size_gb * 1024 * 1024 * 1024;
                self.current_size >= max_size_bytes
            }
            RotationStrategy::Hourly => {
                let now = Local::now();
                now.hour() != self.last_rotation_time.hour()
                    || now.date_naive() != self.last_rotation_time.date_naive()
            }
            RotationStrategy::Daily => {
                let now = Local::now();
                now.date_naive() != self.last_rotation_time.date_naive()
            }
        }
    }

    async fn rotate(&mut self) -> Result<(), MqttBrokerError> {
        self.writer.flush().await?;
        self.writer.shutdown().await?;

        let new_path = self.generate_rotated_filename();
        tokio::fs::rename(&self.current_path, &new_path).await?;

        info!(
            "File rotated: {} -> {}",
            self.current_path.display(),
            new_path.display()
        );

        self.writer = Self::create_file(&self.current_path).await?;
        self.current_size = 0;
        self.last_rotation_time = Local::now();

        Ok(())
    }

    async fn write(&mut self, data: &[u8]) -> Result<(), MqttBrokerError> {
        if self.should_rotate().await {
            self.rotate().await?;
        }

        self.writer.write_all(data).await?;
        self.current_size += data.len() as u64;
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), MqttBrokerError> {
        self.writer.flush().await?;
        Ok(())
    }

    async fn shutdown(mut self) -> Result<(), MqttBrokerError> {
        self.writer.flush().await?;
        self.writer.shutdown().await?;
        Ok(())
    }
}

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
    type SinkResource = FileWriter;

    async fn validate(&self) -> ResultMqttBrokerError {
        let file_path = Path::new(&self.config.local_file_path);

        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
                info!("Created directory: {}", parent.display());
            }
        }

        Ok(())
    }

    async fn init_sink(
        &self,
    ) -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError> {
        FileWriter::new(self.config.clone()).await
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        writer: &mut FileWriter,
    ) -> ResultMqttBrokerError {
        for record in records {
            let msg = MqttMessage::decode(&record.data)?;
            let data = serde_json::to_string(&msg)?;
            writer.write(data.as_ref()).await?;
            writer.write(b"\n").await?;
        }
        writer.flush().await?;
        Ok(())
    }

    async fn cleanup_sink(&self, writer: FileWriter) -> ResultMqttBrokerError {
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
        let local_file_config = match &connector.config {
            metadata_struct::mqtt::bridge::ConnectorConfig::LocalFile(config) => config.clone(),
            _ => {
                error!("Invalid connector config type, expected LocalFile config");
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
                strategy: connector.failure_strategy,
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
    use common_base::tools::now_second;
    use metadata_struct::{
        mqtt::bridge::{
            config_local_file::LocalFileConnectorConfig, connector::FailureHandlingStrategy,
        },
        storage::{
            adapter_offset::ShardInfo,
            adapter_record::{AdapterWriteRecord, Header},
        },
    };
    use std::{fs, path::PathBuf, sync::Arc, time::Duration};
    use storage_adapter::storage::build_memory_storage_driver;
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
        let storage_adapter = build_memory_storage_driver();

        let shard_name = "test_topic".to_string();

        // prepare some data for testing
        storage_adapter
            .create_shard(&ShardInfo {
                shard_name: shard_name.clone(),
                ..Default::default()
            })
            .await
            .unwrap();

        let mut test_data = vec![];

        for i in 0..1000 {
            let record = AdapterWriteRecord {
                pkid: i,
                header: Some(vec![Header {
                    name: "test_name".to_string(),
                    value: "test_value".to_string(),
                }]),
                key: Some(format!("test_key_{i}")),
                data: format!("test_data_{i}").as_bytes().to_vec().into(),
                tags: Some(vec![]),
                timestamp: now_second(),
            };

            test_data.push(record);
        }

        storage_adapter
            .batch_write(&shard_name, &test_data)
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
            rotation_strategy:
                metadata_struct::mqtt::bridge::config_local_file::RotationStrategy::None,
            max_size_gb: 1,
        };

        // create such file
        fs::create_dir_all(dir_path).unwrap();
        File::create(config.local_file_path.clone()).await.unwrap();

        let (stop_send, _) = broadcast::channel(1);

        let file_bridge_plugin = FileBridgePlugin::new(config.clone());

        let read_config = BridgePluginReadConfig {
            topic_name: shard_name.clone(),
            record_num: 100,
            strategy: FailureHandlingStrategy::Discard,
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
