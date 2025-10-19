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
    extractor::ValidatedJson,
    request::mqtt::{BlackListListReq, CreateBlackListReq, DeleteBlackListReq},
    response::{mqtt::BlackListListRow, PageReplyData},
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::{extract::State, Json};
use common_base::{
    enum_type::mqtt::acl::mqtt_acl_blacklist_type::get_blacklist_type_by_str,
    http_response::{error_response, success_response},
    utils::time_util::timestamp_to_local_datetime,
};
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;
use mqtt_broker::security::AuthDriver;
use std::sync::Arc;

pub async fn blacklist_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<BlackListListReq>,
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
    let data = match auth_driver.read_all_blacklist().await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    let mut blacklists = Vec::new();
    for blacklist in data {
        blacklists.push(BlackListListRow {
            blacklist_type: blacklist.blacklist_type.to_string(),
            resource_name: blacklist.resource_name,
            end_time: timestamp_to_local_datetime(blacklist.end_time as i64),
            desc: blacklist.desc,
        });
    }

    let filtered = apply_filters(blacklists, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for BlackListListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "blacklist_type" => Some(self.blacklist_type.clone()),
            "resource_name" => Some(self.resource_name.clone()),
            _ => None,
        }
    }
}

pub async fn blacklist_create(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<CreateBlackListReq>,
) -> String {
    let blacklist_type = match get_blacklist_type_by_str(&params.blacklist_type) {
        Ok(blacklist_type) => blacklist_type,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    let mqtt_blacklist = MqttAclBlackList {
        blacklist_type,
        resource_name: params.resource_name.clone(),
        end_time: params.end_time,
        desc: params.desc.clone(),
    };
    let auth_driver = AuthDriver::new(
        state.mqtt_context.cache_manager.clone(),
        state.client_pool.clone(),
    );

    match auth_driver.save_blacklist(mqtt_blacklist).await {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}

pub async fn blacklist_delete(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<DeleteBlackListReq>,
) -> String {
    let blacklist_type = match get_blacklist_type_by_str(&params.blacklist_type) {
        Ok(blacklist_type) => blacklist_type,
        Err(e) => {
            return error_response(e.to_string());
        }
    };
    let mqtt_blacklist = MqttAclBlackList {
        blacklist_type,
        resource_name: params.resource_name.clone(),
        end_time: 0,
        desc: "".to_string(),
    };

    let auth_driver = AuthDriver::new(
        state.mqtt_context.cache_manager.clone(),
        state.client_pool.clone(),
    );
    match auth_driver.delete_blacklist(mqtt_blacklist).await {
        Ok(_) => success_response("success"),
        Err(e) => error_response(e.to_string()),
    }
}
