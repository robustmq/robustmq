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
    request::mqtt::ClientListReq,
    response::{mqtt::ClientListRow, PageReplyData},
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::{extract::State, Json};
use common_base::http_response::success_response;
use std::sync::Arc;

pub async fn client_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<ClientListReq>,
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

    let mut clients: Vec<ClientListRow> = Vec::new();
    for (connection_id, mqtt_client) in state.mqtt_context.cache_manager.connection_info.clone() {
        let session = state
            .mqtt_context
            .cache_manager
            .get_session_info(&mqtt_client.client_id);
        let network_connection = state.connection_manager.get_connect(mqtt_client.connect_id);
        let heartbeat = state
            .mqtt_context
            .cache_manager
            .get_heartbeat(&mqtt_client.client_id);

        clients.push(ClientListRow {
            connection_id,
            client_id: mqtt_client.client_id.clone(),
            mqtt_connection: mqtt_client.clone(),
            network_connection,
            session,
            heartbeat,
        });
    }

    let filtered = apply_filters(clients, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for ClientListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "connection_id" => Some(self.connection_id.to_string()),
            "client_id" => Some(self.client_id.to_string()),
            _ => None,
        }
    }
}
