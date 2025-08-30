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

use crate::admin::query::{apply_filters, apply_pagination, apply_sorting, Queryable};
use crate::handler::cache::MQTTCacheManager;
use crate::handler::error::MqttBrokerError;
use protocol::broker::broker_mqtt_admin::{ListSessionReply, ListSessionRequest, SessionRaw};
use std::sync::Arc;

pub async fn list_session_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    request: &ListSessionRequest,
) -> Result<ListSessionReply, MqttBrokerError> {
    let sessions = extract_sessions(cache_manager);
    let filtered = apply_filters(sessions, &request.options);
    let sorted = apply_sorting(filtered, &request.options);
    let pagination = apply_pagination(sorted, &request.options);

    Ok(ListSessionReply {
        sessions: pagination.0,
        total_count: pagination.1 as u32,
    })
}

fn extract_sessions(cache_manager: &Arc<MQTTCacheManager>) -> Vec<SessionRaw> {
    cache_manager
        .session_info
        .iter()
        .map(|entry| {
            let session = entry.value();
            SessionRaw {
                client_id: session.client_id.clone(),
                session_expiry: session.session_expiry,
                is_contain_last_will: session.is_contain_last_will,
                last_will_delay_interval: session.last_will_delay_interval,
                create_time: session.create_time,
                connection_id: session.connection_id,
                broker_id: session.broker_id,
                reconnect_time: session.reconnect_time,
                distinct_time: session.distinct_time,
            }
        })
        .collect()
}

impl Queryable for SessionRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "client_id" => Some(self.client_id.clone()),
            "session_expiry" => Some(self.session_expiry.to_string()),
            "is_contain_last_will" => Some(self.is_contain_last_will.to_string()),
            "last_will_delay_interval" => self.last_will_delay_interval.map(|v| v.to_string()),
            "create_time" => Some(self.create_time.to_string()),
            "connection_id" => self.connection_id.map(|v| v.to_string()),
            "broker_id" => self.broker_id.map(|v| v.to_string()),
            "reconnect_time" => self.reconnect_time.map(|v| v.to_string()),
            "distinct_time" => self.distinct_time.map(|v| v.to_string()),
            _ => None,
        }
    }
}
