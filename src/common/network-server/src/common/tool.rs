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

use std::net::SocketAddr;
use std::sync::Arc;

use crate::common::{
    channel::RequestChannel, connection_manager::ConnectionManager, packet::RequestPackage,
};
use broker_core::cache::NodeCacheManager;
use common_metrics::mqtt::packets::record_packet_received_metrics;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::{mqtt::common::MqttPacket, robust::RobustMQPacket};
use rate_limit::global::GlobalRateLimiterManager;
use tracing::debug;

pub fn is_ignore_print(packet: &RobustMQPacket) -> bool {
    if let RobustMQPacket::MQTT(pack) = packet {
        if let MqttPacket::PingResp(_) = pack {
            return true;
        }
        if let MqttPacket::PingReq(_) = pack {
            return true;
        }
    }

    if let RobustMQPacket::KAFKA(_) = packet {
        return true;
    }

    false
}

pub async fn read_packet(
    pack: RobustMQPacket,
    request_channel: &RequestChannel,
    connection: &NetworkConnection,
    network_type: &NetworkConnectionType,
) {
    if !is_ignore_print(&pack) {
        debug!(
            "recv {} packet:{:?}, connect_id:{}",
            network_type, pack, connection.connection_id
        );
    }
    if let RobustMQPacket::MQTT(mqtt_pack) = &pack {
        record_packet_received_metrics(connection, mqtt_pack, network_type);
    }

    let package = RequestPackage::new(
        connection.connection_id,
        connection.addr,
        pack,
        network_type.clone(),
    );
    request_channel.send(package).await;
}

pub async fn check_connection_limit(
    global_limit_manager: &Arc<GlobalRateLimiterManager>,
    node_cache: &Arc<NodeCacheManager>,
    connection_manager: &Arc<ConnectionManager>,
    addr: &SocketAddr,
) -> bool {
    let _ = global_limit_manager.network_connection_rate_limit().await;

    let limit = node_cache.get_cluster_config().cluster_limit;

    // total connection count limit
    if connection_manager.connections.len() > limit.max_network_connection as usize {
        return true;
    }

    // per-IP connection count limit
    if connection_manager.ip_connection_count(addr) > limit.max_connection_per_ip {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_config::broker::default_broker_config;
    use rate_limit::global::GlobalRateLimiterManager;
    use std::net::SocketAddr;

    fn addr(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    fn make_conn(addr: &SocketAddr) -> NetworkConnection {
        NetworkConnection::new(NetworkConnectionType::Tcp, *addr, None)
    }

    #[tokio::test]
    async fn check_connection_limit_per_ip_pass_when_under_limit() {
        let limit_manager = Arc::new(GlobalRateLimiterManager::new(10000).unwrap());
        let cache = Arc::new(NodeCacheManager::new(default_broker_config()));
        let cm = Arc::new(ConnectionManager::new());
        let client_addr = addr("127.0.0.1:8080");

        cm.add_connection(make_conn(&client_addr));

        let result = check_connection_limit(&limit_manager, &cache, &cm, &client_addr).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn check_connection_limit_per_ip_rejects_when_over_limit() {
        let limit_manager = Arc::new(GlobalRateLimiterManager::new(10000).unwrap());
        let node_cache = Arc::new(NodeCacheManager::new(default_broker_config()));
        let cm = Arc::new(ConnectionManager::new());
        let client_addr = addr("127.0.0.1:8080");

        for _ in 0..5001 {
            cm.add_connection(make_conn(&client_addr));
        }

        let result = check_connection_limit(&limit_manager, &node_cache, &cm, &client_addr).await;
        assert!(result);
    }

    #[tokio::test]
    async fn check_connection_limit_ok_when_no_prior_connections() {
        let limit_manager = Arc::new(GlobalRateLimiterManager::new(10000).unwrap());
        let node_cache = Arc::new(NodeCacheManager::new(default_broker_config()));
        let cm = Arc::new(ConnectionManager::new());
        let client_addr = addr("192.168.1.1:9090");

        let result = check_connection_limit(&limit_manager, &node_cache, &cm, &client_addr).await;
        assert!(!result);
    }

    #[tokio::test]
    async fn check_connection_limit_per_ip_is_per_ip_not_port() {
        let limit_manager = Arc::new(GlobalRateLimiterManager::new(10000).unwrap());
        let node_cache = Arc::new(NodeCacheManager::new(default_broker_config()));
        let cm = Arc::new(ConnectionManager::new());

        let addr_a = addr("10.0.0.1:8080");
        let addr_b = addr("10.0.0.1:9090");

        for _ in 0..5001 {
            cm.add_connection(make_conn(&addr_a));
        }

        let result = check_connection_limit(&limit_manager, &node_cache, &cm, &addr_b).await;
        assert!(result);
    }

    #[tokio::test]
    async fn check_connection_limit_different_ips_independent() {
        let limit_manager = Arc::new(GlobalRateLimiterManager::new(10000).unwrap());
        let node_cache = Arc::new(NodeCacheManager::new(default_broker_config()));
        let cm = Arc::new(ConnectionManager::new());

        let addr_a = addr("10.0.0.1:8080");
        let addr_b = addr("10.0.0.2:8080");

        for _ in 0..5001 {
            cm.add_connection(make_conn(&addr_a));
        }

        let result = check_connection_limit(&limit_manager, &node_cache, &cm, &addr_b).await;
        assert!(!result);
    }
}
