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

const MAX_SAMPLE_SIZE: usize = 100;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ClientListReq {
    pub tenant: Option<String>,
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
    pub tenant: String,
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
    let filter_tenant = params.tenant.clone();
    let filter_client_id = params.client_id.clone();
    let filter_connection_id = params.connection_id;
    let filter_source_ip = params.source_ip.clone();

    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        params.filter_field,
        params.filter_values,
        params.exact_match,
    );

    let cache = &state.mqtt_context.cache_manager;
    let total_count = cache.get_connection_count();

    let sample = sample_connections_up_to_100(
        &cache.connection_info,
        filter_tenant.as_deref(),
        filter_connection_id,
        filter_client_id.as_deref(),
        filter_source_ip.as_deref(),
    );

    let filtered = apply_filters(sample, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    let data = pagination
        .0
        .into_iter()
        .map(|lite| {
            let session = cache.get_session_info(&lite.client_id);
            let network_connection = state
                .connection_manager
                .get_connect(lite.mqtt_connection.connect_id);
            let heartbeat = cache.get_heartbeat(&lite.client_id);
            ClientListRow {
                tenant: lite.mqtt_connection.tenant.clone(),
                client_id: lite.client_id,
                connection_id: lite.connection_id,
                mqtt_connection: lite.mqtt_connection,
                network_connection,
                session,
                heartbeat,
            }
        })
        .collect::<Vec<ClientListRow>>();

    success_response(PageReplyData { data, total_count })
}

/// Collects up to MAX_SAMPLE_SIZE (100) connections from the cache, optionally filtered by
/// tenant, connection_id, client_id prefix, and source IP. When no tenant is specified,
/// iterates across all tenants and stops early once the limit is reached.
fn sample_connections_up_to_100(
    connection_info: &dashmap::DashMap<String, dashmap::DashMap<u64, MQTTConnection>>,
    filter_tenant: Option<&str>,
    filter_connection_id: Option<u64>,
    filter_client_id: Option<&str>,
    filter_source_ip: Option<&str>,
) -> Vec<ClientListRowLite> {
    let mut sample = Vec::with_capacity(MAX_SAMPLE_SIZE);

    if let Some(tenant) = filter_tenant {
        if let Some(inner) = connection_info.get(tenant) {
            collect_connections(
                inner.value(),
                &mut sample,
                filter_connection_id,
                filter_client_id,
                filter_source_ip,
            );
        }
    } else {
        for tenant_entry in connection_info.iter() {
            collect_connections(
                tenant_entry.value(),
                &mut sample,
                filter_connection_id,
                filter_client_id,
                filter_source_ip,
            );
            if sample.len() >= MAX_SAMPLE_SIZE {
                break;
            }
        }
    }

    sample
}

fn collect_connections(
    inner: &dashmap::DashMap<u64, MQTTConnection>,
    out: &mut Vec<ClientListRowLite>,
    filter_connection_id: Option<u64>,
    filter_client_id: Option<&str>,
    filter_source_ip: Option<&str>,
) {
    for inner_entry in inner.iter() {
        if out.len() >= MAX_SAMPLE_SIZE {
            break;
        }
        let connection_id = *inner_entry.key();
        let mqtt_client = inner_entry.value();
        if connection_matches(
            mqtt_client,
            connection_id,
            filter_connection_id,
            filter_client_id,
            filter_source_ip,
        ) {
            out.push(ClientListRowLite {
                connection_id,
                client_id: mqtt_client.client_id.clone(),
                mqtt_connection: mqtt_client.clone(),
            });
        }
    }
}

fn connection_matches(
    conn: &MQTTConnection,
    conn_id: u64,
    filter_connection_id: Option<u64>,
    filter_client_id: Option<&str>,
    filter_source_ip: Option<&str>,
) -> bool {
    if let Some(want_conn_id) = filter_connection_id {
        if conn_id != want_conn_id {
            return false;
        }
    }
    if let Some(prefix) = filter_client_id {
        if !conn.client_id.starts_with(prefix) {
            return false;
        }
    }
    if let Some(want_ip) = filter_source_ip {
        let ok = if want_ip.contains(':') {
            conn.source_ip_addr == want_ip
        } else {
            conn.source_ip_addr.starts_with(want_ip)
        };
        if !ok {
            return false;
        }
    }
    true
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
