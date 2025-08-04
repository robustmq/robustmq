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

use crate::admin::query::{apply_filters, apply_pagination, apply_sorting, Queryable};
use crate::common::types::ResultMqttBrokerError;
use crate::handler::error::MqttBrokerError;
use crate::storage::connector::ConnectorStorage;
use common_base::tools::now_second;
use common_config::broker::broker_config;
use grpc_clients::placement::mqtt::call::placement_list_connector;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::bridge::config_kafka::KafkaConnectorConfig;
use metadata_struct::mqtt::bridge::config_local_file::LocalFileConnectorConfig;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::bridge::connector_type::ConnectorType;
use metadata_struct::mqtt::bridge::status::MQTTStatus;
use protocol::broker_mqtt::broker_mqtt_admin::{
    self, ConnectorRaw, ListConnectorReply, ListConnectorRequest,
};
use protocol::placement_center::placement_center_mqtt;
use std::sync::Arc;

// List connectors by request
pub async fn list_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: &ListConnectorRequest,
) -> Result<broker_mqtt_admin::ListConnectorReply, MqttBrokerError> {
    let config = broker_config();
    let placement_request = placement_center_mqtt::ListConnectorRequest {
        cluster_name: config.cluster_name.clone(),
        connector_name: request.connector_name.clone(),
    };

    let connectors_byte = placement_list_connector(
        client_pool,
        &config.get_placement_center_addr(),
        placement_request,
    )
    .await
    .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?
    .connectors;
    let mut connectors = Vec::new();

    for connector_byte in connectors_byte {
        let restored_connector = MQTTConnector::decode(&connector_byte);
        connectors.push(ConnectorRaw::from(restored_connector));
    }

    let filtered = apply_filters(connectors, &request.options);
    let sorted = apply_sorting(filtered, &request.options);
    let pagination = apply_pagination(sorted, &request.options);

    Ok(ListConnectorReply {
        connectors: pagination.0,
        total_count: pagination.1 as u32,
    })
}

// Create a new connector
pub async fn create_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: &broker_mqtt_admin::CreateConnectorRequest,
) -> Result<broker_mqtt_admin::CreateConnectorReply, MqttBrokerError> {
    let connector_type = parse_mqtt_connector_type(request.connector_type());
    connector_config_validator(&connector_type, &request.config)?;

    let config = broker_config();
    let storage = ConnectorStorage::new(client_pool.clone());
    let connector = MQTTConnector {
        cluster_name: config.cluster_name.clone(),
        connector_name: request.connector_name.clone(),
        connector_type: parse_mqtt_connector_type(request.connector_type()),
        config: request.config.clone(),
        topic_id: request.topic_id.clone(),
        status: MQTTStatus::Idle,
        broker_id: None,
        create_time: now_second(),
        update_time: now_second(),
    };

    storage
        .create_connector(connector)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(broker_mqtt_admin::CreateConnectorReply {})
}

// Update an existing connector
pub async fn update_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: &broker_mqtt_admin::UpdateConnectorRequest,
) -> Result<broker_mqtt_admin::UpdateConnectorReply, MqttBrokerError> {
    let connector = serde_json::from_slice::<MQTTConnector>(&request.connector)
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    connector_config_validator(&connector.connector_type, &connector.config)?;

    let storage = ConnectorStorage::new(client_pool.clone());
    storage
        .update_connector(connector)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(broker_mqtt_admin::UpdateConnectorReply {})
}

// Delete an existing connector
pub async fn delete_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: &broker_mqtt_admin::DeleteConnectorRequest,
) -> Result<broker_mqtt_admin::DeleteConnectorReply, MqttBrokerError> {
    let config = broker_config();
    let storage = ConnectorStorage::new(client_pool.clone());

    storage
        .delete_connector(&config.cluster_name, &request.connector_name)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(broker_mqtt_admin::DeleteConnectorReply {})
}

fn connector_config_validator(
    connector_type: &ConnectorType,
    config: &str,
) -> ResultMqttBrokerError {
    match connector_type {
        ConnectorType::LocalFile => {
            let _file_config: LocalFileConnectorConfig = serde_json::from_str(config)?;
        }
        ConnectorType::Kafka => {
            let _kafka_config: KafkaConnectorConfig = serde_json::from_str(config)?;
        }
    }
    Ok(())
}

fn parse_mqtt_connector_type(connector_type: broker_mqtt_admin::ConnectorType) -> ConnectorType {
    match connector_type {
        broker_mqtt_admin::ConnectorType::File => ConnectorType::LocalFile,
        broker_mqtt_admin::ConnectorType::Kafka => ConnectorType::Kafka,
    }
}

impl Queryable for ConnectorRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "connector_name" => Some(self.connector_name.clone()),
            "status" => Some(self.status.to_string()),
            "connector_type" => Some(self.connector_type.to_string()),
            "topic_id" => Some(self.topic_id.clone()),
            "cluster_name" => Some(self.cluster_name.clone()),
            "create_time" => Some(self.create_time.to_string()),
            "update_time" => Some(self.update_time.to_string()),
            "broker_id" => self.broker_id.map(|id| id.to_string()),
            _ => None,
        }
    }
}
