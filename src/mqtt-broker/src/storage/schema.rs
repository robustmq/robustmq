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
use common_base::config::broker_mqtt::broker_mqtt_conf;
use grpc_clients::{
    placement::inner::call::{
        bind_schema, create_schema, delete_schema, list_bind_schema, list_schema, un_bind_schema,
        update_schema,
    },
    pool::ClientPool,
};
use metadata_struct::schema::{SchemaData, SchemaType};
use protocol::broker_mqtt::broker_mqtt_admin::{
    MqttBindSchemaReply, MqttCreateSchemaReply, MqttDeleteSchemaReply, MqttListBindSchemaReply,
    MqttListSchemaReply, MqttUnbindSchemaReply, MqttUpdateSchemaReply,
};
use protocol::{
    broker_mqtt::broker_mqtt_admin::{
        MqttBindSchemaRequest, MqttCreateSchemaRequest, MqttDeleteSchemaRequest,
        MqttListBindSchemaRequest, MqttListSchemaRequest, MqttUnbindSchemaRequest,
        MqttUpdateSchemaRequest,
    },
    placement_center::placement_center_inner::{
        BindSchemaRequest, CreateSchemaRequest, DeleteSchemaRequest, ListBindSchemaRequest,
        ListSchemaRequest, UnBindSchemaRequest, UpdateSchemaRequest,
    },
};
use std::sync::Arc;
use tonic::{Request, Response, Status};
pub async fn list_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: Request<MqttListSchemaRequest>,
) -> Result<Response<MqttListSchemaReply>, Status> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();
    let request = ListSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
    };

    let schemas = list_schema(client_pool, &config.placement_center, request)
        .await?
        .schemas;
    Ok(Response::new(MqttListSchemaReply { schemas }))
}

pub async fn create_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: Request<MqttCreateSchemaRequest>,
) -> Result<Response<MqttCreateSchemaReply>, Status> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();

    let schema_type = match req.schema_type.as_str() {
        "" | "json" => SchemaType::JSON,
        "avro" => SchemaType::AVRO,
        "protobuf" => SchemaType::PROTOBUF,
        _ => {
            return Err(Status::cancelled(
                MqttBrokerError::InvalidSchemaType(req.schema_type.clone()).to_string(),
            ))
        }
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
    Ok(Response::new(MqttCreateSchemaReply::default()))
}

pub async fn update_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: Request<MqttUpdateSchemaRequest>,
) -> Result<Response<MqttUpdateSchemaReply>, Status> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();

    let schema_type = match req.schema_type.as_str() {
        "" | "json" => SchemaType::JSON,
        "avro" => SchemaType::AVRO,
        "protobuf" => SchemaType::PROTOBUF,
        _ => {
            return Err(Status::cancelled(
                MqttBrokerError::InvalidSchemaType(req.schema_type.clone()).to_string(),
            ))
        }
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
    Ok(Response::new(MqttUpdateSchemaReply::default()))
}

pub async fn delete_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: Request<MqttDeleteSchemaRequest>,
) -> Result<Response<MqttDeleteSchemaReply>, Status> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();
    let request = DeleteSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
    };

    delete_schema(client_pool, &config.placement_center, request).await?;
    Ok(Response::new(MqttDeleteSchemaReply::default()))
}
pub async fn list_bind_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: Request<MqttListBindSchemaRequest>,
) -> Result<Response<MqttListBindSchemaReply>, Status> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();
    let request = ListBindSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
        resource_name: req.resource_name.clone(),
    };

    let schemas = list_bind_schema(client_pool, &config.placement_center, request)
        .await?
        .schema_binds;

    Ok(Response::new(MqttListBindSchemaReply {
        schema_binds: schemas,
    }))
}
pub async fn bind_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: Request<MqttBindSchemaRequest>,
) -> Result<Response<MqttBindSchemaReply>, Status> {
    let req = request.into_inner();

    let config = broker_mqtt_conf();
    let request = BindSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
        resource_name: req.resource_name.clone(),
    };

    bind_schema(client_pool, &config.placement_center, request).await?;

    Ok(Response::new(MqttBindSchemaReply::default()))
}

pub async fn unbind_schema_by_req(
    client_pool: &Arc<ClientPool>,
    request: Request<MqttUnbindSchemaRequest>,
) -> Result<Response<MqttUnbindSchemaReply>, Status> {
    let req = request.into_inner();
    let config = broker_mqtt_conf();
    let request = UnBindSchemaRequest {
        cluster_name: config.cluster_name.clone(),
        schema_name: req.schema_name.clone(),
        resource_name: req.resource_name.clone(),
    };

    un_bind_schema(client_pool, &config.placement_center, request).await?;
    Ok(Response::new(MqttUnbindSchemaReply::default()))
}
