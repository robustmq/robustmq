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
    request::mqtt::{CreateUserReq, DeleteUserReq, UserListReq},
    response::{mqtt::UserListRow, PageReplyData},
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::{extract::State, Json};
use common_base::http_response::{error_response, success_response};
use metadata_struct::mqtt::user::MqttUser;
use mqtt_broker::security::AuthDriver;
use std::sync::Arc;

pub async fn user_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<UserListReq>,
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

    let auth_driver = AuthDriver::new(
        state.mqtt_context.cache_manager.clone(),
        state.client_pool.clone(),
    );

    let data = match auth_driver.read_all_user().await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    let mut users = Vec::new();

    for ele in data {
        let user_raw = UserListRow {
            username: ele.1.username,
            is_superuser: ele.1.is_superuser,
        };
        users.push(user_raw);
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
            "username" => Some(self.username.clone()),
            _ => None,
        }
    }
}

pub async fn user_create(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<CreateUserReq>,
) -> String {
    let mqtt_user = MqttUser {
        username: params.username.clone(),
        password: params.password.clone(),
        is_superuser: params.is_superuser,
    };

    let auth_driver = AuthDriver::new(
        state.mqtt_context.cache_manager.clone(),
        state.client_pool.clone(),
    );
    match auth_driver.save_user(mqtt_user).await {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}

pub async fn user_delete(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<DeleteUserReq>,
) -> String {
    let auth_driver = AuthDriver::new(
        state.mqtt_context.cache_manager.clone(),
        state.client_pool.clone(),
    );

    match auth_driver.delete_user(params.username.clone()).await {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}
