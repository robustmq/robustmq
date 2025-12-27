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
use metadata_struct::{
    mqtt::bridge::config_greptimedb::GreptimeDBConnectorConfig,
    mqtt::bridge::connector::MQTTConnector, storage::adapter_record::AdapterWriteRecord,
};

use storage_adapter::storage::ArcStorageAdapter;
use tracing::error;

use crate::{
    bridge::{
        core::{run_connector_loop, BridgePluginReadConfig, BridgePluginThread, ConnectorSink},
        manager::ConnectorManager,
    },
    handler::tool::ResultMqttBrokerError,
};

mod sender;

pub struct GreptimeDBBridgePlugin {
    config: GreptimeDBConnectorConfig,
}

impl GreptimeDBBridgePlugin {
    pub fn new(config: GreptimeDBConnectorConfig) -> Self {
        GreptimeDBBridgePlugin { config }
    }
}

#[async_trait]
impl ConnectorSink for GreptimeDBBridgePlugin {
    type SinkResource = sender::Sender;

    async fn validate(&self) -> ResultMqttBrokerError {
        Ok(())
    }

    async fn init_sink(
        &self,
    ) -> Result<Self::SinkResource, crate::handler::error::MqttBrokerError> {
        sender::Sender::new(&self.config)
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        sender: &mut sender::Sender,
    ) -> ResultMqttBrokerError {
        sender.send_batch(records).await
    }
}

pub fn start_greptimedb_connector(
    connector_manager: Arc<ConnectorManager>,
    message_storage: ArcStorageAdapter,
    connector: MQTTConnector,
    thread: BridgePluginThread,
) {
    tokio::spawn(async move {
        let greptimedb_config = match &connector.config {
            metadata_struct::mqtt::bridge::ConnectorConfig::GreptimeDB(config) => config.clone(),
            _ => {
                error!("Invalid connector config type, expected GreptimeDB config");
                return;
            }
        };

        let bridge = GreptimeDBBridgePlugin::new(greptimedb_config);

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
                "Failed to start GreptimeDBBridgePlugin with error message: {:?}",
                e
            );
        }
    });
}
