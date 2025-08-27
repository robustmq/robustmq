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
use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::session::MqttSession;
use protocol::broker::broker_mqtt_admin::{ClientRaw, ListClientReply, ListClientRequest};
use std::sync::Arc;

// List all clients by request
pub async fn list_client_by_req(
    cache_manager: &Arc<CacheManager>,
    request: &ListClientRequest,
) -> Result<ListClientReply, MqttBrokerError> {
    let clients = extract_clients(cache_manager);
    let filtered = apply_filters(clients, &request.options);
    let sorted = apply_sorting(filtered, &request.options);
    let pagination = apply_pagination(sorted, &request.options);

    Ok(ListClientReply {
        clients: pagination.0,
        total_count: pagination.1 as u32,
    })
}

fn extract_clients(cache_manager: &Arc<CacheManager>) -> Vec<ClientRaw> {
    cache_manager
        .session_info
        .iter()
        .map(|session_entry| {
            let session = session_entry.value();
            let connection = session
                .connection_id
                .and_then(|cid| cache_manager.connection_info.get(&cid))
                .map(|c| c.value().clone());
            merge_client_info(session.clone(), connection)
        })
        .collect()
}

fn merge_client_info(session: MqttSession, connection: Option<MQTTConnection>) -> ClientRaw {
    let (is_online, conn_data) = match connection {
        Some(conn) => (true, conn),
        // if connection is None, it means the client is offline
        None => (false, MQTTConnection::default()),
    };

    ClientRaw {
        client_id: session.client_id.clone(),
        username: conn_data.login_user.clone(),
        is_online,
        source_ip: conn_data.source_ip_addr.clone(),
        connected_at: conn_data.create_time,
        keep_alive: conn_data.keep_alive as u32,
        // clean session is true when session_expiry is 0 (MQTT 5.0)
        clean_session: session.session_expiry == 0,
        session_expiry_interval: session.session_expiry,
    }
}

impl Queryable for ClientRaw {
    fn get_field_str(&self, field: &str) -> Option<String> {
        match field {
            "client_id" => Some(self.client_id.clone()),
            "username" => Some(self.username.clone()),
            "is_online" => Some(self.is_online.to_string()),
            "source_ip" => Some(self.source_ip.clone()),
            "connected_at" => Some(self.connected_at.to_string()),
            "keep_alive" => Some(self.keep_alive.to_string()),
            "clean_session" => Some(self.clean_session.to_string()),
            "session_expiry_interval" => Some(self.session_expiry_interval.to_string()),
            _ => None,
        }
    }
}
