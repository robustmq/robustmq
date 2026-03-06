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

use async_trait::async_trait;
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    connector::config_greptimedb::GreptimeDBConnectorConfig, connector::MQTTConnector,
    storage::adapter_record::AdapterWriteRecord,
};

use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc::Receiver;
use tracing::error;

use crate::{
    core::{BridgePluginReadConfig, BridgePluginThread},
    loops::run_connector_loop,
    manager::ConnectorManager,
    traits::ConnectorSink,
};
use common_base::error::common::CommonError;

mod sender;

pub struct GreptimeDBBridgePlugin {
    connector: MQTTConnector,
    config: GreptimeDBConnectorConfig,
}

impl GreptimeDBBridgePlugin {
    pub fn new(connector: MQTTConnector) -> Result<Self, CommonError> {
        let config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::GreptimeDB(config) => config.clone(),
            _ => {
                return Err(CommonError::CommonError(
                    "invalid connector type for greptimedb plugin".to_string(),
                ));
            }
        };
        Ok(GreptimeDBBridgePlugin { connector, config })
    }
}

#[async_trait]
impl ConnectorSink for GreptimeDBBridgePlugin {
    type SinkResource = sender::Sender;

    async fn validate(&self) -> Result<(), CommonError> {
        Ok(())
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
        sender::Sender::new(&self.config)
    }

    async fn apply_rule(
        &self,
        _rules: &Vec<metadata_struct::connector::rule::ETLRule>,
        data: &bytes::Bytes,
    ) -> Result<bytes::Bytes, CommonError> {
        Ok(data.clone())
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        sender: &mut sender::Sender,
    ) -> Result<(), CommonError> {
        sender.send_batch(records).await
    }
}

pub fn start_greptimedb_connector(
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
        let bridge = match GreptimeDBBridgePlugin::new(connector.clone()) {
            Ok(bridge) => bridge,
            Err(e) => {
                error!(
                    "Invalid connector config type for GreptimeDB connector, connector_name='{}', connector_type='{}', error={}",
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
                "Failed to start GreptimeDBBridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}
