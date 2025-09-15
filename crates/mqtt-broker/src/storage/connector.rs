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

use common_base::error::ResultCommonError;
use common_config::broker::broker_config;
use grpc_clients::{
    meta::mqtt::call::{
        placement_connector_heartbeat, placement_create_connector, placement_delete_connector,
        placement_list_connector, placement_update_connector,
    },
    pool::ClientPool,
};
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use protocol::meta::placement_center_mqtt::{
    ConnectorHeartbeatRaw, ConnectorHeartbeatRequest, CreateConnectorRequest,
    DeleteConnectorRequest, ListConnectorRequest, UpdateConnectorRequest,
};

use crate::common::types::ResultMqttBrokerError;
use crate::handler::error::MqttBrokerError;

pub struct ConnectorStorage {
    client_pool: Arc<ClientPool>,
}

impl ConnectorStorage {
    pub fn new(client_pool: Arc<ClientPool>) -> Self {
        ConnectorStorage { client_pool }
    }

    pub async fn list_connector(
        &self,
        connector_name: &str,
    ) -> Result<Vec<MQTTConnector>, MqttBrokerError> {
        let config = broker_config();
        let request = ListConnectorRequest {
            cluster_name: config.cluster_name.clone(),
            connector_name: connector_name.to_owned(),
        };
        let reply = placement_list_connector(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        let mut list = Vec::new();
        for raw in reply.connectors {
            list.push(serde_json::from_slice::<MQTTConnector>(raw.as_slice())?);
        }
        Ok(list)
    }

    pub async fn list_all_connectors(&self) -> Result<Vec<MQTTConnector>, MqttBrokerError> {
        self.list_connector("").await
    }

    pub async fn create_connector(&self, connector: MQTTConnector) -> ResultCommonError {
        let config = broker_config();
        let request = CreateConnectorRequest {
            cluster_name: config.cluster_name.clone(),
            connector_name: connector.connector_name.clone(),
            connector: connector.encode(),
        };
        placement_create_connector(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn update_connector(&self, connector: MQTTConnector) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = UpdateConnectorRequest {
            cluster_name: config.cluster_name.clone(),
            connector_name: connector.connector_name.clone(),
            connector: connector.encode(),
        };
        placement_update_connector(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn delete_connector(
        &self,
        cluster_name: &str,
        connector_name: &str,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = DeleteConnectorRequest {
            cluster_name: cluster_name.to_owned(),
            connector_name: connector_name.to_owned(),
        };
        placement_delete_connector(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(())
    }

    pub async fn connector_heartbeat(
        &self,
        heatbeats: Vec<ConnectorHeartbeatRaw>,
    ) -> ResultMqttBrokerError {
        let config = broker_config();
        let request = ConnectorHeartbeatRequest {
            cluster_name: config.cluster_name.clone(),
            heatbeats,
        };
        placement_connector_heartbeat(
            &self.client_pool,
            &config.get_placement_center_addr(),
            request,
        )
        .await?;
        Ok(())
    }
}
