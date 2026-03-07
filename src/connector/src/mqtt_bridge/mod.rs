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
    connector::config_mqtt::MqttBridgeConnectorConfig, connector::config_mqtt::MqttProtocolVersion,
    connector::MQTTConnector, storage::adapter_record::AdapterWriteRecord,
};
use paho_mqtt as mqtt;
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

pub struct MqttBridgePlugin {
    connector: MQTTConnector,
    config: MqttBridgeConnectorConfig,
}

impl MqttBridgePlugin {
    #[allow(clippy::result_large_err)]
    pub fn new(connector: MQTTConnector) -> Result<Self, CommonError> {
        let config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::MqttBridge(config) => config.clone(),
            _ => {
                return Err(CommonError::CommonError(
                    "invalid connector type for mqtt bridge plugin".to_string(),
                ));
            }
        };
        Ok(MqttBridgePlugin { connector, config })
    }

    fn build_target_topic(&self, record: &AdapterWriteRecord) -> String {
        let original_topic = record.key.as_deref().unwrap_or("robustmq/bridge/default");

        if let Some(prefix) = &self.config.topic_prefix {
            format!("{}/{}", prefix.trim_end_matches('/'), original_topic)
        } else {
            original_topic.to_string()
        }
    }
}

#[async_trait]
impl ConnectorSink for MqttBridgePlugin {
    type SinkResource = mqtt::AsyncClient;

    async fn validate(&self) -> Result<(), CommonError> {
        self.config.validate()
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
        let client_id = if let Some(prefix) = &self.config.client_id_prefix {
            format!("{}:{}", prefix, common_base::uuid::unique_id())
        } else {
            format!("robustmq-bridge:{}", common_base::uuid::unique_id())
        };

        let create_opts = mqtt::CreateOptionsBuilder::new()
            .server_uri(&self.config.server)
            .client_id(&client_id)
            .finalize();

        let client = mqtt::AsyncClient::new(create_opts).map_err(|e| {
            CommonError::CommonError(format!("Failed to create MQTT client: {}", e))
        })?;

        let conn_opts = {
            let mut conn_builder = match self.config.protocol_version {
                MqttProtocolVersion::V5 => mqtt::ConnectOptionsBuilder::new_v5(),
                _ => mqtt::ConnectOptionsBuilder::new(),
            };
            conn_builder
                .keep_alive_interval(Duration::from_secs(self.config.keepalive_secs))
                .connect_timeout(Duration::from_secs(self.config.connect_timeout_secs))
                .clean_session(true);

            if let Some(username) = &self.config.username {
                conn_builder.user_name(username);
            }
            if let Some(password) = &self.config.password {
                conn_builder.password(password);
            }

            if self.config.enable_tls {
                let ssl_opts = mqtt::SslOptionsBuilder::new().finalize();
                conn_builder.ssl_options(ssl_opts);
            }

            conn_builder.finalize()
        };

        client.connect(conn_opts).await.map_err(|e| {
            CommonError::CommonError(format!(
                "Failed to connect to MQTT broker {}: {}",
                self.config.server, e
            ))
        })?;

        debug!(
            "Connected to remote MQTT broker: {} as {}",
            self.config.server, client_id
        );

        Ok(client)
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        client: &mut mqtt::AsyncClient,
    ) -> Result<Vec<FailureRecordInfo>, CommonError> {
        if records.is_empty() {
            return Ok(vec![]);
        }

        if !client.is_connected() {
            return Err(CommonError::CommonError(
                "MQTT bridge client is disconnected".to_string(),
            ));
        }

        let mut fail_messages = Vec::new();
        for record in records {
            let topic = self.build_target_topic(record);
            let payload = match apply_rule_engine(&self.connector.etl_rule, &record.data).await {
                Ok(data) => data,
                Err(e) => {
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

            let msg = mqtt::MessageBuilder::new()
                .topic(&topic)
                .payload(payload)
                .qos(self.config.qos)
                .retained(self.config.retain)
                .finalize();

            client.publish(msg).await.map_err(|e| {
                CommonError::CommonError(format!(
                    "Failed to publish to remote MQTT broker topic '{}': {}",
                    topic, e
                ))
            })?;
        }

        Ok(fail_messages)
    }
}

pub fn start_mqtt_bridge_connector(
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
        let bridge = match MqttBridgePlugin::new(connector.clone()) {
            Ok(bridge) => bridge,
            Err(e) => {
                error!(
                    "Invalid connector config type for MqttBridge connector, connector_name='{}', connector_type='{}', error={}",
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
                "Failed to start MqttBridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}
