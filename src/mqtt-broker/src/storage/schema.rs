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

use common_base::config::broker_mqtt::broker_mqtt_conf;
use grpc_clients::{
    placement::inner::call::{
        bind_schema, create_schema, delete_schema, list_schema, un_bind_schema, update_schema,
    },
    pool::ClientPool,
};
use metadata_struct::schema::{SchemaData, SchemaType};
use protocol::{
    broker_mqtt::broker_mqtt_admin::{
        MqttBindSchemaRequest, MqttCreateSchemaRequest, MqttDeleteSchemaRequest,
        MqttListBindSchemaRequest, MqttListSchemaRequest, MqttUnbindSchemaRequest,
        MqttUpdateSchemaRequest,
    },
    placement_center::placement_center_inner::{
        BindSchemaRequest, CreateSchemaRequest, DeleteSchemaRequest, ListSchemaRequest,
        UnBindSchemaRequest, UpdateSchemaRequest,
    },
};
use schema_register::schema::SchemaRegisterManager;

use crate::handler::error::MqttBrokerError;

pub async fn list_schema_by_req(
    client_pool: &Arc<ClientPool>,
    schema_manager: &Arc<SchemaRegisterManager>,
    req: &MqttListSchemaRequest,
) -> Result<Vec<Vec<u8>>, MqttBrokerError> {
    let data_list = if !req.schema_name.is_empty() {
        if let Some(data) = schema_manager.get_schema(req.schema_name.as_str()) {
            vec![data]
        } else {
            // request from placement center
            let config = broker_mqtt_conf();
            let request = ListSchemaRequest {
                cluster_name: config.cluster_name.clone(),
                schema_name: req.schema_name.clone(),
            };

            let schemas = list_schema(client_pool, &config.placement_center, request)
                .await?
                .schemas;
            return Ok(schemas);
        }
    } else {
        schema_manager.get_all_schema()
    };
    let mut results = Vec::new();
    for data in data_list {
        results.push(data.encode());
    }

    Ok(results)
}

pub async fn create_schema_by_req(
    client_pool: &Arc<ClientPool>,
    req: &MqttCreateSchemaRequest,
) -> Result<(), MqttBrokerError> {
    let config = broker_mqtt_conf();

    let schema_type = match req.schema_type.as_str() {
        "" | "json" => SchemaType::JSON,
        "avro" => SchemaType::AVRO,
        "protobuf" => SchemaType::PROTOBUF,
        _ => return Err(MqttBrokerError::InvalidSchemaType(req.schema_type.clone())),
    };

    let schema_data = SchemaData {
        cluster_name: config.cluster_name.clone(),
        name: req.schema_name.clone(),
        schema_type,
        schema: req.schema.clone(),
        desc: req.desc.clone(),
    };

    let request = CreateSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
        schema: serde_json::to_vec(&schema_data).unwrap(),
    };

    create_schema(client_pool, &config.placement_center, request).await?;
    Ok(())
}

pub async fn update_schema_by_req(
    client_pool: &Arc<ClientPool>,
    req: &MqttUpdateSchemaRequest,
) -> Result<(), MqttBrokerError> {
    let config = broker_mqtt_conf();

    let schema_type = match req.schema_type.as_str() {
        "" | "json" => SchemaType::JSON,
        "avro" => SchemaType::AVRO,
        "protobuf" => SchemaType::PROTOBUF,
        _ => return Err(MqttBrokerError::InvalidSchemaType(req.schema_type.clone())),
    };

    let schema_data = SchemaData {
        cluster_name: config.cluster_name.clone(),
        name: req.schema_name.clone(),
        schema_type,
        schema: req.schema.clone(),
        desc: req.desc.clone(),
    };

    let request = UpdateSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
        schema: serde_json::to_vec(&schema_data).unwrap(),
    };

    update_schema(client_pool, &config.placement_center, request).await?;
    Ok(())
}

pub async fn delete_schema_by_req(
    client_pool: &Arc<ClientPool>,
    req: &MqttDeleteSchemaRequest,
) -> Result<(), MqttBrokerError> {
    let config = broker_mqtt_conf();
    let request = DeleteSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
    };

    delete_schema(client_pool, &config.placement_center, request).await?;
    Ok(())
}

pub async fn list_bind_schema_by_req(
    schema_manager: &Arc<SchemaRegisterManager>,
    req: &MqttListBindSchemaRequest,
) -> Result<Vec<Vec<u8>>, MqttBrokerError> {
    let data_list = schema_manager.get_schema_resource(req.resource_name.as_str());

    let mut results = Vec::new();
    for data in data_list {
        results.push(data.encode());
    }

    Ok(results)
}

pub async fn bind_schema_by_req(
    client_pool: &Arc<ClientPool>,
    req: &MqttBindSchemaRequest,
) -> Result<(), MqttBrokerError> {
    let config = broker_mqtt_conf();
    let request = BindSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
        resource_name: req.resource_name.clone(),
    };

    bind_schema(client_pool, &config.placement_center, request).await?;
    Ok(())
}

pub async fn unbind_schema_by_req(
    client_pool: &Arc<ClientPool>,
    req: &MqttUnbindSchemaRequest,
) -> Result<(), MqttBrokerError> {
    let config = broker_mqtt_conf();
    let request = UnBindSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
        resource_name: req.resource_name.clone(),
    };

    un_bind_schema(client_pool, &config.placement_center, request).await?;
    Ok(())
}
