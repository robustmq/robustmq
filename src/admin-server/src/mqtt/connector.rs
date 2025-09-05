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
    request::ConnectorListReq,
    response::{ConnectorListRow, PageReplyData},
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::extract::{Query, State};
use common_base::{http_response::success_response, utils::time_util::timestamp_to_local_datetime};
use std::sync::Arc;

pub async fn connector_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<ConnectorListReq>,
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

    let mut connectors = Vec::new();
    for connector in state.mqtt_context.connector_manager.get_all_connector() {
        connectors.push(ConnectorListRow {
            connector_name: connector.connector_name.clone(),
            connector_type: connector.connector_type.to_string(),
            config: connector.config.clone(),
            topic_id: connector.topic_id.clone(),
            status: connector.status.to_string(),
            broker_id: if let Some(id) = connector.broker_id {
                id.to_string()
            } else {
                "-".to_string()
            },
            create_time: timestamp_to_local_datetime(connector.create_time as i64),
            update_time: timestamp_to_local_datetime(connector.update_time as i64),
        });
    }

    let filtered = apply_filters(connectors, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for ConnectorListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "connector_name" => Some(self.connector_name.clone()),
            "connector_type" => Some(self.connector_type.clone()),
            "topic_id" => Some(self.topic_id.clone()),
            "status" => Some(self.status.clone()),
            "broker_id" => Some(self.broker_id.clone()),
            _ => None,
        }
    }
}
