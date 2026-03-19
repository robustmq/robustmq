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

use crate::{
    state::HttpState,
    tool::extractor::ValidatedJson,
    tool::{
        query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
        PageReplyData,
    },
};
use axum::extract::{Query, State};
use broker_core::tenant::TenantStorage;
use common_base::http_response::{error_response, success_response};
use common_config::broker::broker_config;
use metadata_struct::tenant::TenantConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct MqttTenantListReq {
    pub tenant_name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate, Default)]
pub struct CreateMqttTenantReq {
    #[validate(length(
        min = 1,
        max = 128,
        message = "Tenant name length must be between 1-128"
    ))]
    pub tenant_name: String,

    #[validate(length(max = 500, message = "Description length cannot exceed 500"))]
    pub desc: Option<String>,

    pub max_connections_per_node: Option<u64>,
    pub max_create_connection_rate_per_second: Option<u32>,
    pub max_topics: Option<u64>,
    pub max_sessions: Option<u64>,
    pub max_publish_rate: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct DeleteMqttTenantReq {
    #[validate(length(
        min = 1,
        max = 128,
        message = "Tenant name length must be between 1-128"
    ))]
    pub tenant_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MqttTenantListRow {
    pub tenant_name: String,
    pub desc: String,
    pub create_time: u64,
    pub max_connections_per_node: u64,
    pub max_create_connection_rate_per_second: u32,
    pub max_topics: u64,
    pub max_sessions: u64,
    pub max_publish_rate: u32,
}

impl Queryable for MqttTenantListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "tenant_name" => Some(self.tenant_name.clone()),
            _ => None,
        }
    }
}

fn tenant_to_row(t: &metadata_struct::tenant::Tenant) -> MqttTenantListRow {
    MqttTenantListRow {
        tenant_name: t.tenant_name.clone(),
        desc: t.desc.clone(),
        create_time: t.create_time,
        max_connections_per_node: t.config.max_connections_per_node,
        max_create_connection_rate_per_second: t.config.max_create_connection_rate_per_second,
        max_topics: t.config.max_topics,
        max_sessions: t.config.max_sessions,
        max_publish_rate: t.config.max_publish_rate,
    }
}

pub async fn mqtt_tenant_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<MqttTenantListReq>,
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

    let tenants: Vec<MqttTenantListRow> = if let Some(ref name) = params.tenant_name {
        state
            .broker_cache
            .tenant_list
            .get(name)
            .map(|t| vec![tenant_to_row(&t)])
            .unwrap_or_default()
    } else {
        state
            .broker_cache
            .tenant_list
            .iter()
            .map(|entry| tenant_to_row(&entry))
            .collect()
    };

    let filtered = apply_filters(tenants, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

pub async fn mqtt_tenant_create(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<CreateMqttTenantReq>,
) -> String {
    let tenant_limit = broker_config().mqtt_limit.tenant.clone();
    let config = TenantConfig {
        max_connections_per_node: params
            .max_connections_per_node
            .unwrap_or(tenant_limit.max_connections_per_node),
        max_create_connection_rate_per_second: params
            .max_create_connection_rate_per_second
            .unwrap_or(tenant_limit.max_connection_rate),
        max_topics: params.max_topics.unwrap_or(tenant_limit.max_topics),
        max_sessions: params.max_sessions.unwrap_or(tenant_limit.max_sessions),
        max_publish_rate: params
            .max_publish_rate
            .unwrap_or(tenant_limit.max_publish_rate),
    };
    let storage = TenantStorage::new(state.client_pool.clone());
    match storage
        .create(
            &params.tenant_name,
            params.desc.as_deref().unwrap_or_default(),
            config,
        )
        .await
    {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}

pub async fn mqtt_tenant_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<DeleteMqttTenantReq>,
) -> String {
    let storage = TenantStorage::new(state.client_pool.clone());
    match storage.delete(&params.tenant_name).await {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}
