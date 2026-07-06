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

use crate::quic::stream::QuicFramedWriteStream;
use axum::extract::ws::{Message, WebSocket};
use common_base::tools::now_second;
use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use futures::stream::SplitSink;
use futures::SinkExt;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::codec::RobustMQCodec;
use protocol::robust::RobustMQProtocol;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio_util::codec::FramedWrite;
use tracing::debug;

type TcpWriter =
    Arc<Mutex<FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, RobustMQCodec>>>;
type TcpTlsWriter = Arc<
    Mutex<
        FramedWrite<
            tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
            RobustMQCodec,
        >,
    >,
>;
type WebSocketWriter = Arc<Mutex<SplitSink<WebSocket, Message>>>;
type QuicWriter = Arc<Mutex<QuicFramedWriteStream>>;

pub struct ConnectionManager {
    pub connections: DashMap<u64, NetworkConnection>,
    pub tcp_write_list: DashMap<u64, TcpWriter>,
    pub tcp_tls_write_list: DashMap<u64, TcpTlsWriter>,
    pub websocket_write_list: DashMap<u64, WebSocketWriter>,
    pub quic_write_list: DashMap<u64, QuicWriter>,
    pub ip_conn_count: DashMap<IpAddr, AtomicU64>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for ConnectionManager {
    fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
            tcp_write_list: self.tcp_write_list.clone(),
            tcp_tls_write_list: self.tcp_tls_write_list.clone(),
            websocket_write_list: self.websocket_write_list.clone(),
            quic_write_list: self.quic_write_list.clone(),
            ip_conn_count: DashMap::with_capacity(64),
        }
    }
}

// connection manager
impl ConnectionManager {
    pub fn new() -> ConnectionManager {
        let connections = DashMap::with_capacity(64);
        let tcp_write_list = DashMap::with_capacity(64);
        let tcp_tls_write_list = DashMap::with_capacity(64);
        let websocket_write_list = DashMap::with_capacity(64);
        let quic_write_list = DashMap::with_capacity(64);
        let ip_conn_count = DashMap::with_capacity(64);
        ConnectionManager {
            connections,
            tcp_write_list,
            tcp_tls_write_list,
            websocket_write_list,
            quic_write_list,
            ip_conn_count,
        }
    }

    pub fn add_connection(&self, connection: NetworkConnection) -> u64 {
        let connection_id = connection.connection_id();
        self.ip_conn_count
            .entry(connection.addr.ip())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
        self.connections.insert(connection_id, connection);
        connection_id
    }

    pub fn list_connect(&self) -> DashMap<u64, NetworkConnection> {
        self.connections.clone()
    }

    pub async fn mark_close_connect(&self, connection_id: u64) {
        if let Some(mut conn) = self.connections.get_mut(&connection_id) {
            conn.mark_close = now_second();
        }
    }

    pub fn get_connect(&self, connect_id: u64) -> Option<NetworkConnection> {
        if let Some(connect) = self.connections.get(&connect_id) {
            return Some(connect.clone());
        }
        None
    }

    pub fn get_connect_protocol(&self, connect_id: u64) -> Option<RobustMQProtocol> {
        if let Some(connect) = self.connections.get(&connect_id) {
            return connect.protocol.clone();
        }
        None
    }

    pub fn is_websocket(&self, connect_id: u64) -> bool {
        if let Some(connect) = self.connections.get(&connect_id) {
            return connect.connection_type == NetworkConnectionType::WebSocket;
        }
        false
    }

    pub fn is_quic(&self, connect_id: u64) -> bool {
        if let Some(connect) = self.connections.get(&connect_id) {
            return connect.connection_type == NetworkConnectionType::QUIC;
        }
        false
    }

    pub fn get_network_type(&self, connect_id: u64) -> Option<NetworkConnectionType> {
        if let Some(connect) = self.connections.get(&connect_id) {
            return Some(connect.connection_type.clone());
        }
        None
    }

    pub fn report_heartbeat(&self, connect_id: u64, time: u64) {
        if let Some(mut connect) = self.connections.get_mut(&connect_id) {
            connect.set_heartbeat_time(time);
        }
    }

    pub fn ip_connection_count(&self, addr: &SocketAddr) -> u64 {
        self.ip_conn_count
            .get(&addr.ip())
            .map(|r| r.load(Ordering::Relaxed))
            .unwrap_or(0)
    }
}

// Add Write
impl ConnectionManager {
    pub fn add_tcp_write(
        &self,
        connection_id: u64,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, RobustMQCodec>,
    ) {
        self.tcp_write_list
            .insert(connection_id, Arc::new(Mutex::new(write)));
    }

    pub fn add_tcp_tls_write(
        &self,
        connection_id: u64,
        write: FramedWrite<
            tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
            RobustMQCodec,
        >,
    ) {
        self.tcp_tls_write_list
            .insert(connection_id, Arc::new(Mutex::new(write)));
    }

    pub fn add_websocket_write(&self, connection_id: u64, write: SplitSink<WebSocket, Message>) {
        self.websocket_write_list
            .insert(connection_id, Arc::new(Mutex::new(write)));
    }

    pub fn add_mqtt_quic_write(
        &self,
        connection_id: u64,
        quic_framed_write_stream: QuicFramedWriteStream,
    ) {
        self.quic_write_list.insert(
            connection_id,
            Arc::new(Mutex::new(quic_framed_write_stream)),
        );
    }
}

// Set Protocol
impl ConnectionManager {
    pub fn set_mqtt_connect_protocol(&self, connect_id: u64, protocol: u8) {
        if let Some(mut connect) = self.connections.get_mut(&connect_id) {
            match protocol {
                3 => connect.set_protocol(RobustMQProtocol::MQTT3),
                4 => connect.set_protocol(RobustMQProtocol::MQTT4),
                5 => connect.set_protocol(RobustMQProtocol::MQTT5),
                10 => connect.set_protocol(RobustMQProtocol::KAFKA),
                11 => connect.set_protocol(RobustMQProtocol::StorageEngine),
                _ => {}
            };
        }
    }

    pub fn set_storage_engine_protocol(&self, connect_id: u64) {
        if let Some(mut connect) = self.connections.get_mut(&connect_id) {
            if connect.protocol.is_none() {
                connect.set_protocol(RobustMQProtocol::StorageEngine);
            }
        }
    }

    pub fn set_connect_protocol(&self, connect_id: u64, protocol: RobustMQProtocol) {
        if let Some(mut connect) = self.connections.get_mut(&connect_id) {
            connect.set_protocol(protocol);
        }
    }
}

// close connect
const CLOSE_TIMEOUT: Duration = Duration::from_secs(1);
impl ConnectionManager {
    pub async fn close_all_connect(&self) {
        for (connect_id, _) in self.connections.clone() {
            self.close_connect(connect_id).await;
        }
    }

    pub async fn close_connect(&self, connection_id: u64) {
        if let Some((_, conn)) = self.connections.remove(&connection_id) {
            let ip = conn.addr.ip();
            match self.ip_conn_count.entry(ip) {
                Entry::Occupied(entry) => {
                    let prev = entry.get().fetch_sub(1, Ordering::Relaxed);
                    if prev == 1 {
                        entry.remove();
                    }
                }
                Entry::Vacant(_) => {}
            }
        }

        if let Some((id, writer)) = self.tcp_write_list.remove(&connection_id) {
            match tokio::time::timeout(CLOSE_TIMEOUT, async {
                let mut stream = writer.lock().await;
                stream.close().await
            })
            .await
            {
                Ok(Ok(())) => debug!(
                    "server closes the tcp connection actively, connection id [{}]",
                    id
                ),
                Ok(Err(e)) => debug!("tcp close error for connection id [{}]: {}", id, e),
                Err(_) => debug!(
                    "tcp close timed out for connection id [{}], forcing drop",
                    id
                ),
            }
        }

        if let Some((id, writer)) = self.tcp_tls_write_list.remove(&connection_id) {
            match tokio::time::timeout(CLOSE_TIMEOUT, async {
                let mut stream = writer.lock().await;
                stream.close().await
            })
            .await
            {
                Ok(Ok(())) => debug!(
                    "server closes the tls connection actively, connection id [{}]",
                    id
                ),
                Ok(Err(e)) => debug!("tls close error for connection id [{}]: {}", id, e),
                Err(_) => debug!(
                    "tls close timed out for connection id [{}], forcing drop",
                    id
                ),
            }
        }

        if let Some((id, writer)) = self.websocket_write_list.remove(&connection_id) {
            match tokio::time::timeout(CLOSE_TIMEOUT, async {
                let mut stream = writer.lock().await;
                stream.close().await
            })
            .await
            {
                Ok(Ok(())) => debug!(
                    "server closes the websocket connection actively, connection id [{}]",
                    id
                ),
                Ok(Err(e)) => debug!("websocket close error for connection id [{}]: {}", id, e),
                Err(_) => debug!(
                    "websocket close timed out for connection id [{}], forcing drop",
                    id
                ),
            }
        }

        if let Some((id, _writer)) = self.quic_write_list.remove(&connection_id) {
            debug!(
                "server closes the quic connection actively, connection id [{}]",
                id
            );
        }
    }
}

// connection gc
impl ConnectionManager {
    pub async fn connection_gc(&self) {
        let now = now_second();
        let gc_ids: Vec<u64> = self
            .connections
            .iter()
            .filter_map(|entry| {
                let conn = entry.value();
                // Connection was explicitly marked for closure and the grace period (5s) has elapsed.
                let marked_and_expired = conn.mark_close > 0 && (now - conn.mark_close) > 5;
                // No heartbeat received for over 180s — treat as dead.
                let heartbeat_timeout = now - conn.last_heartbeat_time > 180;
                // Protocol handshake never completed within 30s — invalid connection.
                let stale_no_protocol = conn.protocol.is_none() && (now - conn.create_time) > 30;
                if marked_and_expired || heartbeat_timeout || stale_no_protocol {
                    Some(conn.connection_id)
                } else {
                    None
                }
            })
            .collect();

        for id in gc_ids {
            self.close_connect(id).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use metadata_struct::connection::NetworkConnectionType;
    use std::net::SocketAddr;

    fn addr(s: &str) -> SocketAddr {
        s.parse().unwrap()
    }

    fn new_conn(addr: &SocketAddr) -> NetworkConnection {
        NetworkConnection::new(NetworkConnectionType::Tcp, *addr, None)
    }

    #[tokio::test]
    async fn add_connection_tracks_ip_count() {
        let cm = ConnectionManager::new();
        let addr1 = addr("127.0.0.1:8080");
        let conn = new_conn(&addr1);

        cm.add_connection(conn);
        assert_eq!(cm.ip_connection_count(&addr1), 1);
    }

    #[tokio::test]
    async fn ip_connection_count_returns_zero_for_unknown_ip() {
        let cm = ConnectionManager::new();
        let addr = addr("127.0.0.1:8080");
        assert_eq!(cm.ip_connection_count(&addr), 0);
    }

    #[tokio::test]
    async fn multiple_connections_same_ip_increment_count() {
        let cm = ConnectionManager::new();
        let addr1 = addr("127.0.0.1:8080");

        cm.add_connection(new_conn(&addr1));
        cm.add_connection(new_conn(&addr1));
        cm.add_connection(new_conn(&addr1));
        assert_eq!(cm.ip_connection_count(&addr1), 3);
    }

    #[tokio::test]
    async fn different_ips_tracked_independently() {
        let cm = ConnectionManager::new();
        let addr1 = addr("127.0.0.1:8080");
        let addr2 = addr("192.168.1.1:9090");
        let addr3 = addr("10.0.0.1:3000");

        cm.add_connection(new_conn(&addr1));
        cm.add_connection(new_conn(&addr1));
        cm.add_connection(new_conn(&addr2));
        cm.add_connection(new_conn(&addr3));
        cm.add_connection(new_conn(&addr3));

        assert_eq!(cm.ip_connection_count(&addr1), 2);
        assert_eq!(cm.ip_connection_count(&addr2), 1);
        assert_eq!(cm.ip_connection_count(&addr3), 2);
    }

    #[tokio::test]
    async fn close_connect_decrements_ip_count() {
        let cm = ConnectionManager::new();
        let addr1 = addr("127.0.0.1:8080");

        let id1 = cm.add_connection(new_conn(&addr1));
        let id2 = cm.add_connection(new_conn(&addr1));
        assert_eq!(cm.ip_connection_count(&addr1), 2);

        cm.close_connect(id1).await;
        assert_eq!(cm.ip_connection_count(&addr1), 1);
        assert!(!cm.connections.contains_key(&id1));
        assert!(cm.connections.contains_key(&id2));
    }

    #[tokio::test]
    async fn close_connect_removes_ip_entry_when_count_reaches_zero() {
        let cm = ConnectionManager::new();
        let addr1 = addr("127.0.0.1:8080");

        let id = cm.add_connection(new_conn(&addr1));
        assert_eq!(cm.ip_connection_count(&addr1), 1);

        cm.close_connect(id).await;
        assert_eq!(cm.ip_connection_count(&addr1), 0);
        assert!(!cm.ip_conn_count.contains_key(&addr1.ip()));
    }

    #[tokio::test]
    async fn close_connect_on_unknown_id_does_not_panic() {
        let cm = ConnectionManager::new();
        cm.close_connect(99999).await;
    }

    #[tokio::test]
    async fn close_all_connect_cleans_up_ip_counts() {
        let cm = ConnectionManager::new();
        let addr1 = addr("127.0.0.1:8080");
        let addr2 = addr("192.168.1.1:9090");

        cm.add_connection(new_conn(&addr1));
        cm.add_connection(new_conn(&addr1));
        cm.add_connection(new_conn(&addr2));

        cm.close_all_connect().await;

        assert_eq!(cm.ip_connection_count(&addr1), 0);
        assert_eq!(cm.ip_connection_count(&addr2), 0);
        assert_eq!(cm.connections.len(), 0);
    }

    #[tokio::test]
    async fn same_ip_different_ports_share_count() {
        let cm = ConnectionManager::new();
        let addr_a = addr("127.0.0.1:8080");
        let addr_b = addr("127.0.0.1:9090");

        cm.add_connection(new_conn(&addr_a));
        cm.add_connection(new_conn(&addr_b));

        assert_eq!(cm.ip_connection_count(&addr_a), 2);
        assert_eq!(cm.ip_connection_count(&addr_b), 2);
    }
}
