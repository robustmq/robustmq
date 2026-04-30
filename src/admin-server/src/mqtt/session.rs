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
use metadata_struct::mqtt::lastwill::MqttLastWillData;
use serde::{Deserialize, Serialize};

const MAX_SAMPLE_SIZE: usize = 100;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SessionListReq {
    pub tenant: Option<String>,
    pub client_id: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SessionListRow {
    pub tenant: String,
    pub client_id: String,
    pub session_expiry: u64,
    pub is_contain_last_will: bool,
    pub last_will_delay_interval: Option<u64>,
    pub create_time: u64,
    pub connection_id: Option<u64>,
    pub broker_id: Option<u64>,
    pub reconnect_time: Option<u64>,
    pub distinct_time: Option<u64>,
    pub last_will: Option<MqttLastWillData>,
}

use axum::extract::Query;
use common_base::http_response::{error_response, success_response};
use metadata_struct::mqtt::session::MqttSession;
use mqtt_broker::storage::last_will::LastWillStorage;
use std::sync::Arc;

pub async fn session_list(
    State(state): State<Arc<HttpState>>,
    Query(params): Query<SessionListReq>,
) -> String {
    // Extract filter params before build_query_params partially moves `params`.
    let filter_tenant = params.tenant.clone();
    let filter_client_id = params.client_id.clone();

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

    let sample =
        sample_sessions_up_to_100(cache, filter_tenant.as_deref(), filter_client_id.as_deref());

    let total_count = sample.len();

    let rows: Vec<SessionListRow> = sample
        .into_iter()
        .map(|session| SessionListRow {
            tenant: session.tenant.clone(),
            client_id: session.client_id.clone(),
            session_expiry: session.session_expiry_interval,
            is_contain_last_will: session.is_contain_last_will,
            last_will_delay_interval: session.last_will_delay_interval,
            create_time: session.create_time,
            connection_id: session.connection_id,
            broker_id: session.broker_id,
            reconnect_time: session.reconnect_time,
            distinct_time: session.distinct_time,
            last_will: None,
        })
        .collect();

    let sorted = apply_sorting(rows, &options);
    let pagination = apply_pagination(sorted, &options);

    let last_will_storage = LastWillStorage::new(state.mqtt_context.storage_driver_manager.clone());
    let mut data = Vec::with_capacity(pagination.0.len());
    for mut row in pagination.0 {
        let last_will = match last_will_storage
            .get_last_will_message(&row.tenant, &row.client_id)
            .await
        {
            Ok(v) => v,
            Err(e) => return error_response(e.to_string()),
        };
        row.last_will = last_will;
        data.push(row);
    }

    success_response(PageReplyData { data, total_count })
}

/// Collects up to MAX_SAMPLE_SIZE (100) sessions from the cache, optionally filtered by
/// tenant and client_id prefix. When tenant is specified, uses the index for O(1) lookup.
fn sample_sessions_up_to_100(
    cache: &mqtt_broker::core::cache::MQTTCacheManager,
    filter_tenant: Option<&str>,
    filter_client_id: Option<&str>,
) -> Vec<MqttSession> {
    let mut sample = Vec::with_capacity(MAX_SAMPLE_SIZE);

    if let Some(tenant) = filter_tenant {
        if let Some(id_set) = cache.tenant_session_index.get(tenant) {
            for client_id in id_set.iter() {
                if sample.len() >= MAX_SAMPLE_SIZE {
                    break;
                }
                if let Some(session) = cache.session_info.get(&*client_id) {
                    if session_matches(session.value(), filter_client_id) {
                        sample.push(session.clone());
                    }
                }
            }
        }
    } else {
        for entry in cache.session_info.iter() {
            if sample.len() >= MAX_SAMPLE_SIZE {
                break;
            }
            if session_matches(entry.value(), filter_client_id) {
                sample.push(entry.value().clone());
            }
        }
    }

    sample
}

fn session_matches(session: &MqttSession, filter_client_id: Option<&str>) -> bool {
    if let Some(keyword) = filter_client_id {
        if !session.client_id.contains(keyword) {
            return false;
        }
    }
    true
}

impl Queryable for SessionListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "tenant" => Some(self.tenant.clone()),
            "client_id" => Some(self.client_id.clone()),
            _ => None,
        }
    }
}
