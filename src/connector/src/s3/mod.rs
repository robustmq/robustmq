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
use common_base::{error::common::CommonError, tools::now_millis, uuid::unique_id};
use grpc_clients::pool::ClientPool;
use metadata_struct::{
    connector::{config_s3::S3ConnectorConfig, MQTTConnector},
    storage::adapter_record::{AdapterWriteRecord, AdapterWriteRecordHeader},
};
use opendal::{services::S3, Operator};
use serde::Serialize;
use std::sync::Arc;
use storage_adapter::driver::StorageDriverManager;
use tokio::sync::mpsc::Receiver;
use tracing::{debug, error};

use super::{
    core::{BridgePluginReadConfig, BridgePluginThread},
    loops::run_connector_loop,
    manager::ConnectorManager,
    traits::ConnectorSink,
};

#[derive(Serialize)]
struct S3MessageRecord {
    pkid: u64,
    key: Option<String>,
    headers: Option<Vec<AdapterWriteRecordHeader>>,
    tags: Option<Vec<String>>,
    data: Vec<u8>,
    timestamp: u64,
}

pub struct S3BridgePlugin {
    config: S3ConnectorConfig,
}

impl S3BridgePlugin {
    pub fn new(config: S3ConnectorConfig) -> Self {
        S3BridgePlugin { config }
    }

    fn normalize_prefix(prefix: &str) -> &str {
        prefix.trim_matches('/')
    }

    fn build_object_key(&self) -> String {
        let prefix = Self::normalize_prefix(&self.config.object_key_prefix);
        let extension = self.config.file_extension.trim_start_matches('.');
        format!("{}/{}-{}.{}", prefix, now_millis(), unique_id(), extension)
    }

    #[allow(clippy::result_large_err)]
    fn build_operator(&self) -> Result<Operator, CommonError> {
        let mut builder = S3::default()
            .bucket(&self.config.bucket)
            .region(&self.config.region);

        if !self.config.endpoint.is_empty() {
            builder = builder.endpoint(&self.config.endpoint);
        }

        if !self.config.access_key_id.is_empty() {
            builder = builder.access_key_id(&self.config.access_key_id);
            builder = builder.secret_access_key(&self.config.secret_access_key);
        }

        if !self.config.session_token.is_empty() {
            builder = builder.session_token(&self.config.session_token);
        }

        if !self.config.root.is_empty() {
            builder = builder.root(&self.config.root);
        }

        Ok(Operator::new(builder)?.finish())
    }
}

#[async_trait]
impl ConnectorSink for S3BridgePlugin {
    type SinkResource = Operator;

    async fn validate(&self) -> Result<(), CommonError> {
        self.config.validate()
    }

    async fn init_sink(&self) -> Result<Self::SinkResource, CommonError> {
        let op = self.build_operator()?;
        debug!(
            "S3 connector initialized: bucket={}, region={}, root={}",
            self.config.bucket, self.config.region, self.config.root
        );
        Ok(op)
    }

    async fn send_batch(
        &self,
        records: &[AdapterWriteRecord],
        operator: &mut Operator,
    ) -> Result<(), CommonError> {
        if records.is_empty() {
            return Ok(());
        }

        let payload: Vec<S3MessageRecord> = records
            .iter()
            .map(|record| S3MessageRecord {
                pkid: record.pkid,
                key: record.key.clone(),
                headers: record.header.clone(),
                tags: record.tags.clone(),
                data: record.data.to_vec(),
                timestamp: record.timestamp,
            })
            .collect();

        let object_key = self.build_object_key();
        let bytes = serde_json::to_vec(&payload).map_err(|e| {
            CommonError::CommonError(format!("Failed to serialize S3 payload to JSON: {}", e))
        })?;
        operator.write(&object_key, bytes).await?;

        Ok(())
    }
}

pub fn start_s3_connector(
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
        let s3_config = match &connector.connector_type {
            metadata_struct::connector::ConnectorType::S3(config) => config.clone(),
            _ => {
                error!(
                    "Invalid connector config type for S3 connector, connector_name='{}', connector_type='{}'",
                    connector_name, connector_type
                );
                return;
            }
        };
        let bridge = S3BridgePlugin::new(s3_config);
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
                "Failed to start S3BridgePlugin, connector_name='{}', connector_type='{}', error={:?}",
                connector_name, connector_type, e
            );
        }
    }));
}
