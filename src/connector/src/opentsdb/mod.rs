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
    connector::config_opentsdb::OpenTSDBConnectorConfig, connector::MQTTConnector,
    storage::storage_record::StorageRecord,
};
use reqwest::Client;
use rule_engine::apply_rule_engine;
use serde_json::{json, Map, Value};
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc::Receiver;
use tracing::error;

use super::{
    core::{BridgePluginReadConfig, BridgePluginThread},
    failure::FailureRecordInfo,
    loops::run_connector_loop,
    manager::ConnectorManager,
    traits::ConnectorSink,
};

pub struct OpenTSDBBridgePlugin {
    connector: MQTTConnector,
    config: OpenTSDBConnectorConfig,
}

impl OpenTSDBBridgePlugin {
    #[allow(clippy::result_large_err)]
    pub fn new(connector: MQTTConnector) -> Result<Self, CommonError> {
        let config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::OpenTSDB(config) => config.clone(),
            _ => {
                return Err(CommonError::CommonError(
                    "invalid connector type for opentsdb plugin".to_string(),
                ));
            }
        };
        Ok(OpenTSDBBridgePlugin { connector, config })
    }

    #[allow(clippy::result_large_err)]
    fn build_client(&self) -> Result<Client, CommonError> {
        Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .build()
            .map_err(|e| CommonError::CommonError(format!("Failed to build HTTP client: {}", e)))
    }

    #[allow(clippy::result_large_err)]
    async fn record_to_data_point(
        &self,
        record: &StorageRecord,
    ) -> Result<Value, CommonError> {
        let processed_data = apply_rule_engine(&self.connector.etl_rule, &record.data).await?;
        let payload_str = String::from_utf8_lossy(&processed_data);
        let payload: Value = serde_json::from_str(&payload_str).map_err(|e| {
            CommonError::CommonError(format!("Failed to parse payload as JSON: {}", e))
        })?;

        let metric = payload
            .get(&self.config.metric_field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                CommonError::CommonError(format!(
                    "Missing or invalid metric field '{}'",
                    self.config.metric_field
                ))
            })?;

        let value = payload.get(&self.config.value_field).ok_or_else(|| {
            CommonError::CommonError(format!("Missing value field '{}'", self.config.value_field))
        })?;

        let tags = if !self.config.tags_fields.is_empty() {
            let mut tag_map = Map::new();
            for field in &self.config.tags_fields {
                if let Some(v) = payload.get(field) {
                    let tag_val = match v {
                        Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    tag_map.insert(field.clone(), Value::String(tag_val));
                }
            }
            Value::Object(tag_map)
        } else if let Some(tags_obj) = payload.get("tags") {
            tags_obj.clone()
        } else {
            json!({})
        };

        let timestamp = payload
            .get("timestamp")
            .and_then(|v| v.as_u64())
            .unwrap_or(record.metadata.create_t);

        Ok(json!({
            "metric": metric,
            "timestamp": timestamp,
            "value": value,
            "tags": tags
        }))
    }
}

#[async_trait]
impl ConnectorSink for OpenTSDBBridgePlugin {
    type SinkResource = Client;

    async fn validate(&self) -> Result<(), CommonError> {
        self.config.validate()
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
        self.build_client()
    }

    async fn send_batch(
        &self,
        records: &[StorageRecord],
        client: &mut Client,
    ) -> Result<Vec<FailureRecordInfo>, CommonError> {
        if records.is_empty() {
            return Ok(vec![]);
        }

        let mut data_points = Vec::new();
        let mut fail_messages = Vec::new();
        for record in records {
            match self.record_to_data_point(record).await {
                Ok(dp) => data_points.push(dp),
                Err(e) => {
                    error!(
                        "Failed to convert record to OpenTSDB data point: {}. Skipping.",
                        e
                    );
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
            }
        }

        if data_points.is_empty() {
            return Ok(fail_messages);
        }

        let url = self.config.put_url();
        let body = if data_points.len() == 1 {
            data_points.into_iter().next().unwrap()
        } else {
            Value::Array(data_points)
        };

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                CommonError::CommonError(format!("OpenTSDB HTTP request failed: {}", e))
            })?;

        if response.status().is_success() {
            Ok(fail_messages)
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(CommonError::CommonError(format!(
                "OpenTSDB returned HTTP {}: {}",
                status, error_text
            )))
        }
    }
}

pub fn start_opentsdb_connector(
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
        let bridge = match OpenTSDBBridgePlugin::new(connector.clone()) {
            Ok(bridge) => bridge,
            Err(e) => {
                error!(
                    "Invalid connector config type for OpenTSDB connector, connector_name='{}', connector_type='{}', error={}",
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
                "Failed to start OpenTSDBBridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}
