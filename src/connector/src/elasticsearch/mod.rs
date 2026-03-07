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
use elasticsearch::{
    auth::Credentials,
    http::{
        request::JsonBody,
        transport::{SingleNodeConnectionPool, TransportBuilder},
    },
    BulkParts, Elasticsearch,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    connector::config_elasticsearch::ElasticsearchConnectorConfig, connector::MQTTConnector,
    storage::adapter_record::AdapterWriteRecord,
};
use rule_engine::apply_rule_engine;
use serde_json::{json, Value};
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc::Receiver;
use tracing::error;

use common_base::error::common::CommonError;

use super::{
    core::{BridgePluginReadConfig, BridgePluginThread},
    failure::FailureRecordInfo,
    loops::run_connector_loop,
    manager::ConnectorManager,
    traits::ConnectorSink,
};

pub struct ElasticsearchBridgePlugin {
    connector: MQTTConnector,
    config: ElasticsearchConnectorConfig,
}

impl ElasticsearchBridgePlugin {
    #[allow(clippy::result_large_err)]
    pub fn new(connector: MQTTConnector) -> Result<Self, CommonError> {
        let config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::Elasticsearch(config) => config.clone(),
            _ => {
                return Err(CommonError::CommonError(
                    "invalid connector type for elasticsearch plugin".to_string(),
                ));
            }
        };
        Ok(ElasticsearchBridgePlugin { connector, config })
    }

    async fn create_client(&self) -> Result<Elasticsearch, CommonError> {
        let url = self
            .config
            .url
            .parse()
            .map_err(|e| CommonError::CommonError(format!("Invalid URL: {}", e)))?;

        let conn_pool = SingleNodeConnectionPool::new(url);
        let mut transport_builder = TransportBuilder::new(conn_pool);

        match self.config.auth_type {
            metadata_struct::connector::config_elasticsearch::ElasticsearchAuthType::Basic => {
                if let (Some(username), Some(password)) =
                    (&self.config.username, &self.config.password)
                {
                    transport_builder = transport_builder
                        .auth(Credentials::Basic(username.clone(), password.clone()));
                }
            }
            metadata_struct::connector::config_elasticsearch::ElasticsearchAuthType::ApiKey => {
                if let Some(api_key) = &self.config.api_key {
                    transport_builder = transport_builder
                        .auth(Credentials::ApiKey("".to_string(), api_key.clone()));
                }
            }
            metadata_struct::connector::config_elasticsearch::ElasticsearchAuthType::None => {}
        }

        let transport = transport_builder
            .build()
            .map_err(|e| CommonError::CommonError(format!("Failed to build transport: {}", e)))?;

        Ok(Elasticsearch::new(transport))
    }

    #[allow(clippy::result_large_err)]
    async fn record_to_json(&self, record: &AdapterWriteRecord) -> Result<Value, CommonError> {
        let processed_data = apply_rule_engine(&self.connector.etl_rule, &record.data).await?;
        let payload_str = String::from_utf8_lossy(&processed_data).to_string();

        let mut doc = json!({
            "key": record.key,
            "timestamp": record.timestamp,
            "payload": payload_str,
            "data": processed_data,
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

    async fn validate(&self) -> Result<(), CommonError> {
        Ok(())
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
        let client = self.create_client().await?;
        Ok(client)
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        client: &mut Elasticsearch,
    ) -> Result<Vec<FailureRecordInfo>, CommonError> {
        if records.is_empty() {
            return Ok(vec![]);
        }

        let mut body_parts: Vec<JsonBody<_>> = Vec::new();
        let mut fail_messages = Vec::new();

        for record in records {
            let doc = match self.record_to_json(record).await {
                Ok(d) => d,
                Err(e) => {
                    error!(
                        "Failed to convert record to JSON: {}. Record will be skipped.",
                        e
                    );
                    fail_messages.push(FailureRecordInfo {
                        connector_name: self.connector.connector_name.clone(),
                        connector_type: self.connector.connector_type.to_string(),
                        source_topic: self.connector.topic_name.clone(),
                        error_message: e.to_string(),
                        records: vec![record.clone()],
                    });
                    continue;
                }
            };

            let action = json!({"index": {}});
            body_parts.push(JsonBody::new(action));
            body_parts.push(JsonBody::new(doc));
        }

        if body_parts.is_empty() {
            return Ok(fail_messages);
        }

        let response = client
            .bulk(BulkParts::Index(&self.config.index))
            .body(body_parts)
            .send()
            .await
            .map_err(|e| {
                CommonError::CommonError(format!("Elasticsearch bulk request failed: {}", e))
            })?;

        if response.status_code().is_success() {
            Ok(fail_messages)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(CommonError::CommonError(format!(
                "Elasticsearch bulk operation failed: {}",
                error_text
            )))
        }
    }
}

pub fn start_elasticsearch_connector(
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
        let bridge = match ElasticsearchBridgePlugin::new(connector.clone()) {
            Ok(bridge) => bridge,
            Err(e) => {
                error!(
                    "Invalid connector config type for Elasticsearch connector, connector_name='{}', connector_type='{}', error={}",
                    connector_name, connector_type, e
                );
                return;
            }
        };

        connector_manager.add_connector_thread(&connector.connector_name, thread);

        if let Err(e) = run_connector_loop(
            &bridge,
            &client_pool,
            &connector_manager,
            &storage_driver_manager,
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
                "Failed to start ElasticsearchBridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}
