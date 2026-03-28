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
use dashmap::DashMap;
use futures::stream::SplitSink;
use futures::SinkExt;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::codec::RobustMQCodec;
use protocol::robust::RobustMQProtocol;
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

#[derive(Clone, Default)]
pub struct ConnectionManager {
    pub connections: DashMap<u64, NetworkConnection>,
    pub tcp_write_list: DashMap<u64, TcpWriter>,
    pub tcp_tls_write_list: DashMap<u64, TcpTlsWriter>,
    pub websocket_write_list: DashMap<u64, WebSocketWriter>,
    pub quic_write_list: DashMap<u64, QuicWriter>,
}

// connection manager
impl ConnectionManager {
    pub fn new() -> ConnectionManager {
        let connections = DashMap::with_capacity(64);
        let tcp_write_list = DashMap::with_capacity(64);
        let tcp_tls_write_list = DashMap::with_capacity(64);
        let websocket_write_list = DashMap::with_capacity(64);
        let quic_write_list = DashMap::with_capacity(64);
        ConnectionManager {
            connections,
            tcp_write_list,
            tcp_tls_write_list,
            websocket_write_list,
            quic_write_list,
        }
    }

    pub fn add_connection(&self, connection: NetworkConnection) -> u64 {
        let connection_id = connection.connection_id();
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
        self.connections.remove(&connection_id);

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
