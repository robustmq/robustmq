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
use broker_core::cache::NodeCacheManager;
use common_base::tools::get_local_ip;
use common_base::uuid::unique_id;
use common_base::version::version;
use common_config::broker::broker_config;
use common_config::config::BrokerConfig;
use metadata_struct::connection::NetworkConnectionType;
use protocol::nats::packet::{NatsPacket, ServerInfo};
use protocol::robust::{
    NatsWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::error;

/// Send INFO + PING to the client immediately after a NATS connection is established.
///
/// Standard NATS handshake: Server → INFO, Server → PING, Client → CONNECT, Client → PONG.
/// The PING prompts the client to complete the handshake without waiting for the
/// keep-alive tick. TCP_NODELAY is set on the socket so both packets are flushed immediately.
pub async fn send_nats_info(
    node_cache: &Arc<NodeCacheManager>,
    connection_id: u64,
    connection_manager: &Arc<ConnectionManager>,
    network_type: &NetworkConnectionType,
    addr: &SocketAddr,
) {
    let info_wrapper = RobustMQPacketWrapper {
        protocol: RobustMQProtocol::NATS,
        extend: RobustMQWrapperExtend::NATS(NatsWrapperExtend {}),
        packet: RobustMQPacket::NATS(build_nats_info(
            node_cache,
            connection_id,
            network_type,
            addr,
        )),
    };

    if let Err(e) = connection_manager
        .write_tcp_frame(connection_id, info_wrapper)
        .await
    {
        error!(connection_id, "Failed to send NATS INFO: {}", e);
        return;
    }

    let ping_wrapper = RobustMQPacketWrapper {
        protocol: RobustMQProtocol::NATS,
        extend: RobustMQWrapperExtend::NATS(NatsWrapperExtend {}),
        packet: RobustMQPacket::NATS(NatsPacket::Ping),
    };

    if let Err(e) = connection_manager
        .write_tcp_frame(connection_id, ping_wrapper)
        .await
    {
        error!(connection_id, "Failed to send NATS PING: {}", e);
    }
}

fn build_nats_info(
    node_cache: &Arc<NodeCacheManager>,
    connection_id: u64,
    network_type: &NetworkConnectionType,
    addr: &SocketAddr,
) -> NatsPacket {
    let conf = broker_config();
    let local_ip = conf.broker_ip.clone().unwrap_or_else(get_local_ip);
    NatsPacket::Info(ServerInfo {
        server_id: conf.broker_id.to_string(),
        server_name: format!("server-{}", conf.broker_id),
        version: version(),
        proto: 1,
        host: local_ip,
        port: get_port_by_network_type(conf, network_type) as u16,
        headers: true,
        auth_required: conf.nats_runtime.auth_required,
        tls_required: false,
        tls_verify: false,
        tls_available: matches!(
            network_type,
            NetworkConnectionType::Tcp | NetworkConnectionType::WebSocket
        ),
        max_payload: conf.nats_runtime.max_payload,
        jetstream: false,
        client_id: Some(connection_id),
        client_ip: Some(addr.ip().to_string()),
        nonce: Some(unique_id()),
        cluster: Some(conf.cluster_name.clone()),
        cluster_dynamic: Some(true),
        connect_urls: Some(get_connect_urls(node_cache, network_type)),
        ws_connect_urls: Some(get_ws_connect_urls(node_cache, network_type)),
        ldm: Some(false),
        git_commit: Some("-".to_string()),
        go: Some("robustmq-rust".to_string()),
        domain: Some("-".to_string()),
        xkey: None,
    })
}

fn get_port_by_network_type(conf: &BrokerConfig, network_type: &NetworkConnectionType) -> u32 {
    match network_type.clone() {
        NetworkConnectionType::QUIC => 0,
        NetworkConnectionType::Tcp => conf.nats_runtime.tcp_port,
        NetworkConnectionType::Tls => conf.nats_runtime.tls_port,
        NetworkConnectionType::WebSocket => conf.nats_runtime.ws_port,
        NetworkConnectionType::WebSockets => conf.nats_runtime.wss_port,
    }
}

fn get_connect_urls(
    node_cache: &Arc<NodeCacheManager>,
    network_type: &NetworkConnectionType,
) -> Vec<String> {
    let is_ssl = network_type.clone() == NetworkConnectionType::Tls
        || network_type.clone() == NetworkConnectionType::WebSockets;
    if is_ssl {
        build_network_connect_urls(node_cache, &NetworkConnectionType::Tls)
    } else {
        build_network_connect_urls(node_cache, &NetworkConnectionType::Tcp)
    }
}

fn get_ws_connect_urls(
    node_cache: &Arc<NodeCacheManager>,
    network_type: &NetworkConnectionType,
) -> Vec<String> {
    let is_ssl = network_type.clone() == NetworkConnectionType::Tls
        || network_type.clone() == NetworkConnectionType::WebSockets;
    if is_ssl {
        build_network_connect_urls(node_cache, &NetworkConnectionType::WebSocket)
    } else {
        build_network_connect_urls(node_cache, &NetworkConnectionType::WebSockets)
    }
}

fn build_network_connect_urls(
    node_cache: &Arc<NodeCacheManager>,
    network_type: &NetworkConnectionType,
) -> Vec<String> {
    match network_type {
        NetworkConnectionType::QUIC => Vec::new(),
        NetworkConnectionType::Tls => node_cache
            .node_list()
            .iter()
            .map(|data| data.extend.nats.tls_addr.clone())
            .collect(),
        NetworkConnectionType::Tcp => node_cache
            .node_list()
            .iter()
            .map(|data| data.extend.nats.tcp_addr.clone())
            .collect(),
        NetworkConnectionType::WebSocket => node_cache
            .node_list()
            .iter()
            .map(|data| data.extend.nats.ws_addr.clone())
            .collect(),
        NetworkConnectionType::WebSockets => node_cache
            .node_list()
            .iter()
            .map(|data| data.extend.nats.wss_addr.clone())
            .collect(),
    }
}
