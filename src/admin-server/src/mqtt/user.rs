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
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct UserListReq {
    pub tenant: Option<String>,
    pub user_name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct CreateUserReq {
    #[validate(length(min = 1, max = 64, message = "Tenant length must be between 1-64"))]
    pub tenant: String,

    #[validate(length(min = 1, max = 64, message = "Username length must be between 1-64"))]
    pub username: String,

    #[validate(length(min = 1, max = 128, message = "Password length must be between 1-128"))]
    pub password: String,

    pub is_superuser: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Validate)]
pub struct DeleteUserReq {
    #[validate(length(min = 1, max = 64, message = "Tenant length must be between 1-64"))]
    pub tenant: String,

    #[validate(length(min = 1, max = 64, message = "Username length must be between 1-64"))]
    pub username: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserListRow {
    pub tenant: String,
    pub username: String,
    pub is_superuser: bool,
    pub create_time: u64,
}

use common_base::{
    http_response::{error_response, success_response},
    tools::now_second,
};
use metadata_struct::mqtt::user::MqttUser;
use mqtt_broker::storage::user::UserStorage;
use std::sync::Arc;

pub async fn user_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<UserListReq>,
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

    let mut users = Vec::new();
    for tenant_entry in state.mqtt_context.cache_manager.user_info.iter() {
        if let Some(ref t) = params.tenant {
            if tenant_entry.key() != t {
                continue;
            }
        }
        for ele in tenant_entry.value().iter() {
            if let Some(ref name) = params.user_name {
                if !ele.value().username.contains(name.as_str()) {
                    continue;
                }
            }
            let user_raw = UserListRow {
                tenant: tenant_entry.key().clone(),
                username: ele.value().username.clone(),
                is_superuser: ele.value().is_superuser,
                create_time: ele.value().create_time,
            };
            users.push(user_raw);
        }
    }

    let filtered = apply_filters(users, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for UserListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "tenant" => Some(self.tenant.clone()),
            "username" => Some(self.username.clone()),
            _ => None,
        }
    }
}

pub async fn user_create(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<CreateUserReq>,
) -> String {
    let user_info = MqttUser {
        tenant: params.tenant.clone(),
        username: params.username.clone(),
        password: params.password.clone(),
        salt: None,
        is_superuser: params.is_superuser,
        create_time: now_second(),
    };

    let user_storage = UserStorage::new(state.client_pool.clone());
    match user_storage.save_user(user_info).await {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}

pub async fn user_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<DeleteUserReq>,
) -> String {
    let user_storage = UserStorage::new(state.client_pool.clone());
    match user_storage
        .delete_user(params.tenant.clone(), params.username.clone())
        .await
    {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}
