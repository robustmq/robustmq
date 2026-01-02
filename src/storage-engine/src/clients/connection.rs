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
use protocol::storage::codec::{StorageEngineCodec, StorageEnginePacket};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};
use tokio_util::codec::Framed;
use tracing::{error, warn};

const MAX_RETRY_TIMES: u32 = 10;
const RETRY_SLEEP_MS: u64 = 100;
const REQUEST_TIMEOUT_SECS: u64 = 30;

pub struct ClientConnection {
    pub stream: Framed<TcpStream, StorageEngineCodec>,
    pub last_active_time: u64,
}

pub struct NodeConnection {
    pub node_id: u64,
    cache_manager: Arc<StorageCacheManager>,
    connection: Mutex<Option<ClientConnection>>,
}

impl NodeConnection {
    pub fn new(node_id: u64, cache_manager: Arc<StorageCacheManager>) -> Self {
        Self {
            node_id,
            cache_manager,
            connection: Mutex::new(None),
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

    async fn send0(
        &self,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        let mut times = 0;
        loop {
            if times >= MAX_RETRY_TIMES {
                return Err(StorageEngineError::NoAvailableConn(self.node_id));
            }

            match self.try_send_with_connection(&req_packet).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    warn!(
                        "Send failed to node {}: {}, retry {}/{}",
                        self.node_id,
                        e,
                        times + 1,
                        MAX_RETRY_TIMES
                    );
                    times += 1;
                    sleep(Duration::from_millis(RETRY_SLEEP_MS)).await;
                }
            }
        }
    }

    async fn try_send_with_connection(
        &self,
        req_packet: &StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        let mut conn_guard = self.connection.lock().await;

        if conn_guard.is_none() {
            drop(conn_guard);
            self.ensure_connection().await?;
            conn_guard = self.connection.lock().await;
        }

        let conn = conn_guard
            .as_mut()
            .ok_or_else(|| StorageEngineError::NoAvailableConn(self.node_id))?;

        match self.send_and_recv(conn, req_packet).await {
            Ok(response) => Ok(response),
            Err(e) => {
                *conn_guard = None;
                Err(e)
            }
        }
    }

    async fn send_and_recv(
        &self,
        conn: &mut ClientConnection,
        req_packet: &StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        conn.stream
            .send(req_packet.clone())
            .await
            .map_err(|e| StorageEngineError::CommonErrorStr(format!("Send error: {}", e)))?;

        match conn.stream.next().await {
            Some(Ok(response)) => {
                conn.last_active_time = now_second();
                Ok(response)
            }
            Some(Err(e)) => Err(StorageEngineError::CommonErrorStr(format!(
                "Received packet error: {}",
                e
            ))),
            None => Err(StorageEngineError::CommonErrorStr(
                "Connection closed unexpectedly".to_string(),
            )),
        }
    }

    async fn ensure_connection(&self) -> Result<(), StorageEngineError> {
        let stream = self.open().await?;
        let mut conn_guard = self.connection.lock().await;
        *conn_guard = Some(ClientConnection {
            stream,
            last_active_time: now_second(),
        });
        Ok(())
    }

    async fn open(&self) -> Result<Framed<TcpStream, StorageEngineCodec>, StorageEngineError> {
        let Some(node) = self
            .cache_manager
            .broker_cache
            .node_lists
            .get(&self.node_id)
        else {
            return Err(StorageEngineError::NoAvailableConn(self.node_id));
        };
        let addr = node.engine_addr.clone();
        println!("{:?}", addr);
        let socket = TcpStream::connect(&addr).await?;
        Ok(Framed::new(socket, StorageEngineCodec::new()))
    }

    pub async fn connect(&self) -> Result<(), StorageEngineError> {
        let stream = self.open().await?;
        let new_conn = ClientConnection {
            stream,
            last_active_time: now_second(),
        };

        let mut conn_guard = self.connection.lock().await;
        if conn_guard.is_some() {
            warn!("Replaced existing connection on node {}", self.node_id);
        }
        *conn_guard = Some(new_conn);

        Ok(())
    }

    pub async fn get_last_active_time(&self) -> Option<u64> {
        let conn_guard = self.connection.lock().await;
        conn_guard.as_ref().map(|c| c.last_active_time)
    }

    pub async fn close_connection(&self) -> Result<(), StorageEngineError> {
        use futures::SinkExt;

        let mut conn_guard = self.connection.lock().await;
        if let Some(conn) = conn_guard.as_mut() {
            if let Err(e) = conn.stream.close().await {
                error!("Failed to close connection to node {}: {}", self.node_id, e);
            }
        }
        *conn_guard = None;
        Ok(())
    }
}
