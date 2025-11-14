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
    controller::mqtt::call_broker::{update_cache_by_add_connector, MQTTInnerCallManager},
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
    call_manager: Arc<MQTTInnerCallManager>,
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<CacheManager>,
}

impl ConnectorContext {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        call_manager: Arc<MQTTInnerCallManager>,
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
        cluster_name: &str,
        connector_name: &str,
    ) -> Result<(), MetaServiceError> {
        info!(
            "Updating connector {} status to Idle in cluster {}",
            connector_name, cluster_name
        );
        self.update_status(cluster_name, connector_name, MQTTStatus::Idle)
            .await
    }

    /// Update connector status to Running
    pub async fn update_status_to_running(
        &self,
        cluster_name: &str,
        connector_name: &str,
    ) -> Result<(), MetaServiceError> {
        info!(
            "Updating connector {} status to Running in cluster {}",
            connector_name, cluster_name
        );
        self.update_status(cluster_name, connector_name, MQTTStatus::Running)
            .await
    }

    /// Update connector status to the specified value
    async fn update_status(
        &self,
        cluster_name: &str,
        connector_name: &str,
        status: MQTTStatus,
    ) -> Result<(), MetaServiceError> {
        let mut connector = self
            .cache_manager
            .get_connector(cluster_name, connector_name)
            .ok_or_else(|| {
                warn!(
                    "Connector {} not found in cluster {} during status update",
                    connector_name, cluster_name
                );
                MetaServiceError::ConnectorNotFound(connector_name.to_string())
            })?;

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
            cluster_name: connector.cluster_name.clone(),
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
        update_cache_by_add_connector(
            &req.cluster_name,
            &self.call_manager,
            &self.client_pool,
            connector,
        )
        .await?;

        Ok(())
    }
}
