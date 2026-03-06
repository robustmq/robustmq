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
    connector::config_webhook::{WebhookAuthType, WebhookConnectorConfig, WebhookHttpMethod},
    connector::MQTTConnector,
    storage::adapter_record::AdapterWriteRecord,
};
use reqwest::{Client, RequestBuilder};
use rule_engine::apply_rule_engine;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc::Receiver;
use tracing::error;

use super::{
    core::{BridgePluginReadConfig, BridgePluginThread},
    loops::run_connector_loop,
    manager::ConnectorManager,
    traits::ConnectorSink,
};

pub struct WebhookBridgePlugin {
    connector: MQTTConnector,
    config: WebhookConnectorConfig,
}

impl WebhookBridgePlugin {
    #[allow(clippy::result_large_err)]
    pub fn new(connector: MQTTConnector) -> Result<Self, CommonError> {
        let config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::Webhook(config) => config.clone(),
            _ => {
                return Err(CommonError::CommonError(
                    "invalid connector type for webhook plugin".to_string(),
                ));
            }
        };
        Ok(WebhookBridgePlugin { connector, config })
    }

    #[allow(clippy::result_large_err)]
    fn build_client(&self) -> Result<Client, CommonError> {
        Client::builder()
            .timeout(Duration::from_millis(self.config.timeout_ms))
            .build()
            .map_err(|e| CommonError::CommonError(format!("Failed to build HTTP client: {}", e)))
    }

    fn build_request(&self, client: &Client, body: String) -> RequestBuilder {
        let mut req = match self.config.method {
            WebhookHttpMethod::Post => client.post(&self.config.url),
            WebhookHttpMethod::Put => client.put(&self.config.url),
        };

        req = req.header("Content-Type", "application/json");

        for (key, value) in &self.config.headers {
            req = req.header(key, value);
        }

        match &self.config.auth_type {
            WebhookAuthType::Basic => {
                if let (Some(username), Some(password)) =
                    (&self.config.username, &self.config.password)
                {
                    req = req.basic_auth(username, Some(password));
                }
            }
            WebhookAuthType::Bearer => {
                if let Some(token) = &self.config.bearer_token {
                    req = req.bearer_auth(token);
                }
            }
            WebhookAuthType::None => {}
        }

        req.body(body)
    }

    async fn records_to_json(&self, records: &[AdapterWriteRecord]) -> String {
        let mut items: Vec<serde_json::Value> = Vec::with_capacity(records.len());
        for record in records {
            let processed_data = match apply_rule_engine(&self.connector.rules, &record.data).await
            {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!("Failed to apply rule before Webhook send: {}", e);
                    continue;
                }
            };

            let payload = String::from_utf8_lossy(&processed_data).to_string();
            let mut item = json!({
                "payload": payload,
                "timestamp": record.timestamp,
            });
            if let Some(key) = &record.key {
                item["key"] = json!(key);
            }
            if let Some(headers) = &record.header {
                if !headers.is_empty() {
                    let h: Vec<serde_json::Value> = headers
                        .iter()
                        .map(|h| json!({"name": h.name, "value": h.value}))
                        .collect();
                    item["headers"] = json!(h);
                }
            }
            items.push(item);
        }

        if items.len() == 1 {
            items[0].to_string()
        } else {
            json!(items).to_string()
        }
    }
}

#[async_trait]
impl ConnectorSink for WebhookBridgePlugin {
    type SinkResource = Client;

    async fn validate(&self) -> Result<(), CommonError> {
        self.config.validate()
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
        self.build_client()
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        client: &mut Client,
    ) -> Result<(), CommonError> {
        if records.is_empty() {
            return Ok(());
        }

        let body = self.records_to_json(records).await;
        let request = self.build_request(client, body);

        let response = request
            .send()
            .await
            .map_err(|e| CommonError::CommonError(format!("Webhook HTTP request failed: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(CommonError::CommonError(format!(
                "Webhook returned HTTP {}: {}",
                status, error_text
            )))
        }
    }
}

pub fn start_webhook_connector(
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
        let bridge = match WebhookBridgePlugin::new(connector.clone()) {
            Ok(bridge) => bridge,
            Err(e) => {
                error!(
                    "Invalid connector config type for Webhook connector, connector_name='{}', connector_type='{}', error={}",
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
            storage_driver_manager.clone(),
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
                "Failed to start WebhookBridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}
