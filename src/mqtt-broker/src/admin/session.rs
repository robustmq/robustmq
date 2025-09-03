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

use crate::handler::cache::MQTTCacheManager;
use crate::handler::error::MqttBrokerError;
use protocol::broker::broker_mqtt_admin::{ListSessionReply, ListSessionRequest, SessionRaw};
use std::sync::Arc;

pub async fn list_session_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    _request: &ListSessionRequest,
) -> Result<ListSessionReply, MqttBrokerError> {
    let sessions = cache_manager
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
        .collect();

    Ok(ListSessionReply { sessions })
}
