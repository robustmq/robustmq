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
    request::SubscribeListReq,
    response::{PageReplyData, SubscribeListRow},
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::extract::{Query, State};
use common_base::{http_response::success_response, utils::time_util::timestamp_to_local_datetime};
use metadata_struct::mqtt::subscribe_data::is_mqtt_share_subscribe;
use std::sync::Arc;

pub async fn subscribe_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<SubscribeListReq>,
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

    let mut subscribes = Vec::new();
    for (_, sub) in state.mqtt_context.subscribe_manager.subscribe_list.clone() {
        subscribes.push(SubscribeListRow {
            broker_id: sub.broker_id,
            client_id: sub.client_id,
            create_time: timestamp_to_local_datetime(sub.create_time as i64),
            no_local: if sub.filter.nolocal { 1 } else { 0 },
            path: sub.path.clone(),
            pk_id: sub.pkid as u32,
            preserve_retain: if sub.filter.preserve_retain { 1 } else { 0 },
            properties: serde_json::to_string(&sub.subscribe_properties).unwrap(),
            protocol: format!("{:?}", sub.protocol),
            qos: format!("{:?}", sub.filter.qos),
            retain_handling: format!("{:?}", sub.filter.retain_handling),
            is_share_sub: is_mqtt_share_subscribe(&sub.path),
        });
    }
    let filtered = apply_filters(subscribes, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for SubscribeListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "client_id" => Some(self.client_id.clone()),
            _ => None,
        }
    }
}
