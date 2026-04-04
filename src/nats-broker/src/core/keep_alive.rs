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

use axum::extract::ws::Message;
use bytes::BytesMut;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use common_config::broker::broker_config;
use metadata_struct::connection::NetworkConnectionType;
use network_server::common::connection_manager::ConnectionManager;
use protocol::codec::RobustMQCodec;
use protocol::nats::packet::NatsPacket;
use protocol::robust::{
    NatsWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio_util::codec::Encoder;
use tracing::{debug, info, warn};

/// Per-connection timeout for sending a single PING frame.
const PING_SEND_TIMEOUT: Duration = Duration::from_secs(1);
/// Per-connection timeout for sending the -ERR frame before closing.
const ERR_SEND_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone)]
pub struct NatsClientKeepAlive {
    connection_manager: Arc<ConnectionManager>,
}

impl NatsClientKeepAlive {
    pub fn new(connection_manager: Arc<ConnectionManager>) -> Self {
        NatsClientKeepAlive { connection_manager }
    }

    pub async fn start_heartbeat_check(&self, stop_send: &broadcast::Sender<bool>) {
        let conf = broker_config();
        let tick_ms = conf.nats_runtime.ping_interval * 1000;
        let ac_fn = async || -> ResultCommonError { self.tick().await };
        loop_select_ticket(ac_fn, tick_ms, stop_send).await;
    }

    async fn tick(&self) -> ResultCommonError {
        let conf = broker_config();
        let ping_interval = conf.nats_runtime.ping_interval;
        let ping_max = conf.nats_runtime.ping_max;
        let chunk_size = conf.nats_runtime.ping_send_chunk;

        let all_ids = self.collect_connection_ids();

        let alive_ids = self
            .close_stale_connections(&all_ids, ping_interval * ping_max)
            .await;

        self.send_ping_to_alive(&alive_ids, chunk_size, ping_interval)
            .await;

        info!(
            alive = alive_ids.len(),
            total = all_ids.len(),
            "NATS keep-alive tick completed"
        );
        Ok(())
    }

    fn collect_connection_ids(&self) -> Vec<u64> {
        self.connection_manager
            .connections
            .iter()
            .map(|e| *e.key())
            .collect()
    }

    /// Returns the subset of `ids` whose connections are still active.
    /// Timed-out connections are closed in parallel.
    async fn close_stale_connections(&self, ids: &[u64], timeout_threshold: u64) -> Vec<u64> {
        let now = now_second();
        let mut alive_ids = Vec::with_capacity(ids.len());
        let mut stale_ids = Vec::new();

        for &connect_id in ids {
            if let Some(conn) = self.connection_manager.get_connect(connect_id) {
                let elapsed = now.saturating_sub(conn.last_heartbeat_time);
                if elapsed >= timeout_threshold {
                    debug!(
                        connect_id,
                        elapsed, timeout_threshold, "NATS keep-alive timeout, closing connection"
                    );
                    stale_ids.push(connect_id);
                } else {
                    alive_ids.push(connect_id);
                }
            }
        }

        // Close all stale connections in a single background task (fire and forget).
        if !stale_ids.is_empty() {
            let cm = self.connection_manager.clone();
            tokio::spawn(async move {
                for connect_id in stale_ids {
                    close_stale_connection(&cm, connect_id).await;
                }
            });
        }

        alive_ids
    }

    /// Sends PING to all alive connections in parallel chunks.
    /// Waits for all chunk tasks with a total timeout of `ping_interval / 2`.
    async fn send_ping_to_alive(&self, alive_ids: &[u64], chunk_size: usize, ping_interval: u64) {
        let ping_wrapper = build_ping_wrapper();
        let mut handles = Vec::new();
        for chunk in alive_ids.chunks(chunk_size) {
            let chunk: Vec<u64> = chunk.to_vec();
            let cm = self.connection_manager.clone();
            let wrapper = ping_wrapper.clone();
            handles.push(tokio::spawn(async move {
                for connect_id in chunk {
                    send_ping_to_connection(&cm, connect_id, &wrapper).await;
                }
            }));
        }

        let batch_timeout = Duration::from_secs((ping_interval / 2).max(1));
        let wait_all = async {
            for handle in handles {
                let _ = handle.await;
            }
        };
        if tokio::time::timeout(batch_timeout, wait_all).await.is_err() {
            warn!(
                alive = alive_ids.len(),
                timeout_secs = (ping_interval / 2).max(1),
                "NATS keep-alive PING batch did not finish within timeout"
            );
        }
    }
}

/// Best-effort: send -ERR to the connection according to its type, then close it.
async fn close_stale_connection(cm: &Arc<ConnectionManager>, connect_id: u64) {
    let err_wrapper = build_err_wrapper("Stale Connection");

    let send_fut = async {
        let conn_type = cm
            .get_network_type(connect_id)
            .unwrap_or(NetworkConnectionType::Tcp);

        match conn_type {
            NetworkConnectionType::WebSocket | NetworkConnectionType::WebSockets => {
                let mut codec = RobustMQCodec::new();
                let mut buf = BytesMut::new();
                let nats_pkt = match err_wrapper.packet.clone() {
                    RobustMQPacket::NATS(p) => p,
                    _ => unreachable!(),
                };
                if codec.nats_codec.encode(nats_pkt, &mut buf).is_err() {
                    return;
                }
                let _ = cm
                    .write_websocket_frame(
                        connect_id,
                        err_wrapper,
                        Message::Binary(buf.to_vec().into()),
                    )
                    .await;
            }
            NetworkConnectionType::QUIC => {
                let _ = cm.write_quic_frame(connect_id, err_wrapper).await;
            }
            // Tcp and Tls: write_tcp_frame already routes Tls internally.
            NetworkConnectionType::Tcp | NetworkConnectionType::Tls => {
                let _ = cm.write_tcp_frame(connect_id, err_wrapper).await;
            }
        }
    };

    match tokio::time::timeout(ERR_SEND_TIMEOUT, send_fut).await {
        Ok(()) => {}
        Err(_) => warn!(connect_id, "Timed out sending -ERR on stale connection"),
    }

    cm.close_connect(connect_id).await;
    info!(
        connect_id,
        "NATS stale connection closed (keep-alive timeout)"
    );
}

async fn send_ping_to_connection(
    cm: &Arc<ConnectionManager>,
    connect_id: u64,
    wrapper: &RobustMQPacketWrapper,
) {
    let conn_type = cm
        .get_network_type(connect_id)
        .unwrap_or(NetworkConnectionType::Tcp);

    let send_result = match conn_type {
        NetworkConnectionType::WebSocket | NetworkConnectionType::WebSockets => {
            let mut codec = RobustMQCodec::new();
            let mut buf = BytesMut::new();
            let nats_pkt = match wrapper.packet.clone() {
                RobustMQPacket::NATS(p) => p,
                _ => unreachable!(),
            };
            if codec.nats_codec.encode(nats_pkt, &mut buf).is_err() {
                debug!(connect_id, "Failed to encode PING for WebSocket");
                return;
            }
            let send_fut = cm.write_websocket_frame(
                connect_id,
                wrapper.clone(),
                Message::Binary(buf.to_vec().into()),
            );
            tokio::time::timeout(PING_SEND_TIMEOUT, send_fut).await
        }
        NetworkConnectionType::QUIC => {
            let send_fut = cm.write_quic_frame(connect_id, wrapper.clone());
            tokio::time::timeout(PING_SEND_TIMEOUT, send_fut).await
        }
        NetworkConnectionType::Tcp | NetworkConnectionType::Tls => {
            let send_fut = cm.write_tcp_frame(connect_id, wrapper.clone());
            tokio::time::timeout(PING_SEND_TIMEOUT, send_fut).await
        }
    };

    match send_result {
        Ok(Ok(())) => debug!(connect_id, "Sent PING to NATS connection"),
        Ok(Err(e)) => debug!(connect_id, "Failed to send PING: {}", e),
        Err(_) => debug!(connect_id, "Timed out sending PING"),
    }
}

fn build_ping_wrapper() -> RobustMQPacketWrapper {
    RobustMQPacketWrapper {
        protocol: RobustMQProtocol::NATS,
        extend: RobustMQWrapperExtend::NATS(NatsWrapperExtend {}),
        packet: RobustMQPacket::NATS(NatsPacket::Ping),
    }
}

fn build_err_wrapper(msg: &str) -> RobustMQPacketWrapper {
    RobustMQPacketWrapper {
        protocol: RobustMQProtocol::NATS,
        extend: RobustMQWrapperExtend::NATS(NatsWrapperExtend {}),
        packet: RobustMQPacket::NATS(NatsPacket::Err(msg.to_string())),
    }
}
