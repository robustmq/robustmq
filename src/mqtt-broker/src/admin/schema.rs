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

use common_config::broker::broker_config;
use grpc_clients::{
    placement::inner::call::{
        bind_schema, create_schema, delete_schema, list_bind_schema, list_schema, un_bind_schema,
        update_schema,
    },
    pool::ClientPool,
};
use metadata_struct::schema::{SchemaData, SchemaType};
use protocol::broker_mqtt::broker_mqtt_admin;
use protocol::placement_center::placement_center_inner;
use std::sync::Arc;

// List schemas by request
pub async fn list_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &broker_mqtt_admin::ListSchemaRequest,
) -> Result<broker_mqtt_admin::ListSchemaReply, MqttBrokerError> {
    let config = broker_config();
    let request = placement_center_inner::ListSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
    };

    let schemas = list_schema(client_pool, &config.get_placement_center_addr(), request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?
        .schemas;

    Ok(broker_mqtt_admin::ListSchemaReply { schemas })
}

// Create a new schema
pub async fn create_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &broker_mqtt_admin::CreateSchemaRequest,
) -> Result<broker_mqtt_admin::CreateSchemaReply, MqttBrokerError> {
    let config = broker_config();

    let schema_type = match request.schema_type.as_str() {
        "" | "json" => SchemaType::JSON,
        "avro" => SchemaType::AVRO,
        "protobuf" => SchemaType::PROTOBUF,
        _ => {
            return Err(MqttBrokerError::InvalidSchemaType(
                request.schema_type.clone(),
            ))
        }
    };

    let schema_data = SchemaData {
        cluster_name: config.cluster_name.clone(),
        name: request.schema_name.clone(),
        schema_type,
        schema: request.schema.clone(),
        desc: request.desc.clone(),
    };

    let request = placement_center_inner::CreateSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
        schema: serde_json::to_vec(&schema_data)
            .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?,
    };

    create_schema(client_pool, &config.get_placement_center_addr(), request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(broker_mqtt_admin::CreateSchemaReply {})
}

// Update an existing schema
pub async fn update_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &broker_mqtt_admin::UpdateSchemaRequest,
) -> Result<broker_mqtt_admin::UpdateSchemaReply, MqttBrokerError> {
    let config = broker_config();

    let schema_type = match request.schema_type.as_str() {
        "" | "json" => SchemaType::JSON,
        "avro" => SchemaType::AVRO,
        "protobuf" => SchemaType::PROTOBUF,
        _ => {
            return Err(MqttBrokerError::InvalidSchemaType(
                request.schema_type.clone(),
            ))
        }
    };

    let schema_data = SchemaData {
        cluster_name: config.cluster_name.clone(),
        name: request.schema_name.clone(),
        schema_type,
        schema: request.schema.clone(),
        desc: request.desc.clone(),
    };

    let request = placement_center_inner::UpdateSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
        schema: serde_json::to_vec(&schema_data)
            .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?,
    };

    update_schema(client_pool, &config.get_placement_center_addr(), request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(broker_mqtt_admin::UpdateSchemaReply {})
}

// Delete an existing schema
pub async fn delete_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &broker_mqtt_admin::DeleteSchemaRequest,
) -> Result<broker_mqtt_admin::DeleteSchemaReply, MqttBrokerError> {
    let config = broker_config();
    let request = placement_center_inner::DeleteSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
    };

    delete_schema(client_pool, &config.get_placement_center_addr(), request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(broker_mqtt_admin::DeleteSchemaReply {})
}

// List schema bindings
pub async fn list_bind_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &broker_mqtt_admin::ListBindSchemaRequest,
) -> Result<broker_mqtt_admin::ListBindSchemaReply, MqttBrokerError> {
    let config = broker_config();
    let request = placement_center_inner::ListBindSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
        resource_name: request.resource_name.clone(),
    };

    let schema_binds = list_bind_schema(client_pool, &config.get_placement_center_addr(), request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?
        .schema_binds;

    Ok(broker_mqtt_admin::ListBindSchemaReply { schema_binds })
}

// Bind schema to resource
pub async fn bind_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &broker_mqtt_admin::BindSchemaRequest,
) -> Result<broker_mqtt_admin::BindSchemaReply, MqttBrokerError> {
    let config = broker_config();
    let request = placement_center_inner::BindSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
        resource_name: request.resource_name.clone(),
    };

    bind_schema(client_pool, &config.get_placement_center_addr(), request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(broker_mqtt_admin::BindSchemaReply {})
}

// Unbind schema from resource
pub async fn unbind_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &broker_mqtt_admin::UnbindSchemaRequest,
) -> Result<broker_mqtt_admin::UnbindSchemaReply, MqttBrokerError> {
    let config = broker_config();
    let request = placement_center_inner::UnBindSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
        resource_name: request.resource_name.clone(),
    };

    un_bind_schema(client_pool, &config.get_placement_center_addr(), request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(broker_mqtt_admin::UnbindSchemaReply {})
}
