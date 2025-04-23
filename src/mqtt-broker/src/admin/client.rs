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

use crate::handler::cache::CacheManager;
use metadata_struct::mqtt::connection::MQTTConnection;
use metadata_struct::mqtt::session::MqttSession;
use protocol::broker_mqtt::broker_mqtt_admin::{ClientRaw, ListClientReply};
use std::sync::Arc;
use tonic::{Response, Status};

pub async fn list_client(
    cache_manager: &Arc<CacheManager>,
) -> Result<Response<ListClientReply>, Status> {
    let client_query_result: Vec<ClientRaw> = cache_manager
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

    let reply = ListClientReply {
        clients: client_query_result,
    };

    Ok(Response::new(reply))
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
