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

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use axum::async_trait;
use metadata_struct::mqtt::message::MqttMessage;
use metadata_struct::{
    adapter::adapter_record::AdapterWriteRecord, mqtt::bridge::config_redis::RedisConnectorConfig,
    mqtt::bridge::config_redis::RedisMode, mqtt::bridge::connector::MQTTConnector,
};
use redis::aio::ConnectionManager;
use redis::{Client, Cmd, RedisError};
use storage_adapter::storage::ArcStorageAdapter;
use tracing::{error, info, warn};

use crate::handler::error::MqttBrokerError;
use crate::handler::tool::ResultMqttBrokerError;

use super::{
    core::{run_connector_loop, BridgePluginReadConfig, BridgePluginThread, ConnectorSink},
    manager::ConnectorManager,
};
pub struct RedisBridgePlugin {
    config: RedisConnectorConfig,
}

impl RedisBridgePlugin {
    pub fn new(config: RedisConnectorConfig) -> Self {
        RedisBridgePlugin { config }
    }

    fn build_redis_client(&self) -> Result<Client, RedisError> {
        let connection_info = match self.config.mode {
            RedisMode::Single => {
                let mut url = String::from("redis://");

                match (&self.config.username, &self.config.password) {
                    (Some(username), Some(password)) => {
                        url.push_str(&format!("{}:{}@", username, password));
                    }
                    (None, Some(password)) => {
                        url.push_str(&format!(":{}@", password));
                    }
                    _ => {}
                }

                url.push_str(&self.config.server);
                url.push_str(&format!("/{}", self.config.database));

                if self.config.tls_enabled {
                    url = url.replace("redis://", "rediss://");
                }

                info!("Connecting to Redis (single mode): {}", url);
                url
            }
            RedisMode::Cluster => {
                let mut url = String::from("redis://");

                match (&self.config.username, &self.config.password) {
                    (Some(username), Some(password)) => {
                        url.push_str(&format!("{}:{}@", username, password));
                    }
                    (None, Some(password)) => {
                        url.push_str(&format!(":{}@", password));
                    }
                    _ => {}
                }

                url.push_str(&self.config.server);

                if self.config.tls_enabled {
                    url = url.replace("redis://", "rediss://");
                }

                info!("Connecting to Redis (cluster mode): {}", url);
                url
            }
            RedisMode::Sentinel => {
                let mut url = String::from("redis+sentinel://");

                match (&self.config.username, &self.config.password) {
                    (Some(username), Some(password)) => {
                        url.push_str(&format!("{}:{}@", username, password));
                    }
                    (None, Some(password)) => {
                        url.push_str(&format!(":{}@", password));
                    }
                    _ => {}
                }

                url.push_str(&self.config.server);
                if let Some(master_name) = &self.config.sentinel_master_name {
                    url.push_str(&format!("/{}", master_name));
                }

                info!("Connecting to Redis (sentinel mode): {}", url);
                url
            }
        };

        Client::open(connection_info)
    }

    fn render_command_template(
        &self,
        record: &AdapterWriteRecord,
        msg: &MqttMessage,
    ) -> Result<Vec<String>, MqttBrokerError> {
        let mut rendered = self.config.command_template.clone();

        let mut replacements = HashMap::new();
        replacements.insert("client_id", msg.client_id.clone());
        replacements.insert("topic", String::from_utf8_lossy(&msg.topic).to_string());
        replacements.insert("payload", String::from_utf8_lossy(&msg.payload).to_string());
        replacements.insert("timestamp", record.timestamp.to_string());
        replacements.insert("qos", format!("{:?}", msg.qos));
        replacements.insert("retain", msg.retain.to_string());
        replacements.insert("key", record.key.clone().unwrap_or_default());

        for (placeholder, value) in replacements.iter() {
            let pattern = format!("${{{}}}", placeholder);
            rendered = rendered.replace(&pattern, value);
        }

        let parts: Vec<String> = rendered.split_whitespace().map(|s| s.to_string()).collect();

        if parts.is_empty() {
            return Err(MqttBrokerError::CommonError(
                "Rendered command template is empty".to_string(),
            ));
        }

        Ok(parts)
    }
    async fn execute_command(
        &self,
        conn: &mut ConnectionManager,
        command_parts: Vec<String>,
    ) -> Result<(), RedisError> {
        if command_parts.is_empty() {
            return Ok(());
        }

        let mut cmd = Cmd::new();
        cmd.arg(&command_parts[0]);

        for arg in &command_parts[1..] {
            cmd.arg(arg);
        }

        cmd.query_async::<()>(conn).await?;
        Ok(())
    }
}

#[async_trait]
impl ConnectorSink for RedisBridgePlugin {
    type SinkResource = ConnectionManager;

    async fn validate(&self) -> ResultMqttBrokerError {
        self.config.validate()?;

        Ok(())
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, MqttBrokerError> {
        let client = self.build_redis_client().map_err(|e| {
            MqttBrokerError::CommonError(format!("Failed to build Redis client: {}", e))
        })?;

        let conn_manager = ConnectionManager::new(client).await.map_err(|e| {
            MqttBrokerError::CommonError(format!(
                "Failed to create Redis connection manager: {}",
                e
            ))
        })?;

        info!(
            "Redis connection manager initialized successfully: server={}, mode={:?}",
            self.config.server, self.config.mode
        );

        Ok(conn_manager)
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        conn: &mut ConnectionManager,
    ) -> ResultMqttBrokerError {
        let mut success_count = 0;
        let mut error_count = 0;

        for record in records {
            let msg = match MqttMessage::decode_record(record.clone()) {
                Ok(m) => m,
                Err(e) => {
                    error!("Failed to parse MQTT message: {}", e);
                    error_count += 1;
                    continue;
                }
            };

            let command_parts = match self.render_command_template(record, &msg) {
                Ok(parts) => parts,
                Err(e) => {
                    error!("Failed to render command template: {}", e);
                    error_count += 1;
                    continue;
                }
            };

            let mut retry_count = 0;
            let mut last_error = None;

            while retry_count <= self.config.max_retries {
                match self.execute_command(conn, command_parts.clone()).await {
                    Ok(_) => {
                        success_count += 1;
                        break;
                    }
                    Err(e) => {
                        last_error = Some(e);
                        retry_count += 1;

                        if retry_count <= self.config.max_retries {
                            warn!(
                                "Redis command failed, retrying ({}/{}): {}",
                                retry_count,
                                self.config.max_retries,
                                last_error.as_ref().unwrap()
                            );
                            tokio::time::sleep(Duration::from_millis(
                                self.config.retry_interval_ms,
                            ))
                            .await;
                        }
                    }
                }
            }

            if retry_count > self.config.max_retries {
                error!(
                    "Redis command failed after {} retries: {}",
                    self.config.max_retries,
                    last_error.unwrap()
                );
                error_count += 1;
            }
        }

        info!(
            "Redis batch send completed: success={}, errors={}",
            success_count, error_count
        );

        if error_count == records.len() && !records.is_empty() {
            return Err(MqttBrokerError::CommonError(
                "All records failed to send to Redis".to_string(),
            ));
        }

        Ok(())
    }

    async fn cleanup_sink(&self, _conn: ConnectionManager) -> ResultMqttBrokerError {
        info!("Redis connection manager cleanup completed");
        Ok(())
    }
}

pub fn start_redis_connector(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        let redis_config = match &connector.config {
            metadata_struct::mqtt::bridge::ConnectorConfig::Redis(config) => config.clone(),
            _ => {
                error!("Invalid connector config type, expected Redis config");
                return;
            }
        };

        let bridge = RedisBridgePlugin::new(redis_config);

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
            error!(
                "Redis connector loop error for connector {}: {}",
                connector.connector_name, e
            );
        }
    });
}
