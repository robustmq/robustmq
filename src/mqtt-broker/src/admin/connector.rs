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
use common_base::config::broker_mqtt::broker_mqtt_conf;
use common_base::tools::now_second;
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
use tonic::{Request, Response, Status};

pub async fn list_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: Request<MqttListConnectorRequest>,
) -> Result<Response<MqttListConnectorReply>, Status> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();
    let request = ListConnectorRequest {
        cluster_name: config.cluster_name.clone(),
        connector_name: req.connector_name.clone(),
    };

    let connectors = placement_list_connector(client_pool, &config.placement_center, request)
        .await?
        .connectors;

    Ok(Response::new(MqttListConnectorReply { connectors }))
}
pub async fn create_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: Request<MqttCreateConnectorRequest>,
) -> Result<Response<MqttCreateConnectorReply>, Status> {
    let req = request.into_inner();
    let connector_type = parse_mqtt_connector_type(req.connector_type());
    if let Err(e) = connector_config_validator(&connector_type, &req.config) {
        return Err(Status::cancelled(e.to_string()));
    };

    let config = broker_mqtt_conf();
    let storage = ConnectorStorage::new(client_pool.clone());
    let connector = MQTTConnector {
        cluster_name: config.cluster_name.clone(),
        connector_name: req.connector_name.clone(),
        connector_type: parse_mqtt_connector_type(req.connector_type()),
        config: req.config.clone(),
        topic_id: req.topic_id.clone(),
        status: MQTTStatus::Idle,
        broker_id: None,
        create_time: now_second(),
        update_time: now_second(),
    };
    if let Err(e) = storage.create_connector(connector).await {
        return Err(Status::cancelled(e.to_string()));
    };
    Ok(Response::new(MqttCreateConnectorReply::default()))
}
pub async fn update_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: Request<MqttUpdateConnectorRequest>,
) -> Result<Response<MqttUpdateConnectorReply>, Status> {
    let req = request.into_inner();
    let connector = match serde_json::from_slice::<MQTTConnector>(&req.connector) {
        Ok(connector) => connector,
        Err(e) => return Err(Status::cancelled(e.to_string())),
    };
    if let Err(e) = connector_config_validator(&connector.connector_type, &connector.config) {
        return Err(Status::cancelled(e.to_string()));
    };
    let storage = ConnectorStorage::new(client_pool.clone());
    if let Err(e) = storage.update_connector(connector).await {
        return Err(Status::cancelled(e.to_string()));
    };
    Ok(Response::new(MqttUpdateConnectorReply::default()))
}

pub async fn delete_connector_by_req(
    client_pool: &Arc<ClientPool>,
    request: Request<MqttDeleteConnectorRequest>,
) -> Result<Response<MqttDeleteConnectorReply>, Status> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();
    let storage = ConnectorStorage::new(client_pool.clone());
    if let Err(e) = storage
        .delete_connector(&config.cluster_name, &req.connector_name)
        .await
    {
        return Err(Status::cancelled(e.to_string()));
    };
    Ok(Response::new(MqttDeleteConnectorReply::default()))
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
