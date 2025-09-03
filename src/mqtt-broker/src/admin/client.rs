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
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::session::MqttSession;
use network_server::common::connection_manager::ConnectionManager;
use protocol::broker::broker_mqtt_admin::{
    ClientRaw, ListClientReply, ListClientRequest, ListConnectionRaw, ListConnectionReply,
};
use std::sync::Arc;

pub async fn list_connection_by_req(
    connection_manager: &Arc<ConnectionManager>,
    cache_manager: &Arc<MQTTCacheManager>,
) -> Result<ListConnectionReply, MqttBrokerError> {
    let mut reply = ListConnectionReply::default();
    let mut list_connection_raw: Vec<ListConnectionRaw> = Vec::new();
    for (key, value) in connection_manager.list_connect() {
        if let Some(mqtt_value) = cache_manager.get_connection(key) {
            let mqtt_info = serde_json::to_string(&mqtt_value)?;
            let raw = ListConnectionRaw {
                connection_id: value.connection_id,
                connection_type: value.connection_type.to_string(),
                protocol: match value.protocol {
                    Some(protocol) => format!("{protocol:?}"),
                    None => "None".to_string(),
                },
                source_addr: value.addr.to_string(),
                info: mqtt_info,
            };
            list_connection_raw.push(raw);
        }
    }
    reply.list_connection_raw = list_connection_raw;
    Ok(reply)
}

// List all clients by request
pub async fn list_client_by_req(
    cache_manager: &Arc<MQTTCacheManager>,
    _request: &ListClientRequest,
) -> Result<ListClientReply, MqttBrokerError> {
    let clients = cache_manager
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
        .collect();

    Ok(ListClientReply { clients })
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
