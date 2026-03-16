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
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TenantListReq {
    pub tenant_name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct CreateTenantReq {
    #[validate(length(
        min = 1,
        max = 128,
        message = "Tenant name length must be between 1-128"
    ))]
    pub tenant_name: String,

    #[validate(length(max = 500, message = "Description length cannot exceed 500"))]
    pub desc: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct DeleteTenantReq {
    #[validate(length(
        min = 1,
        max = 128,
        message = "Tenant name length must be between 1-128"
    ))]
    pub tenant_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TenantListRow {
    pub tenant_name: String,
    pub desc: String,
    pub create_time: u64,
}

impl Queryable for TenantListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "tenant_name" => Some(self.tenant_name.clone()),
            _ => None,
        }
    }
}

pub async fn tenant_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<TenantListReq>,
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

    let mut tenants: Vec<TenantListRow> = state
        .broker_cache
        .tenant_list
        .iter()
        .filter(|entry| {
            if let Some(name) = &params.tenant_name {
                entry.key() == name
            } else {
                true
            }
        })
        .map(|entry| TenantListRow {
            tenant_name: entry.tenant_name.clone(),
            desc: entry.desc.clone(),
            create_time: entry.create_time,
        })
        .collect();

    tenants.sort_by_key(|t| t.tenant_name.clone());

    let filtered = apply_filters(tenants, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

pub async fn tenant_create(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<CreateTenantReq>,
) -> String {
    let storage = TenantStorage::new(state.client_pool.clone());
    match storage
        .create(
            &params.tenant_name,
            params.desc.as_deref().unwrap_or_default(),
        )
        .await
    {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}

pub async fn tenant_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<DeleteTenantReq>,
) -> String {
    let storage = TenantStorage::new(state.client_pool.clone());
    match storage.delete(&params.tenant_name).await {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}
