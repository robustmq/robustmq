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
    request::ClientListReq,
    response::{ClientListRow, PageReplyData},
    state::HttpState,
    tool::query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
};
use axum::{extract::State, Json};
use common_base::{http_response::success_response, utils::time_util::timestamp_to_local_datetime};
use std::sync::Arc;

pub async fn client_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<ClientListReq>,
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

    let mut clients = Vec::new();
    for (connection_id, network_connection) in state.connection_manager.list_connect() {
        let connection_flag = if let Some(id) = params.connection_id {
            connection_id == id
        } else {
            true
        };

        if !connection_flag {
            continue;
        }

        let ip_flag = if let Some(ip) = params.source_ip.clone() {
            network_connection.addr.to_string().contains(&ip)
        } else {
            true
        };

        if !ip_flag {
            continue;
        }

        clients.push(ClientListRow {
            connection_id,
            connection_type: network_connection.connection_type.to_string(),
            protocol: if let Some(protocol) = network_connection.protocol {
                protocol.to_str()
            } else {
                "-".to_string()
            },
            source_addr: network_connection.addr.to_string(),
            create_time: timestamp_to_local_datetime(network_connection.create_time as i64),
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
            "connection_type" => Some(self.connection_type.to_string()),
            "protocol" => Some(self.protocol.to_string()),
            "source_addr" => Some(self.source_addr.clone()),
            _ => None,
        }
    }
}
