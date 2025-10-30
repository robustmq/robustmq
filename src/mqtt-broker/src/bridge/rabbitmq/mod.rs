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
use lapin::{
    options::{BasicPublishOptions, ConfirmSelectOptions},
    BasicProperties, Connection, ConnectionProperties,
};
use metadata_struct::{
    adapter::record::Record, mqtt::bridge::config_rabbitmq::RabbitMQConnectorConfig,
    mqtt::bridge::connector::MQTTConnector,
};
use storage_adapter::storage::ArcStorageAdapter;
use tracing::error;

use crate::common::types::ResultMqttBrokerError;

use super::{
    core::{run_connector_loop, BridgePluginReadConfig, BridgePluginThread, ConnectorSink},
    manager::ConnectorManager,
};

pub struct RabbitMQBridgePlugin {
    config: RabbitMQConnectorConfig,
}

impl RabbitMQBridgePlugin {
    pub fn new(config: RabbitMQConnectorConfig) -> Self {
        RabbitMQBridgePlugin { config }
    }

    fn build_connection_uri(&self) -> String {
        let protocol = if self.config.enable_tls {
            "amqps"
        } else {
            "amqp"
        };
        format!(
            "{}://{}:{}@{}:{}/{}",
            protocol,
            self.config.username,
            self.config.password,
            self.config.server,
            self.config.port,
            self.config.virtual_host
        )
    }
}

#[async_trait]
impl ConnectorSink for RabbitMQBridgePlugin {
    type SinkResource = Connection;

    async fn validate(&self) -> ResultMqttBrokerError {
        Ok(())
    }

    async fn init_sink(
        &self,
    ) -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError> {
        let uri = self.build_connection_uri();
        let connection = Connection::connect(&uri, ConnectionProperties::default()).await?;
        Ok(connection)
    }

    async fn send_batch(
        &self,
        records: &[Record],
        connection: &mut Connection,
    ) -> ResultMqttBrokerError {
        let channel = connection.create_channel().await?;

        channel
            .confirm_select(ConfirmSelectOptions::default())
            .await?;

        for record in records {
            let data = serde_json::to_string(record)?;

            let properties = BasicProperties::default()
                .with_delivery_mode(self.config.delivery_mode.to_amqp_value());

            let confirm = channel
                .basic_publish(
                    &self.config.exchange,
                    &self.config.routing_key,
                    BasicPublishOptions::default(),
                    data.as_bytes(),
                    properties,
                )
                .await?;

            confirm.await?;
        }

        Ok(())
    }

    async fn cleanup_sink(&self, connection: Connection) -> ResultMqttBrokerError {
        connection.close(200, "Closing connection").await?;
        Ok(())
    }
}

pub fn start_rabbitmq_connector(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        let rabbitmq_config = match serde_json::from_str::<RabbitMQConnectorConfig>(
            &connector.config,
        ) {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to parse RabbitMQConnectorConfig with error message: {}, configuration contents: {}", e, connector.config);
                return;
            }
        };

        let bridge = RabbitMQBridgePlugin::new(rabbitmq_config);

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
                "Failed to start RabbitMQBridgePlugin with error message: {:?}",
                e
            );
        }
    });
}
