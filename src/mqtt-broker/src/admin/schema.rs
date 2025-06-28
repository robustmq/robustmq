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

use common_config::mqtt::broker_mqtt_conf;
use grpc_clients::{
    placement::inner::call::{
        bind_schema, create_schema, delete_schema, list_bind_schema, list_schema, un_bind_schema,
        update_schema,
    },
    pool::ClientPool,
};
use metadata_struct::schema::{SchemaData, SchemaType};
use protocol::{
    broker_mqtt::broker_mqtt_admin::{
        MqttBindSchemaReply, MqttBindSchemaRequest, MqttCreateSchemaReply, MqttCreateSchemaRequest,
        MqttDeleteSchemaReply, MqttDeleteSchemaRequest, MqttListBindSchemaReply,
        MqttListBindSchemaRequest, MqttListSchemaReply, MqttListSchemaRequest,
        MqttUnbindSchemaReply, MqttUnbindSchemaRequest, MqttUpdateSchemaReply,
        MqttUpdateSchemaRequest,
    },
    placement_center::placement_center_inner::{
        BindSchemaRequest, CreateSchemaRequest, DeleteSchemaRequest, ListBindSchemaRequest,
        ListSchemaRequest, UnBindSchemaRequest, UpdateSchemaRequest,
    },
};
use std::sync::Arc;
// List schemas by request
pub async fn list_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &MqttListSchemaRequest,
) -> Result<MqttListSchemaReply, MqttBrokerError> {
    let config = broker_mqtt_conf();
    let request = ListSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
    };

    let schemas = list_schema(client_pool, &config.placement_center, request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?
        .schemas;

    Ok(MqttListSchemaReply { schemas })
}

// Create a new schema
pub async fn create_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &MqttCreateSchemaRequest,
) -> Result<MqttCreateSchemaReply, MqttBrokerError> {
    let config = broker_mqtt_conf();

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

    let request = CreateSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
        schema: serde_json::to_vec(&schema_data)
            .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?,
    };

    create_schema(client_pool, &config.placement_center, request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(MqttCreateSchemaReply {})
}

// Update an existing schema
pub async fn update_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &MqttUpdateSchemaRequest,
) -> Result<MqttUpdateSchemaReply, MqttBrokerError> {
    let config = broker_mqtt_conf();

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

    let request = UpdateSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
        schema: serde_json::to_vec(&schema_data)
            .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?,
    };

    update_schema(client_pool, &config.placement_center, request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(MqttUpdateSchemaReply {})
}

// Delete an existing schema
pub async fn delete_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &MqttDeleteSchemaRequest,
) -> Result<MqttDeleteSchemaReply, MqttBrokerError> {
    let config = broker_mqtt_conf();
    let request = DeleteSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
    };

    delete_schema(client_pool, &config.placement_center, request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(MqttDeleteSchemaReply {})
}

// List schema bindings
pub async fn list_bind_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &MqttListBindSchemaRequest,
) -> Result<MqttListBindSchemaReply, MqttBrokerError> {
    let config = broker_mqtt_conf();
    let request = ListBindSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
        resource_name: request.resource_name.clone(),
    };

    let schema_binds = list_bind_schema(client_pool, &config.placement_center, request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?
        .schema_binds;

    Ok(MqttListBindSchemaReply { schema_binds })
}

// Bind schema to resource
pub async fn bind_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &MqttBindSchemaRequest,
) -> Result<MqttBindSchemaReply, MqttBrokerError> {
    let config = broker_mqtt_conf();
    let request = BindSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
        resource_name: request.resource_name.clone(),
    };

    bind_schema(client_pool, &config.placement_center, request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(MqttBindSchemaReply {})
}

// Unbind schema from resource
pub async fn unbind_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: &MqttUnbindSchemaRequest,
) -> Result<MqttUnbindSchemaReply, MqttBrokerError> {
    let config = broker_mqtt_conf();
    let request = UnBindSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: request.schema_name.clone(),
        resource_name: request.resource_name.clone(),
    };

    un_bind_schema(client_pool, &config.placement_center, request)
        .await
        .map_err(|e| MqttBrokerError::CommonError(e.to_string()))?;

    Ok(MqttUnbindSchemaReply {})
}
