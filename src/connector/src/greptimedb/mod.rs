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

use rule_engine::apply_rule_engine;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc::Receiver;
use tracing::error;

use crate::{
    core::{BridgePluginReadConfig, BridgePluginThread},
    failure::FailureRecordInfo,
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
    #[allow(clippy::result_large_err)]
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

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        sender: &mut sender::Sender,
    ) -> Result<Vec<FailureRecordInfo>, CommonError> {
        let mut processed_records = Vec::with_capacity(records.len());
        let mut fail_messages = Vec::new();
        for record in records {
            let processed_data =
                match apply_rule_engine(&self.connector.etl_rule, &record.data).await {
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
            let mut processed_record = record.clone();
            processed_record.data = processed_data;
            processed_records.push(processed_record);
        }
        if processed_records.is_empty() {
            return Ok(fail_messages);
        }
        sender.send_batch(&processed_records).await?;
        Ok(fail_messages)
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
                "Failed to start GreptimeDBBridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}
