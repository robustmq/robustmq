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
    request::mqtt::SystemAlarmListReq,
    response::{
        mqtt::{FlappingDetectListRaw, SystemAlarmListRow},
        PageReplyData,
    },
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::{extract::State, Json};
use common_base::{http_response::success_response, utils::time_util::timestamp_to_local_datetime};
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

    let results = state
        .mqtt_context
        .cache_manager
        .alarm_events
        .iter()
        .map(|entry| {
            let system_alarm_message = entry.value();
            SystemAlarmListRow {
                name: system_alarm_message.name.clone(),
                message: system_alarm_message.message.clone(),
                activate_at: timestamp_to_local_datetime(system_alarm_message.activate_at),
                activated: system_alarm_message.activated,
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
