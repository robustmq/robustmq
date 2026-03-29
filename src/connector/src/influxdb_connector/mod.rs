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

use async_trait::async_trait;
use common_base::error::common::CommonError;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    connector::config_influxdb::{InfluxDBConnectorConfig, InfluxDBVersion},
    connector::MQTTConnector,
    storage::storage_record::StorageRecord,
};
use reqwest::Client;
use rule_engine::apply_rule_engine;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error};

use super::{
    core::{BridgePluginReadConfig, BridgePluginThread},
    failure::FailureRecordInfo,
    loops::run_connector_loop,
    manager::ConnectorManager,
    traits::ConnectorSink,
};

pub struct InfluxDBBridgePlugin {
    connector: MQTTConnector,
    config: InfluxDBConnectorConfig,
}

impl InfluxDBBridgePlugin {
    #[allow(clippy::result_large_err)]
    pub fn new(connector: MQTTConnector) -> Result<Self, CommonError> {
        let config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::InfluxDB(config) => config.clone(),
            _ => {
                return Err(CommonError::CommonError(
                    "invalid connector type for influxdb plugin".to_string(),
                ));
            }
        };
        Ok(InfluxDBBridgePlugin { connector, config })
    }

    #[allow(clippy::result_large_err)]
    fn build_client(&self) -> Result<Client, CommonError> {
        Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .build()
            .map_err(|e| CommonError::CommonError(format!("Failed to build HTTP client: {}", e)))
    }

    /// Convert a StorageRecord to InfluxDB Line Protocol format:
    /// `measurement,tag1=val1 field1="strval",field2=42i timestamp`
    fn record_to_line_protocol(
        &self,
        record: &StorageRecord,
        processed_data: &bytes::Bytes,
    ) -> String {
        let measurement = &self.config.measurement;
        let payload_str = String::from_utf8_lossy(processed_data);

        let mut tags = String::new();
        if let Some(key) = &record.metadata.key {
            if !key.is_empty() {
                tags.push_str(&format!(",key={}", escape_tag_value(key)));
            }
        }

        let escaped_payload = payload_str.replace('\\', "\\\\").replace('"', "\\\"");
        let fields = format!("payload=\"{}\"", escaped_payload);

        let timestamp = record.metadata.create_t;

        format!("{}{} {} {}", measurement, tags, fields, timestamp)
    }
}

fn escape_tag_value(s: &str) -> String {
    s.replace(' ', "\\ ")
        .replace(',', "\\,")
        .replace('=', "\\=")
}

#[async_trait]
impl ConnectorSink for InfluxDBBridgePlugin {
    type SinkResource = Client;

    async fn validate(&self) -> Result<(), CommonError> {
        self.config.validate()
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
        let client = self.build_client()?;

        debug!(
            "InfluxDB connector initialized: server={}, version={:?}, measurement={}",
            self.config.server, self.config.version, self.config.measurement
        );

        Ok(client)
    }

    async fn send_batch(
        &self,
        records: &[StorageRecord],
        client: &mut Client,
    ) -> Result<Vec<FailureRecordInfo>, CommonError> {
        if records.is_empty() {
            return Ok(vec![]);
        }

        let mut lines: Vec<String> = Vec::with_capacity(records.len());
        let mut fail_messages = Vec::new();
        for record in records {
            let processed_data =
                match apply_rule_engine(&self.connector.etl_rule, &record.data).await {
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
            lines.push(self.record_to_line_protocol(record, &processed_data));
        }
        if lines.is_empty() {
            return Ok(fail_messages);
        }
        let body = lines.join("\n");

        let url = self.config.write_url();

        let mut request = client
            .post(&url)
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(body);

        if matches!(self.config.version, InfluxDBVersion::V2) && !self.config.token.is_empty() {
            request = request.header("Authorization", format!("Token {}", self.config.token));
        }

        let response = request.send().await.map_err(|e| {
            CommonError::CommonError(format!("InfluxDB HTTP request failed: {}", e))
        })?;

        if response.status().is_success() || response.status().as_u16() == 204 {
            Ok(fail_messages)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(CommonError::CommonError(format!(
                "InfluxDB returned HTTP {}: {}",
                status, error_text
            )))
        }
    }
}

pub fn start_influxdb_connector(
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
        let bridge = match InfluxDBBridgePlugin::new(connector.clone()) {
            Ok(bridge) => bridge,
            Err(e) => {
                error!(
                    "Invalid connector config type for InfluxDB connector, connector_name='{}', connector_type='{}', error={}",
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
                "Failed to start InfluxDBBridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}
