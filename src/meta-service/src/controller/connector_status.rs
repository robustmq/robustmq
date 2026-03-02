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
    core::notify::send_notify_by_add_connector,
    core::{cache::MetaCacheManager, error::MetaServiceError},
    raft::{
        manager::MultiRaftManager,
        route::data::{StorageData, StorageDataType},
    },
};
use bytes::Bytes;
use metadata_struct::mqtt::bridge::{connector::MQTTConnector, status::MQTTStatus};
use node_call::NodeCallManager;
use prost::Message;
use protocol::meta::meta_service_mqtt::CreateConnectorRequest;
use std::sync::Arc;
use tracing::{info, warn};

pub struct ConnectorStatus {
    raft_manager: Arc<MultiRaftManager>,
    node_call_manager: Arc<NodeCallManager>,
    cache_manager: Arc<MetaCacheManager>,
}

impl ConnectorStatus {
    pub fn new(
        raft_manager: Arc<MultiRaftManager>,
        node_call_manager: Arc<NodeCallManager>,
        cache_manager: Arc<MetaCacheManager>,
    ) -> Self {
        Self {
            raft_manager,
            node_call_manager,
            cache_manager,
        }
    }

    pub async fn update_status_to_idle(
        &self,
        connector_name: &str,
    ) -> Result<(), MetaServiceError> {
        info!("Updating connector {} status to Idle", connector_name);
        self.update_status(connector_name, MQTTStatus::Idle).await
    }

    pub async fn update_status_to_running(
        &self,
        connector_name: &str,
    ) -> Result<(), MetaServiceError> {
        info!("Updating connector {} status to Running", connector_name);
        self.update_status(connector_name, MQTTStatus::Running)
            .await
    }

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

        connector.status = status;

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

    pub async fn save_connector(&self, connector: MQTTConnector) -> Result<(), MetaServiceError> {
        self.save_connector_internal(connector).await
    }

    async fn save_connector_internal(
        &self,
        connector: MQTTConnector,
    ) -> Result<(), MetaServiceError> {
        let req = CreateConnectorRequest {
            connector_name: connector.connector_name.clone(),
            connector: connector.encode()?,
        };

        let data = StorageData::new(
            StorageDataType::MqttSetConnector,
            Bytes::copy_from_slice(&CreateConnectorRequest::encode_to_vec(&req)),
        );
        self.raft_manager.write_metadata(data).await?;

        send_notify_by_add_connector(&self.node_call_manager, connector).await?;

        Ok(())
    }
}
