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

use crate::{
    core::{BridgePluginReadConfig, BridgePluginThread},
    failure::FailureRecordInfo,
    loops::run_connector_loop,
    manager::ConnectorManager,
    traits::ConnectorSink,
};
use async_trait::async_trait;
use common_base::error::common::CommonError;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::convert::convert_engine_record_to_adapter;
use metadata_struct::{
    connector::config_pulsar::PulsarConnectorConfig, connector::MQTTConnector,
    storage::storage_record::StorageRecord,
};
use rule_engine::apply_rule_engine;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc::Receiver;
use tracing::error;
mod pulsar_producer;

pub struct PulsarBridgePlugin {
    connector: MQTTConnector,
    config: PulsarConnectorConfig,
}

impl PulsarBridgePlugin {
    #[allow(clippy::result_large_err)]
    pub fn new(connector: MQTTConnector) -> Result<Self, CommonError> {
        let config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::Pulsar(config) => config.clone(),
            _ => {
                return Err(CommonError::CommonError(
                    "invalid connector type for pulsar plugin".to_string(),
                ));
            }
        };
        Ok(PulsarBridgePlugin { connector, config })
    }
}

#[async_trait]
impl ConnectorSink for PulsarBridgePlugin {
    type SinkResource = pulsar::producer::Producer<pulsar::TokioExecutor>;

    async fn validate(&self) -> Result<(), CommonError> {
        Ok(())
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
        let producer = pulsar_producer::Producer::new(&self.config)
            .build_producer()
            .await?;
        Ok(producer)
    }

    async fn send_batch(
        &self,
        records: &[StorageRecord],
        producer: &mut pulsar::producer::Producer<pulsar::TokioExecutor>,
    ) -> Result<Vec<FailureRecordInfo>, CommonError> {
        let mut fail_messages = Vec::new();
        for record in records {
            let processed_data =
                match apply_rule_engine(&self.connector.etl_rule, &record.data).await {
                    Ok(data) => data,
                    Err(e) => {
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
                };
            let mut write_record = convert_engine_record_to_adapter(record.clone());
            write_record.data = processed_data;
            producer.send_non_blocking(write_record).await?;
        }
        Ok(fail_messages)
    }
}

pub fn start_pulsar_connector(
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
        let bridge = match PulsarBridgePlugin::new(connector.clone()) {
            Ok(bridge) => bridge,
            Err(e) => {
                error!(
                    "Invalid connector config type for Pulsar connector, connector_name='{}', connector_type='{}', error={}",
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
                "Failed to start PulsarBridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}
