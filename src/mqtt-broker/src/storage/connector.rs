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

use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::{
    placement::mqtt::call::{
        placement_connector_heartbeat, placement_create_connector, placement_delete_connector,
        placement_list_connector, placement_update_connector,
    },
    pool::ClientPool,
};
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use protocol::placement_center::placement_center_mqtt::{
    ConnectorHeartbeatRaw, ConnectorHeartbeatRequest, CreateConnectorRequest,
    DeleteConnectorRequest, ListConnectorRequest, UpdateConnectorRequest,
};

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
        let config = broker_mqtt_conf();
        let request = ListConnectorRequest {
            cluster_name: config.cluster_name.clone(),
            connector_name: connector_name.to_owned(),
        };
        let reply =
            placement_list_connector(&self.client_pool, &config.placement_center, request).await?;
        let mut list = Vec::new();
        for raw in reply.connectors {
            list.push(serde_json::from_slice::<MQTTConnector>(raw.as_slice())?);
        }
        Ok(list)
    }

    pub async fn list_all_connectors(&self) -> Result<Vec<MQTTConnector>, MqttBrokerError> {
        self.list_connector("").await
    }

    pub async fn create_connector(&self, connector: MQTTConnector) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = CreateConnectorRequest {
            cluster_name: config.cluster_name.clone(),
            connector_name: connector.connector_name.clone(),
            connector: connector.encode(),
        };
        placement_create_connector(&self.client_pool, &config.placement_center, request).await?;
        Ok(())
    }

    pub async fn update_connector(&self, connector: MQTTConnector) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = UpdateConnectorRequest {
            cluster_name: config.cluster_name.clone(),
            connector_name: connector.connector_name.clone(),
            connector: connector.encode(),
        };
        placement_update_connector(&self.client_pool, &config.placement_center, request).await?;
        Ok(())
    }

    pub async fn delete_connector(
        &self,
        cluster_name: &str,
        connector_name: &str,
    ) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = DeleteConnectorRequest {
            cluster_name: cluster_name.to_owned(),
            connector_name: connector_name.to_owned(),
        };
        placement_delete_connector(&self.client_pool, &config.placement_center, request).await?;
        Ok(())
    }

    pub async fn connector_heartbeat(
        &self,
        heatbeats: Vec<ConnectorHeartbeatRaw>,
    ) -> Result<(), MqttBrokerError> {
        let config = broker_mqtt_conf();
        let request = ConnectorHeartbeatRequest {
            cluster_name: config.cluster_name.clone(),
            heatbeats,
        };
        placement_connector_heartbeat(&self.client_pool, &config.placement_center, request).await?;
        Ok(())
    }
}
