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

use std::sync::Arc;

use axum::async_trait;
use elasticsearch::{
    auth::Credentials,
    http::{
        request::JsonBody,
        transport::{SingleNodeConnectionPool, TransportBuilder},
    },
    BulkParts, Elasticsearch,
};
use metadata_struct::{
    adapter::record::StorageAdapterRecord,
    mqtt::bridge::config_elasticsearch::ElasticsearchConnectorConfig,
    mqtt::bridge::connector::MQTTConnector,
};
use serde_json::{json, Value};
use storage_adapter::storage::ArcStorageAdapter;
use tracing::error;

use crate::handler::error::MqttBrokerError;
use crate::handler::tool::ResultMqttBrokerError;

use super::{
    core::{run_connector_loop, BridgePluginReadConfig, BridgePluginThread, ConnectorSink},
    manager::ConnectorManager,
};

pub struct ElasticsearchBridgePlugin {
    config: ElasticsearchConnectorConfig,
}

impl ElasticsearchBridgePlugin {
    pub fn new(config: ElasticsearchConnectorConfig) -> Self {
        ElasticsearchBridgePlugin { config }
    }

    async fn create_client(&self) -> Result<Elasticsearch, MqttBrokerError> {
        let url = self
            .config
            .url
            .parse()
            .map_err(|e| MqttBrokerError::CommonError(format!("Invalid URL: {}", e)))?;

        let conn_pool = SingleNodeConnectionPool::new(url);
        let mut transport_builder = TransportBuilder::new(conn_pool);

        match self.config.auth_type {
            metadata_struct::mqtt::bridge::config_elasticsearch::ElasticsearchAuthType::Basic => {
                if let (Some(username), Some(password)) =
                    (&self.config.username, &self.config.password)
                {
                    transport_builder = transport_builder
                        .auth(Credentials::Basic(username.clone(), password.clone()));
                }
            }
            metadata_struct::mqtt::bridge::config_elasticsearch::ElasticsearchAuthType::ApiKey => {
                if let Some(api_key) = &self.config.api_key {
                    transport_builder = transport_builder
                        .auth(Credentials::ApiKey("".to_string(), api_key.clone()));
                }
            }
            metadata_struct::mqtt::bridge::config_elasticsearch::ElasticsearchAuthType::None => {}
        }

        let transport = transport_builder.build().map_err(|e| {
            MqttBrokerError::CommonError(format!("Failed to build transport: {}", e))
        })?;

        Ok(Elasticsearch::new(transport))
    }

    fn record_to_json(&self, record: &StorageAdapterRecord) -> Result<Value, MqttBrokerError> {
        let payload_str = String::from_utf8_lossy(&record.data).to_string();

        let mut doc = json!({
            "key": record.key,
            "timestamp": record.timestamp,
            "payload": payload_str,
            "data": record.data,
        });

        if let Some(headers) = &record.header {
            if !headers.is_empty() {
                let headers_vec: Vec<Value> = headers
                    .iter()
                    .map(|h| json!({"name": h.name, "value": h.value}))
                    .collect();
                doc["headers"] = json!(headers_vec);
            }
        }

        Ok(doc)
    }
}

#[async_trait]
impl ConnectorSink for ElasticsearchBridgePlugin {
    type SinkResource = Elasticsearch;

    async fn validate(&self) -> ResultMqttBrokerError {
        Ok(())
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, MqttBrokerError> {
        let client = self.create_client().await?;
        Ok(client)
    }

    async fn send_batch(
        &self,
        records: &[StorageAdapterRecord],
        client: &mut Elasticsearch,
    ) -> ResultMqttBrokerError {
        if records.is_empty() {
            return Ok(());
        }

        let mut body_parts: Vec<JsonBody<_>> = Vec::new();

        for record in records {
            let doc = match self.record_to_json(record) {
                Ok(d) => d,
                Err(e) => {
                    error!(
                        "Failed to convert record to JSON: {}. Record will be skipped.",
                        e
                    );
                    continue;
                }
            };

            let action = json!({"index": {}});
            body_parts.push(JsonBody::new(action));
            body_parts.push(JsonBody::new(doc));
        }

        if body_parts.is_empty() {
            return Ok(());
        }

        let response = client
            .bulk(BulkParts::Index(&self.config.index))
            .body(body_parts)
            .send()
            .await
            .map_err(|e| {
                MqttBrokerError::CommonError(format!("Elasticsearch bulk request failed: {}", e))
            })?;

        if response.status_code().is_success() {
            Ok(())
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(MqttBrokerError::CommonError(format!(
                "Elasticsearch bulk operation failed: {}",
                error_text
            )))
        }
    }
}

pub fn start_elasticsearch_connector(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        let es_config = match &connector.config {
            metadata_struct::mqtt::bridge::ConnectorConfig::Elasticsearch(config) => config.clone(),
            _ => {
                error!("Invalid connector config type, expected Elasticsearch config");
                return;
            }
        };
        let bridge = ElasticsearchBridgePlugin::new(es_config);

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
                "Failed to start ElasticsearchBridgePlugin with error message: {:?}",
                e
            );
        }
    });
}
