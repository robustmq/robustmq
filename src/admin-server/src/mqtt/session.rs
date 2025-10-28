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
use metadata_struct::mqtt::lastwill::MqttLastWillData;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct SessionListReq {
    pub client_id: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SessionListRow {
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

use common_base::http_response::{error_response, success_response};
use mqtt_broker::storage::session::SessionStorage;
use std::sync::Arc;

pub async fn session_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SessionListReq>,
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

    let mut sessions = Vec::new();
    let storage = SessionStorage::new(state.client_pool.clone());
    if let Some(client_id) = params.client_id {
        if let Some(session) = state
            .mqtt_context
            .cache_manager
            .get_session_info(&client_id)
        {
            let last_will = match storage.get_last_will_message(client_id.clone()).await {
                Ok(data) => data,
                Err(e) => {
                    return error_response(e.to_string());
                }
            };
            sessions.push(SessionListRow {
                client_id: session.client_id.clone(),
                session_expiry: session.session_expiry,
                is_contain_last_will: session.is_contain_last_will,
                last_will_delay_interval: session.last_will_delay_interval,
                create_time: session.create_time,
                connection_id: session.connection_id,
                broker_id: session.broker_id,
                reconnect_time: session.reconnect_time,
                distinct_time: session.distinct_time,
                last_will,
            });
        }
    } else {
        for (_, session) in state.mqtt_context.cache_manager.session_info.clone() {
            let last_will = match storage
                .get_last_will_message(session.client_id.clone())
                .await
            {
                Ok(data) => data,
                Err(e) => {
                    return error_response(e.to_string());
                }
            };
            sessions.push(SessionListRow {
                client_id: session.client_id.clone(),
                session_expiry: session.session_expiry,
                is_contain_last_will: session.is_contain_last_will,
                last_will_delay_interval: session.last_will_delay_interval,
                create_time: session.create_time,
                connection_id: session.connection_id,
                broker_id: session.broker_id,
                reconnect_time: session.reconnect_time,
                distinct_time: session.distinct_time,
                last_will,
            });
        }
    }

    let filtered = apply_filters(sessions, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

impl Queryable for SessionListRow {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "client_id" => Some(self.client_id.clone()),
            _ => None,
        }
    }
}
