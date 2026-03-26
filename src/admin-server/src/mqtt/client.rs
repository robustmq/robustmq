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
        query::{apply_pagination, apply_sorting, build_query_params, Queryable},
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
    pub client_id: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
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
use mqtt_broker::core::cache::MQTTCacheManager;
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
        None,
        None,
        None,
    );

    let cache = &state.mqtt_context.cache_manager;
    let total_count = if let Some(tenant) = params.tenant.as_deref() {
        cache.get_connection_count_by_tenant(tenant)
    } else {
        cache.get_connection_count()
    };

    let sample =
        sample_connections_up_to_100(cache, params.tenant.as_deref(), params.client_id.as_deref());

    let sorted = apply_sorting(sample, &options);
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

fn sample_connections_up_to_100(
    cache: &MQTTCacheManager,
    filter_tenant: Option<&str>,
    filter_client_id: Option<&str>,
) -> Vec<ClientListRowLite> {
    let mut sample = Vec::with_capacity(MAX_SAMPLE_SIZE);

    if let Some(tenant) = filter_tenant {
        // Use the index to iterate only over connect_ids belonging to this tenant
        if let Some(id_set) = cache.tenant_connection_index.get(tenant) {
            for connect_id in id_set.iter() {
                if sample.len() >= MAX_SAMPLE_SIZE {
                    break;
                }
                if let Some(conn) = cache.connection_info.get(&*connect_id) {
                    if let Some(keyword) = filter_client_id {
                        if !conn.client_id.contains(keyword) {
                            continue;
                        }
                    }
                    sample.push(ClientListRowLite {
                        connection_id: *connect_id,
                        client_id: conn.client_id.clone(),
                        mqtt_connection: conn.clone(),
                    });
                }
            }
        }
    } else {
        for entry in cache.connection_info.iter() {
            if sample.len() >= MAX_SAMPLE_SIZE {
                break;
            }
            let conn = entry.value();
            if let Some(keyword) = filter_client_id {
                if !conn.client_id.contains(keyword) {
                    continue;
                }
            }
            sample.push(ClientListRowLite {
                connection_id: *entry.key(),
                client_id: conn.client_id.clone(),
                mqtt_connection: conn.clone(),
            });
        }
    }

    sample
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
