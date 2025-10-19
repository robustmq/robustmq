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
    tool::{
        query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
        PageReplyData,
    },
};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SystemAlarmListReq {
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SystemAlarmListRow {
    pub name: String,
    pub message: String,
    pub create_time: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct FlappingDetectListRaw {
    pub client_id: String,
    pub before_last_windows_connections: u64,
    pub first_request_time: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct BanLogListRaw {
    pub ban_type: String,
    pub resource_name: String,
    pub ban_source: String,
    pub end_time: String,
    pub create_time: String,
}

use common_base::{
    http_response::{error_response, success_response},
    utils::time_util::timestamp_to_local_datetime,
};
use mqtt_broker::storage::local::LocalStorage;
use std::sync::Arc;

pub async fn system_alarm_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SystemAlarmListReq>,
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

    let log_storage = LocalStorage::new(state.rocksdb_engine_handler.clone());
    let data_list = match log_storage.list_system_event().await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    let results = data_list
        .iter()
        .map(|entry| SystemAlarmListRow {
            name: entry.name.clone(),
            message: entry.message.clone(),
            create_time: entry.create_time,
        })
        .collect();

    let filtered = apply_filters(results, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for SystemAlarmListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "name" => Some(self.name.clone()),
            "message" => Some(self.message.clone()),
            _ => None,
        }
    }
}

pub async fn flapping_detect_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SystemAlarmListReq>,
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

    let results = state
        .mqtt_context
        .cache_manager
        .acl_metadata
        .flapping_detect_map
        .iter()
        .map(|entry| {
            let flapping_detect = entry.value();
            FlappingDetectListRaw {
                client_id: flapping_detect.client_id.clone(),
                before_last_windows_connections: flapping_detect.before_last_window_connections,
                first_request_time: flapping_detect.first_request_time,
            }
        })
        .collect();

    let filtered = apply_filters(results, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for FlappingDetectListRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "client_id" => Some(self.client_id.clone()),
            _ => None,
        }
    }
}

pub async fn ban_log_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SystemAlarmListReq>,
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

    let log_storage = LocalStorage::new(state.rocksdb_engine_handler.clone());
    let data_list = match log_storage.list_ban_log().await {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };
    let results = data_list
        .iter()
        .map(|entry| BanLogListRaw {
            ban_source: entry.ban_source.clone(),
            ban_type: entry.ban_type.clone(),
            resource_name: entry.resource_name.clone(),
            end_time: timestamp_to_local_datetime(entry.end_time as i64),
            create_time: timestamp_to_local_datetime(entry.create_time as i64),
        })
        .collect();

    let filtered = apply_filters(results, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for BanLogListRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "resource_name" => Some(self.resource_name.clone()),
            _ => None,
        }
    }
}
