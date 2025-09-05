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
    request::AclListReq,
    response::{AclListRow, PageReplyData},
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::{extract::State, Json};
use common_base::http_response::{error_response, success_response};
use mqtt_broker::security::AuthDriver;
use std::sync::Arc;

pub async fn acl_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<AclListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.page_num,
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
    let data = match auth_driver.read_all_acl().await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    let mut acls_list = Vec::new();
    for acl in data {
        acls_list.push(AclListRow {
            resource_type: acl.resource_type.to_string(),
            resource_name: acl.resource_name.to_string(),
            topic: acl.topic.to_string(),
            ip: acl.ip.to_string(),
            action: acl.action.to_string(),
            permission: acl.permission.to_string(),
        });
    }

    let filtered = apply_filters(acls_list, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for AclListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "resource_type" => Some(self.resource_type.clone()),
            "resource_name" => Some(self.resource_type.clone()),
            "topic" => Some(self.resource_type.clone()),
            "ip" => Some(self.resource_type.clone()),
            _ => None,
        }
    }
}
