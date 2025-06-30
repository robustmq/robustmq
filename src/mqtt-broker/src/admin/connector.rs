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

use crate::handler::error::MqttBrokerError;
use crate::storage::connector::ConnectorStorage;
use common_base::tools::now_second;
use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::placement::mqtt::call::placement_list_connector;
use grpc_clients::pool::ClientPool;
use metadata_struct::mqtt::bridge::config_kafka::KafkaConnectorConfig;
use metadata_struct::mqtt::bridge::config_local_file::LocalFileConnectorConfig;
use metadata_struct::mqtt::bridge::connector::MQTTConnector;
use metadata_struct::mqtt::bridge::connector_type::ConnectorType;
use metadata_struct::mqtt::bridge::status::MQTTStatus;
use protocol::broker_mqtt::broker_mqtt_admin::{
    MqttConnectorType, MqttCreateConnectorReply, MqttCreateConnectorRequest,
    MqttDeleteConnectorReply, MqttDeleteConnectorRequest, MqttListConnectorReply,
    MqttListConnectorRequest, MqttUpdateConnectorReply, MqttUpdateConnectorRequest,
};
use protocol::placement_center::placement_center_mqtt::ListConnectorRequest;
use std::sync::Arc;

// List connectors by request
pub async fn list_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: &MqttListConnectorRequest,
) -> Result<MqttListConnectorReply, MqttBrokerError> {
    let config = broker_mqtt_conf();
    let request = ListConnectorRequest {
        cluster_name: config.cluster_name.clone(),
        connector_name: request.connector_name.clone(),
    };

    let connectors = placement_list_connector(client_pool, &config.placement_center, request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?
        .connectors;

    Ok(MqttListConnectorReply { connectors })
}

// Create a new connector
pub async fn create_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: &MqttCreateConnectorRequest,
) -> Result<MqttCreateConnectorReply, MqttBrokerError> {
    let connector_type = parse_mqtt_connector_type(request.connector_type());
    connector_config_validator(&connector_type, &request.config)?;

    let config = broker_mqtt_conf();
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

    Ok(MqttCreateConnectorReply {})
}
// Update an existing connector
pub async fn update_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: &MqttUpdateConnectorRequest,
) -> Result<MqttUpdateConnectorReply, MqttBrokerError> {
    let connector = serde_json::from_slice::<MQTTConnector>(&request.connector)
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    connector_config_validator(&connector.connector_type, &connector.config)?;

    let storage = ConnectorStorage::new(client_pool.clone());
    storage
        .update_connector(connector)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(MqttUpdateConnectorReply {})
}

// Delete an existing connector
pub async fn delete_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: &MqttDeleteConnectorRequest,
) -> Result<MqttDeleteConnectorReply, MqttBrokerError> {
    let config = broker_mqtt_conf();
    let storage = ConnectorStorage::new(client_pool.clone());

    storage
        .delete_connector(&config.cluster_name, &request.connector_name)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(MqttDeleteConnectorReply {})
}

fn connector_config_validator(
    connector_type: &ConnectorType,
    config: &str,
) -> Result<(), MqttBrokerError> {
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

fn parse_mqtt_connector_type(connector_type: MqttConnectorType) -> ConnectorType {
    match connector_type {
        MqttConnectorType::File => ConnectorType::LocalFile,
        MqttConnectorType::Kafka => ConnectorType::Kafka,
    }
}
