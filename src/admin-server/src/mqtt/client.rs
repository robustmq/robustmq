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
use axum::extract::State;
use common_base::http_response::success_response;
use metadata_struct::{
    connection::NetworkConnection,
    mqtt::{connection::MQTTConnection, session::MqttSession},
};
use mqtt_broker::core::cache::ConnectionLiveTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ClientListReq {
    pub source_ip: Option<String>,
    pub client_id: Option<String>,
    pub connection_id: Option<u64>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ClientListRow {
    pub client_id: String,
    pub connection_id: u64,
    pub mqtt_connection: MQTTConnection,
    pub network_connection: Option<NetworkConnection>,
    pub session: Option<MqttSession>,
    pub heartbeat: Option<ConnectionLiveTime>,
}
use axum::extract::Query;
use std::sync::Arc;

#[derive(Clone)]
struct ClientListRowLite {
    pub client_id: String,
    pub connection_id: u64,
    pub mqtt_connection: MQTTConnection,
}

pub async fn client_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<ClientListReq>,
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

    // Build a lightweight list first (only fields needed for filtering/sorting/pagination),
    // then enrich paginated rows with session/network/heartbeat data.
    let mut clients: Vec<ClientListRowLite> = Vec::new();
    for entry in state.mqtt_context.cache_manager.connection_info.iter() {
        let connection_id = *entry.key();
        let mqtt_client = entry.value();

        if let Some(want_conn_id) = params.connection_id {
            // `connect_id` should equal the dashmap key, but accept either for safety.
            if connection_id != want_conn_id && mqtt_client.connect_id != want_conn_id {
                continue;
            }
        }

        if let Some(want_client_id) = &params.client_id {
            if mqtt_client.client_id != *want_client_id {
                continue;
            }
        }

        if let Some(want_source_ip) = &params.source_ip {
            let ok = if want_source_ip.contains(':') {
                mqtt_client.source_ip_addr == *want_source_ip
            } else {
                mqtt_client.source_ip_addr.starts_with(want_source_ip)
            };
            if !ok {
                continue;
            }
        }

        clients.push(ClientListRowLite {
            connection_id,
            client_id: mqtt_client.client_id.clone(),
            mqtt_connection: mqtt_client.clone(),
        });
    }

    let filtered = apply_filters(clients, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    let data = pagination
        .0
        .into_iter()
        .map(|lite| {
            let session = state
                .mqtt_context
                .cache_manager
                .get_session_info(&lite.client_id);
            let network_connection = state
                .connection_manager
                .get_connect(lite.mqtt_connection.connect_id);
            let heartbeat = state
                .mqtt_context
                .cache_manager
                .get_heartbeat(&lite.client_id);
            ClientListRow {
                client_id: lite.client_id,
                connection_id: lite.connection_id,
                mqtt_connection: lite.mqtt_connection,
                network_connection,
                session,
                heartbeat,
            }
        })
        .collect::<Vec<ClientListRow>>();

    success_response(PageReplyData {
        data,
        total_count: pagination.1,
    })
}

impl Queryable for ClientListRowLite {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "connection_id" => Some(self.connection_id.to_string()),
            "client_id" => Some(self.client_id.to_string()),
            "source_ip" => Some(self.mqtt_connection.source_ip_addr.clone()),
            _ => None,
        }
    }
}

impl Queryable for ClientListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "connection_id" => Some(self.connection_id.to_string()),
            "client_id" => Some(self.client_id.to_string()),
            "source_ip" => Some(self.mqtt_connection.source_ip_addr.clone()),
            _ => None,
        }
    }
}
