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

use crate::failure::FailureRecordInfo;

use super::core::{BridgePluginReadConfig, BridgePluginThread};
use super::loops::run_connector_loop;
use super::manager::ConnectorManager;
use super::traits::ConnectorSink;
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Local, Timelike};
use common_base::error::common::CommonError;
use grpc_clients::pool::ClientPool;
use metadata_struct::connector::config_local_file::RotationStrategy;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::{
    connector::config_local_file::LocalFileConnectorConfig, connector::MQTTConnector,
    storage::adapter_record::AdapterWriteRecord,
};
use rule_engine::apply_rule_engine;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::mpsc::Receiver;
use tracing::{error, info};

pub struct FileWriter {
    writer: BufWriter<File>,
    current_path: PathBuf,
    current_size: u64,
    last_rotation_time: DateTime<Local>,
    config: LocalFileConnectorConfig,
}

impl FileWriter {
    async fn new(config: LocalFileConnectorConfig) -> Result<Self, CommonError> {
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

    async fn create_file(path: &Path) -> Result<BufWriter<File>, CommonError> {
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

    async fn get_file_size(path: &Path) -> Result<u64, CommonError> {
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

    async fn rotate(&mut self) -> Result<(), CommonError> {
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

    async fn write(&mut self, data: &[u8]) -> Result<(), CommonError> {
        if self.should_rotate().await {
            self.rotate().await?;
        }

        self.writer.write_all(data).await?;
        self.current_size += data.len() as u64;
        Ok(())
    }

    async fn flush(&mut self) -> Result<(), CommonError> {
        self.writer.flush().await?;
        Ok(())
    }

    async fn shutdown(mut self) -> Result<(), CommonError> {
        self.writer.flush().await?;
        self.writer.shutdown().await?;
        Ok(())
    }
}

pub struct FileBridgePlugin {
    connector: MQTTConnector,
    config: LocalFileConnectorConfig,
}

impl FileBridgePlugin {
    #[allow(clippy::result_large_err)]
    pub fn new(connector: MQTTConnector) -> Result<Self, CommonError> {
        let config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::LocalFile(config) => config.clone(),
            _ => {
                return Err(CommonError::CommonError(
                    "invalid connector type for file plugin".to_string(),
                ));
            }
        };
        Ok(FileBridgePlugin { connector, config })
    }

    async fn process_data(&self, data: &Bytes) -> Result<String, CommonError> {
        let msg = MqttMessage::decode(data)?;
        let processed_data = apply_rule_engine(&self.connector.etl_rule, &msg.payload).await?;
        let data = serde_json::to_string(&processed_data)?;
        Ok(data)
    }
}

#[async_trait]
impl ConnectorSink for FileBridgePlugin {
    type SinkResource = FileWriter;

    async fn validate(&self) -> Result<(), CommonError> {
        let file_path = Path::new(&self.config.local_file_path);

        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
                info!("Created directory: {}", parent.display());
            }
        }

        Ok(())
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
        FileWriter::new(self.config.clone()).await
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        writer: &mut FileWriter,
    ) -> Result<Vec<FailureRecordInfo>, CommonError> {
        let mut fail_messages = Vec::new();
        for record in records {
            let data = match self.process_data(&record.data).await {
                Ok(data) => data,
                Err(e) => {
                    fail_messages.push(FailureRecordInfo {
                        tenant: self.connector.tenant.clone(),
                        connector_name: self.connector.connector_name.clone(),
                        connector_type: self.connector.connector_type.to_string(),
                        source_topic: self.connector.topic_name.clone(),
                        error_message: e.to_string(),
                        records: vec![record.clone()],
                    });
                    continue;
                }
            };
            writer.write(data.as_ref()).await?;
            writer.write(b"\n").await?;
        }
        writer.flush().await?;
        Ok(fail_messages)
    }

    async fn cleanup_sink(&self, writer: FileWriter) -> Result<(), CommonError> {
        writer.shutdown().await?;
        Ok(())
    }
}

pub fn start_local_file_connector(
    client_pool: Arc<ClientPool>,
    connector_manager: Arc<ConnectorManager>,
    storage_driver_manager: Arc<StorageDriverManager>,
    connector: MQTTConnector,
    thread: BridgePluginThread,
    stop_recv: Receiver<bool>,
) {
    tokio::spawn(Box::pin(async move {
        let connector_name = connector.connector_name.clone();
        let connector_type = connector.connector_type.to_string();
        let bridge = match FileBridgePlugin::new(connector.clone()) {
            Ok(bridge) => bridge,
            Err(e) => {
                error!(
                    "Invalid connector config type for LocalFile connector, connector_name='{}', connector_type='{}', error={}",
                    connector_name, connector_type, e
                );
                return;
            }
        };

        connector_manager.add_connector_thread(
            &connector.tenant,
            &connector.connector_name,
            thread,
        );

        if let Err(e) = run_connector_loop(
            &bridge,
            &client_pool,
            &connector_manager,
            &storage_driver_manager,
            connector.connector_name.clone(),
            BridgePluginReadConfig {
                tenant: connector.tenant,
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
                "Failed to start FileBridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;
    use common_base::tools::now_second;
    use common_base::uuid::unique_id;
    use metadata_struct::{
        connector::{
            config_local_file::LocalFileConnectorConfig, rule::ETLRule, ConnectorType,
            FailureHandlingStrategy, MQTTConnector,
        },
        mqtt::message::MqttMessage,
        storage::adapter_record::{AdapterWriteRecord, AdapterWriteRecordHeader},
        tenant::DEFAULT_TENANT,
    };
    use protocol::mqtt::common::QoS;
    use std::{fs, path::PathBuf};
    use tokio::{fs::File, io::AsyncReadExt};

    use crate::{file::FileBridgePlugin, traits::ConnectorSink};
    use tempfile::tempdir;

    #[tokio::test]
    async fn file_bridge_plugin_test() {
        let mut test_data = vec![];

        for i in 0..1000 {
            let record = AdapterWriteRecord {
                pkid: i,
                header: Some(vec![AdapterWriteRecordHeader {
                    name: "test_name".to_string(),
                    value: "test_value".to_string(),
                }]),
                key: Some(format!("test_key_{i}")),
                data: MqttMessage {
                    client_id: "test-client".to_string(),
                    dup: false,
                    qos: QoS::AtMostOnce,
                    pkid: 0,
                    retain: false,
                    topic: Bytes::from("test/topic"),
                    payload: Bytes::from(format!("test_data_{i}")),
                    format_indicator: None,
                    expiry_interval: 0,
                    response_topic: None,
                    correlation_data: None,
                    user_properties: None,
                    subscription_identifiers: None,
                    content_type: None,
                    create_time: now_second(),
                }
                .encode()
                .unwrap()
                .into(),
                tags: Some(vec![]),
                timestamp: now_second(),
            };

            test_data.push(record);
        }

        let dir_path = tempdir().unwrap().path().to_str().unwrap().to_string();

        let config = LocalFileConnectorConfig {
            local_file_path: PathBuf::from(dir_path.clone())
                .join("test.txt")
                .to_str()
                .unwrap()
                .to_string(),
            rotation_strategy:
                metadata_struct::connector::config_local_file::RotationStrategy::None,
            max_size_gb: 1,
        };

        // create such file
        fs::create_dir_all(dir_path).unwrap();
        File::create(config.local_file_path.clone()).await.unwrap();

        let file_bridge_plugin = FileBridgePlugin::new(MQTTConnector {
            connector_name: "test-file".to_string(),
            connector_type: ConnectorType::LocalFile(config.clone()),
            failure_strategy: FailureHandlingStrategy::Discard,
            tenant: DEFAULT_TENANT.to_string(),
            topic_name: unique_id(),
            status: Default::default(),
            etl_rule: ETLRule::default(),
            broker_id: None,
            create_time: now_second(),
            update_time: now_second(),
        })
        .unwrap();

        let mut writer = file_bridge_plugin.init_sink().await.unwrap();
        let fail_messages = file_bridge_plugin
            .send_batch(&test_data, &mut writer)
            .await
            .unwrap();
        assert!(fail_messages.is_empty());
        file_bridge_plugin.cleanup_sink(writer).await.unwrap();

        // read the file and check the data
        let mut file = File::open(config.local_file_path.clone()).await.unwrap();

        let mut res: String = String::new();
        file.read_to_string(&mut res).await.unwrap();

        let expected = test_data.iter().fold(String::new(), |mut acc, record| {
            let msg = MqttMessage::decode(&record.data).unwrap();
            let data = serde_json::to_string(&msg.payload).unwrap();
            acc.push_str(&data);
            acc.push('\n');
            acc
        });

        assert_eq!(res, expected);
    }
}
