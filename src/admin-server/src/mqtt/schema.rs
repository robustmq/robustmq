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

use axum::{extract::State, Json};
use common_base::http_response::{error_response, success_response};
use common_config::broker::broker_config;
use metadata_struct::schema::{SchemaData, SchemaResourceBind, SchemaType};
use mqtt_broker::{handler::error::MqttBrokerError, storage::schema::SchemaStorage};
use std::sync::Arc;

use crate::{
    extractor::ValidatedJson,
    request::mqtt::{
        CreateSchemaBindReq, CreateSchemaReq, DeleteSchemaBindReq, DeleteSchemaReq,
        SchemaBindListReq, SchemaListReq,
    },
    response::{
        mqtt::{SchemaBindListRow, SchemaListRow},
        PageReplyData,
    },
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};

pub async fn schema_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SchemaListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        params.filter_field,
        params.filter_values,
        params.exact_match,
    );

    let mut schemas = Vec::new();
    for schema in state.mqtt_context.schema_manager.get_all_schema() {
        schemas.push(SchemaListRow {
            name: schema.name.clone(),
            schema_type: schema.schema_type.to_string(),
            desc: schema.desc.clone(),
            schema: schema.schema.clone(),
        });
    }

    let filtered = apply_filters(schemas, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for SchemaListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "name" => Some(self.name.clone()),
            "schema_type" => Some(self.schema_type.clone()),
            _ => None,
        }
    }
}

pub async fn schema_create(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<CreateSchemaReq>,
) -> String {
    if let Err(e) = schema_create_inner(state, params).await {
        return error_response(e.to_string());
    }
    success_response("success")
}

pub async fn schema_create_inner(
    state: Arc<HttpState>,
    req: CreateSchemaReq,
) -> Result<(), MqttBrokerError> {
    let schema_type = match req.schema_type.as_str() {
        "json" => SchemaType::JSON,
        "avro" => SchemaType::AVRO,
        "protobuf" => SchemaType::PROTOBUF,
        _ => return Err(MqttBrokerError::InvalidSchemaType(req.schema_type.clone())),
    };

    let schema_data = SchemaData {
        cluster_name: state.broker_cache.cluster_name.clone(),
        name: req.schema_name.clone(),
        schema_type,
        schema: req.schema.clone(),
        desc: req.desc.clone(),
    };

    let schema_storage = SchemaStorage::new(state.client_pool.clone());
    schema_storage.create(schema_data.clone()).await?;

    state.mqtt_context.schema_manager.add_schema(schema_data);
    Ok(())
}

pub async fn schema_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<DeleteSchemaReq>,
) -> String {
    let schema_storage = SchemaStorage::new(state.client_pool.clone());
    if let Err(e) = schema_storage.delete(params.schema_name.clone()).await {
        return error_response(e.to_string());
    }
    state
        .mqtt_context
        .schema_manager
        .remove_schema(&params.schema_name);
    success_response("success")
}

pub async fn schema_bind_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SchemaBindListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        params.filter_field,
        params.filter_values,
        params.exact_match,
    );

    let mut schema_bind_list = Vec::new();
    if let Some(resource_name) = params.resource_name {
        let results = state
            .mqtt_context
            .schema_manager
            .get_bind_schema_by_resource(&resource_name);

        schema_bind_list.push(SchemaBindListRow {
            data_type: "resource".to_string(),
            data: results.iter().map(|raw| raw.name.clone()).collect(),
        });
    }

    if let Some(schema_name) = params.schema_name {
        let results = state
            .mqtt_context
            .schema_manager
            .get_bind_resource_by_schema(&schema_name);
        schema_bind_list.push(SchemaBindListRow {
            data_type: "schema".to_string(),
            data: results,
        });
    }

    let filtered = apply_filters(schema_bind_list, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for SchemaBindListRow {
    fn get_field_str(&self, _: &str) -> Option<String> {
        None
    }
}

pub async fn schema_bind_create(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<CreateSchemaBindReq>,
) -> String {
    let schema_storage = SchemaStorage::new(state.client_pool.clone());
    if let Err(e) = schema_storage
        .create_bind(&params.schema_name, &params.resource_name)
        .await
    {
        return error_response(e.to_string());
    }

    let config = broker_config();
    let bind = SchemaResourceBind {
        cluster_name: config.cluster_name.clone(),
        schema_name: params.schema_name,
        resource_name: params.resource_name,
    };
    state.mqtt_context.schema_manager.add_bind(&bind);
    success_response("success")
}

pub async fn schema_bind_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<DeleteSchemaBindReq>,
) -> String {
    let schema_storage = SchemaStorage::new(state.client_pool.clone());
    if let Err(e) = schema_storage
        .delete_bind(&params.schema_name, &params.resource_name)
        .await
    {
        return error_response(e.to_string());
    }

    let config = broker_config();
    let bind = SchemaResourceBind {
        cluster_name: config.cluster_name.clone(),
        schema_name: params.schema_name,
        resource_name: params.resource_name,
    };
    state.mqtt_context.schema_manager.remove_bind(&bind);
    success_response("success")
}
