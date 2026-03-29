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

use crate::common::connection_manager::ConnectionManager;
use protocol::nats::packet::{NatsPacket, ServerInfo};
use protocol::robust::{
    NatsWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use std::sync::Arc;
use tracing::error;

/// Send INFO to the client immediately after a NATS connection is established.
/// Also marks the connection's protocol as NATS so the handler can route responses.
pub async fn send_nats_info(connection_id: u64, connection_manager: &Arc<ConnectionManager>) {
    let wrapper = RobustMQPacketWrapper {
        protocol: RobustMQProtocol::NATS,
        extend: RobustMQWrapperExtend::NATS(NatsWrapperExtend {}),
        packet: RobustMQPacket::NATS(build_nats_info()),
    };
    if let Err(e) = connection_manager
        .write_tcp_frame(connection_id, wrapper)
        .await
    {
        error!(connection_id, "Failed to send NATS INFO: {}", e);
    }
}

/// Build the INFO packet sent to a NATS client immediately after connection.
/// Fields are hardcoded for now.
pub fn build_nats_info() -> NatsPacket {
    NatsPacket::Info(ServerInfo {
        server_id: "robustmq".to_string(),
        server_name: "robustmq".to_string(),
        version: "0.0.1".to_string(),
        proto: 1,
        host: "0.0.0.0".to_string(),
        port: 4222,
        headers: true,
        auth_required: false,
        tls_required: false,
        tls_verify: false,
        tls_available: false,
        max_payload: 1024 * 1024,
        jetstream: false,
        client_id: None,
        client_ip: None,
        nonce: None,
        cluster: None,
        cluster_dynamic: None,
        connect_urls: None,
        ws_connect_urls: None,
        ldm: None,
        git_commit: None,
        go: None,
        domain: None,
        xkey: None,
    })
}
