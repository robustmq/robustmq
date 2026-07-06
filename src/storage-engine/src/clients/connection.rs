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

use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use common_base::tools::now_second;
use futures::{SinkExt, StreamExt};
use protocol::codec::{RobustMQCodec, RobustMQCodecWrapper};
use protocol::robust::RobustMQProtocol;
use protocol::storage::codec::StorageEnginePacket;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};
use tokio_util::codec::Framed;
use tracing::{debug, error};

const MAX_RETRY_TIMES: u32 = 10;
const RETRY_SLEEP_MS: u64 = 100;
const REQUEST_TIMEOUT_SECS: u64 = 30;

pub struct ClientConnection {
    pub stream: Framed<TcpStream, RobustMQCodec>,
}

pub struct NodeConnection {
    pub node_id: u64,
    cache_manager: Arc<StorageCacheManager>,
    // Tokio async mutex: held for the full duration of a send so concurrent
    // callers on the same slot queue rather than racing to create new connections.
    connection: Mutex<Option<ClientConnection>>,
    // Updated after every successful recv; read by GC without taking the mutex.
    last_active_secs: AtomicU64,
}

impl NodeConnection {
    pub fn new(node_id: u64, cache_manager: Arc<StorageCacheManager>) -> Self {
        Self {
            node_id,
            cache_manager,
            connection: Mutex::new(None),
            last_active_secs: AtomicU64::new(0),
        }
    }

    pub async fn send(
        &self,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        timeout(
            Duration::from_secs(REQUEST_TIMEOUT_SECS),
            self.send0(req_packet),
        )
        .await?
    }

    // Holds the mutex for the entire send: connect if needed, send, recv.
    // On I/O error the broken connection is cleared so the next attempt reconnects.
    async fn send0(
        &self,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        let mut guard = self.connection.lock().await;

        for attempt in 0..MAX_RETRY_TIMES {
            if guard.is_none() {
                match self.open().await {
                    Ok(stream) => *guard = Some(ClientConnection { stream }),
                    Err(e) => {
                        debug!(
                            "Connect to node {} failed ({}/{}): {}",
                            self.node_id,
                            attempt + 1,
                            MAX_RETRY_TIMES,
                            e
                        );
                        sleep(Duration::from_millis(RETRY_SLEEP_MS)).await;
                        continue;
                    }
                }
            }

            match self
                .send_and_recv(guard.as_mut().unwrap(), &req_packet)
                .await
            {
                Ok(resp) => return Ok(resp),
                Err(e) => {
                    *guard = None; // clear broken connection; next iteration reconnects
                    debug!(
                        "Send to node {} failed ({}/{}): {}",
                        self.node_id,
                        attempt + 1,
                        MAX_RETRY_TIMES,
                        e
                    );
                    sleep(Duration::from_millis(RETRY_SLEEP_MS)).await;
                }
            }
        }

        Err(StorageEngineError::SendRequestError(
            self.node_id,
            format!("exceeded max retry times ({})", MAX_RETRY_TIMES),
        ))
    }

    async fn send_and_recv(
        &self,
        conn: &mut ClientConnection,
        req_packet: &StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        conn.stream
            .send(RobustMQCodecWrapper::StorageEngine(req_packet.clone()))
            .await
            .map_err(|e| StorageEngineError::CommonErrorStr(format!("Send error: {}", e)))?;

        match conn.stream.next().await {
            Some(Ok(response)) => {
                self.last_active_secs.store(now_second(), Ordering::Relaxed);
                match response {
                    RobustMQCodecWrapper::StorageEngine(pkg) => Ok(pkg),
                    _ => Err(StorageEngineError::CommonErrorStr(
                        "Received unexpected packet type".to_string(),
                    )),
                }
            }
            Some(Err(e)) => Err(StorageEngineError::CommonErrorStr(format!(
                "Recv error: {}",
                e
            ))),
            None => Err(StorageEngineError::CommonErrorStr(
                "Connection closed unexpectedly".to_string(),
            )),
        }
    }

    async fn open(&self) -> Result<Framed<TcpStream, RobustMQCodec>, StorageEngineError> {
        let Some(node) = self
            .cache_manager
            .broker_cache
            .node_lists
            .get(&self.node_id)
        else {
            return Err(StorageEngineError::NodeNotFound(self.node_id));
        };
        let addr = node.engine_addr.clone();
        let socket = TcpStream::connect(&addr).await?;
        Ok(Framed::new(
            socket,
            RobustMQCodec::new_with_protocol(RobustMQProtocol::StorageEngine),
        ))
    }

    // Returns None if the connection has never been used (last_active_secs == 0).
    pub fn get_last_active_time(&self) -> Option<u64> {
        match self.last_active_secs.load(Ordering::Relaxed) {
            0 => None,
            t => Some(t),
        }
    }

    pub async fn close_connection(&self) -> Result<(), StorageEngineError> {
        if let Some(mut conn) = self.connection.lock().await.take() {
            if let Err(e) = conn.stream.close().await {
                error!("Failed to close connection to node {}: {}", self.node_id, e);
            }
        }
        Ok(())
    }
}
