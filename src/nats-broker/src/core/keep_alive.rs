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

use crate::core::cache::NatsCacheManager;
use crate::core::write_client::write_nats_packet;
use crate::push::manager::NatsSubscribeManager;
use crate::push::queue::delete_members_by_group;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use network_server::common::connection_manager::ConnectionManager;
use protocol::nats::packet::NatsPacket;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{debug, warn};

/// Per-connection timeout for sending a single PING frame.
const PING_SEND_TIMEOUT: Duration = Duration::from_secs(1);
/// Per-connection timeout for sending the -ERR frame before closing.
const ERR_SEND_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Clone)]
pub struct NatsClientKeepAlive {
    connection_manager: Arc<ConnectionManager>,
    cache_manager: Arc<NatsCacheManager>,
    subscribe_manager: Arc<NatsSubscribeManager>,
    client_pool: Arc<ClientPool>,
}

impl NatsClientKeepAlive {
    pub fn new(
        connection_manager: Arc<ConnectionManager>,
        cache_manager: Arc<NatsCacheManager>,
        subscribe_manager: Arc<NatsSubscribeManager>,
        client_pool: Arc<ClientPool>,
    ) -> Self {
        NatsClientKeepAlive {
            connection_manager,
            cache_manager,
            subscribe_manager,
            client_pool,
        }
    }

    pub async fn start_heartbeat_check(&self, stop_send: &broadcast::Sender<bool>) {
        let conf = broker_config();
        let tick_ms = conf.nats_runtime.ping_interval * 1000;
        let ac_fn = async || -> ResultCommonError { self.tick().await };
        loop_select_ticket(ac_fn, tick_ms, stop_send).await;
    }

    async fn tick(&self) -> ResultCommonError {
        let conf = broker_config();

        let alive_ids = self
            .close_stale_connections(conf.nats_runtime.ping_interval * conf.nats_runtime.ping_max)
            .await;

        self.send_ping_to_alive(
            &alive_ids,
            conf.nats_runtime.ping_send_chunk,
            conf.nats_runtime.ping_interval,
        )
        .await;

        Ok(())
    }

    /// Returns the subset of NATS connections that are still active.
    /// Timed-out connections are closed in place.
    async fn close_stale_connections(&self, timeout_threshold: u64) -> Vec<u64> {
        let now = now_second();
        let mut alive_ids = Vec::new();
        let mut stale_ids = Vec::new();

        for entry in self.connection_manager.connections.iter() {
            let conn = entry.value();
            if !conn.protocol.as_ref().map(|p| p.is_nats()).unwrap_or(false) {
                continue;
            }
            let connect_id = *entry.key();
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

        // Close all stale connections in a single background task (fire and forget).
        if !stale_ids.is_empty() {
            for connect_id in stale_ids {
                close_stale_connection(&self.connection_manager, connect_id).await;
                self.cache_manager.remove_connection(connect_id);
                let removed = self.subscribe_manager.remove_by_connection(connect_id);
                delete_members_by_group(&self.client_pool, removed).await;
            }
        }

        alive_ids
    }

    /// Sends PING to all alive connections in parallel chunks.
    /// Waits for all chunk tasks with a total timeout of `ping_interval / 2`.
    async fn send_ping_to_alive(&self, alive_ids: &[u64], chunk_size: usize, ping_interval: u64) {
        let mut handles = Vec::new();
        for chunk in alive_ids.chunks(chunk_size) {
            let chunk: Vec<u64> = chunk.to_vec();
            let cm = self.connection_manager.clone();
            handles.push(tokio::spawn(async move {
                for connect_id in chunk {
                    send_ping_to_connection(&cm, connect_id).await;
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
    let send_fut = crate::core::write_client::write_nats_packet(
        cm,
        connect_id,
        NatsPacket::Err("Stale Connection".to_string()),
    );

    match tokio::time::timeout(ERR_SEND_TIMEOUT, send_fut).await {
        Ok(Ok(())) => {}
        Ok(Err(e)) => debug!(connect_id, "Failed to send -ERR: {}", e),
        Err(_) => warn!(connect_id, "Timed out sending -ERR on stale connection"),
    }

    cm.close_connect(connect_id).await;
    debug!(
        connect_id,
        "NATS stale connection closed (keep-alive timeout)"
    );
}

async fn send_ping_to_connection(cm: &Arc<ConnectionManager>, connect_id: u64) {
    let send_fut = write_nats_packet(cm, connect_id, NatsPacket::Ping);

    match tokio::time::timeout(PING_SEND_TIMEOUT, send_fut).await {
        Ok(Ok(())) => debug!(connect_id, "Sent PING to NATS connection"),
        Ok(Err(e)) => debug!(connect_id, "Failed to send PING: {}", e),
        Err(_) => debug!(connect_id, "Timed out sending PING"),
    }
}
