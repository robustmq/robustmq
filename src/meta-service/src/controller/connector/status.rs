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

use crate::{
    controller::call_broker::mqtt::{update_cache_by_add_connector, BrokerCallManager},
    core::{cache::CacheManager, error::MetaServiceError},
    raft::{
        manager::MultiRaftManager,
        route::data::{StorageData, StorageDataType},
    },
};
use bytes::Bytes;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::bridge::{connector::MQTTConnector, status::MQTTStatus};
use prost::Message;
use protocol::meta::meta_service_mqtt::CreateConnectorRequest;
use std::sync::Arc;
use tracing::{info, warn};

/// Connector status management context - encapsulates shared dependencies
pub struct ConnectorContext {
    raft_manager: Arc<MultiRaftManager>,
    call_manager: Arc<BrokerCallManager>,
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
}

impl ConnectorContext {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        call_manager: Arc<BrokerCallManager>,
        client_pool: Arc<ClientPool>,
        cache_manager: Arc<CacheManager>,
    ) -> Self {
        Self {
            raft_manager,
            call_manager,
            client_pool,
            cache_manager,
        }
    }

    /// Update connector status to Idle and clear broker assignment
    pub async fn update_status_to_idle(
        &self,
        connector_name: &str,
    ) -> Result<(), MetaServiceError> {
        info!("Updating connector {} status to Idle", connector_name);
        self.update_status(connector_name, MQTTStatus::Idle).await
    }

    /// Update connector status to Running
    pub async fn update_status_to_running(
        &self,
        connector_name: &str,
    ) -> Result<(), MetaServiceError> {
        info!("Updating connector {} status to Running", connector_name);
        self.update_status(connector_name, MQTTStatus::Running)
            .await
    }

    /// Update connector status to the specified value
    async fn update_status(
        &self,
        connector_name: &str,
        status: MQTTStatus,
    ) -> Result<(), MetaServiceError> {
        let mut connector = self
            .cache_manager
            .connector_list
            .get(connector_name)
            .ok_or_else(|| {
                warn!(
                    "Connector {} not found during status update",
                    connector_name
                );
                MetaServiceError::ConnectorNotFound(connector_name.to_string())
            })?
            .clone();

        let old_status = connector.status.clone();
        let new_status = status.clone();

        // Update status
        connector.status = status;

        // Clear broker_id when status changes to Idle
        if connector.status == MQTTStatus::Idle {
            connector.broker_id = None;
            info!(
                "Connector {} status changed: {:?} -> {:?}, broker_id cleared",
                connector_name, old_status, new_status
            );
        } else {
            info!(
                "Connector {} status changed: {:?} -> {:?}",
                connector_name, old_status, new_status
            );
        }

        self.save_connector_internal(connector).await
    }

    /// Save connector to Raft and update cache
    pub async fn save_connector(&self, connector: MQTTConnector) -> Result<(), MetaServiceError> {
        self.save_connector_internal(connector).await
    }

    /// Internal implementation for saving connector
    async fn save_connector_internal(
        &self,
        connector: MQTTConnector,
    ) -> Result<(), MetaServiceError> {
        let req = CreateConnectorRequest {
            connector_name: connector.connector_name.clone(),
            connector: connector.encode()?,
        };

        // Write to Raft for persistence
        let data = StorageData::new(
            StorageDataType::MqttSetConnector,
            Bytes::copy_from_slice(&CreateConnectorRequest::encode_to_vec(&req)),
        );
        self.raft_manager.write_metadata(data).await?;

        // Update cache across all brokers
        update_cache_by_add_connector(&self.call_manager, &self.client_pool, connector).await?;

        Ok(())
    }
}
