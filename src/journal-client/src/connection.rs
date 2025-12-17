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

use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::time::Duration;

use common_base::tools::now_second;
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use protocol::storage::codec::{JournalEnginePacket, JournalServerCodec};
use tokio::net::TcpStream;
use tokio::select;
use tokio::sync::broadcast::Receiver;
use tokio::time::sleep;
use tokio_util::codec::Framed;
use tracing::error;

use crate::cache::MetadataCache;
use crate::consts::{ADMIN_NODE_ID, MODULE_ADMIN, MODULE_READ, MODULE_WRITE};
use crate::error::JournalClientError;

pub struct ClientConnection {
    pub stream: Framed<TcpStream, JournalServerCodec>,
    pub last_active_time: u64,
}

pub struct NodeConnection {
    node_id: i64,
    metadata_cache: Arc<MetadataCache>,
    connection: DashMap<String, ClientConnection>,
}

impl NodeConnection {
    pub fn new(node_id: i64, metadata_cache: Arc<MetadataCache>) -> Self {
        let connection = DashMap::with_capacity(2);
        NodeConnection {
            node_id,
            metadata_cache,
            connection,
        }
    }

    pub async fn admin_send(
        &self,
        req_packet: JournalEnginePacket,
    ) -> Result<JournalEnginePacket, JournalClientError> {
        self.send("admin", req_packet).await
    }

    pub async fn write_send(
        &self,
        req_packet: JournalEnginePacket,
    ) -> Result<JournalEnginePacket, JournalClientError> {
        self.send("write", req_packet).await
    }

    pub async fn read_send(
        &self,
        req_packet: JournalEnginePacket,
    ) -> Result<JournalEnginePacket, JournalClientError> {
        self.send("read", req_packet).await
    }

    async fn send(
        &self,
        module: &str,
        req_packet: JournalEnginePacket,
    ) -> Result<JournalEnginePacket, JournalClientError> {
        let mut times = 3;
        let response_max_try_mut_times = 10;
        let response_try_mut_sleep_time_ms = 100;
        loop {
            match self.connection.try_get_mut(module) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.stream.send(req_packet.clone()).await {
                        Ok(()) => {
                            if let Some(data) = da.stream.next().await {
                                match data {
                                    Ok(da) => return Ok(da),
                                    Err(e) => {
                                        return Err(JournalClientError::ReceivedPacketError(
                                            self.node_id,
                                            e.to_string(),
                                        ));
                                    }
                                }
                            } else {
                                return Err(JournalClientError::ReceivedPacketIsEmpty(
                                    self.node_id,
                                ));
                            }
                        }
                        Err(e) => {
                            if times > response_max_try_mut_times {
                                return Err(JournalClientError::SendRequestError(
                                    self.node_id,
                                    e.to_string(),
                                ));
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    let stream = match self.open().await {
                        Ok(stream) => stream,
                        Err(e) => {
                            error!("{}", e);
                            continue;
                        }
                    };
                    self.connection.insert(
                        module.to_string(),
                        ClientConnection {
                            stream,
                            last_active_time: now_second(),
                        },
                    );

                    if times > response_max_try_mut_times {
                        return Err(JournalClientError::NoAvailableConn(self.node_id));
                    }
                }
                dashmap::try_result::TryResult::Locked => {
                    if times > response_max_try_mut_times {
                        return Err(JournalClientError::ConnectionIsOccupied(self.node_id));
                    }
                }
            }
            times += 1;
            sleep(Duration::from_millis(response_try_mut_sleep_time_ms)).await;
        }
    }

    async fn open(&self) -> Result<Framed<TcpStream, JournalServerCodec>, JournalClientError> {
        let addr = if self.node_id == ADMIN_NODE_ID {
            self.metadata_cache.get_rand_addr()
        } else if let Some(addr) = self
            .metadata_cache
            .get_tcp_addr_by_node_id(self.node_id as u64)
        {
            addr
        } else {
            return Err(JournalClientError::NodeNoAvailableAddr(self.node_id));
        };

        let socket = TcpStream::connect(&addr).await?;
        Ok(Framed::new(socket, JournalServerCodec::new()))
    }

    pub async fn init_conn(&self, module: &str) -> Result<(), JournalClientError> {
        match module {
            "admin" => self.connection.insert(
                MODULE_ADMIN.to_string(),
                ClientConnection {
                    stream: self.open().await?,
                    last_active_time: now_second(),
                },
            ),
            "read" => self.connection.insert(
                MODULE_READ.to_string(),
                ClientConnection {
                    stream: self.open().await?,
                    last_active_time: now_second(),
                },
            ),
            "write" => self.connection.insert(
                MODULE_WRITE.to_string(),
                ClientConnection {
                    stream: self.open().await?,
                    last_active_time: now_second(),
                },
            ),
            _ => None,
        };

        Ok(())
    }
}

pub struct ConnectionManager {
    // node_id, NodeConnection
    node_conns: DashMap<i64, NodeConnection>,
    metadata_cache: Arc<MetadataCache>,
    admin_conn_atom: AtomicU64,
}

impl ConnectionManager {
    pub fn new(metadata_cache: Arc<MetadataCache>) -> Self {
        let node_conns = DashMap::with_capacity(2);
        let admin_conn_atom = AtomicU64::new(0);
        ConnectionManager {
            node_conns,
            metadata_cache,
            admin_conn_atom,
        }
    }

    pub async fn admin_send(
        &self,
        req_packet: JournalEnginePacket,
    ) -> Result<JournalEnginePacket, JournalClientError> {
        let node_id = if let Some(node_id) = self.choose_admin_node() {
            node_id as i64
        } else {
            ADMIN_NODE_ID
        };

        if !self.node_conns.contains_key(&node_id) {
            let conn = NodeConnection::new(node_id, self.metadata_cache.clone());
            conn.init_conn(MODULE_ADMIN).await?;
            self.node_conns.insert(node_id, conn);
        }

        let conn = self.node_conns.get(&node_id).unwrap();
        conn.admin_send(req_packet).await
    }

    pub async fn write_send(
        &self,
        node_id: u64,
        req_packet: JournalEnginePacket,
    ) -> Result<JournalEnginePacket, JournalClientError> {
        let new_node_id = node_id as i64;
        if !self.node_conns.contains_key(&new_node_id) {
            let conn = NodeConnection::new(new_node_id, self.metadata_cache.clone());
            conn.init_conn(MODULE_WRITE).await?;
            self.node_conns.insert(new_node_id, conn);
        }

        let conn = self.node_conns.get(&new_node_id).unwrap();
        conn.write_send(req_packet).await
    }

    pub async fn read_send(
        &self,
        node_id: u64,
        req_packet: JournalEnginePacket,
    ) -> Result<JournalEnginePacket, JournalClientError> {
        let new_node_id = node_id as i64;
        if !self.node_conns.contains_key(&new_node_id) {
            let conn = NodeConnection::new(new_node_id, self.metadata_cache.clone());
            conn.init_conn(MODULE_READ).await?;
            self.node_conns.insert(new_node_id, conn);
        }

        let conn = self.node_conns.get(&new_node_id).unwrap();
        conn.read_send(req_packet).await
    }

    fn choose_admin_node(&self) -> Option<u64> {
        let node_ids = self.metadata_cache.all_node_ids();
        if node_ids.is_empty() {
            return None;
        }
        let posi = self
            .admin_conn_atom
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let index = posi as usize % node_ids.len();
        let node_id = node_ids.get(index).unwrap();
        Some(*node_id)
    }

    pub fn get_inactive_conn(&self) -> Vec<(i64, String)> {
        let mut results = Vec::new();
        for node in self.node_conns.iter() {
            for conn in node.connection.iter() {
                if (now_second() - conn.last_active_time) > 600 {
                    results.push((node.node_id, conn.key().to_string()));
                }
            }
        }
        results
    }

    pub async fn close_conn_by_node(&self, node_id: i64, conn_type: &str) {
        if let Some(node) = self.node_conns.get(&node_id) {
            if let Some(mut conn) = node.connection.get_mut(conn_type) {
                if let Err(e) = conn.stream.close().await {
                    error!("{}", e);
                }
            }
            node.connection.remove(conn_type);
        }
    }

    pub async fn close(&self) {
        for node in self.node_conns.iter() {
            for mut conn in node.connection.iter_mut() {
                if let Err(e) = conn.stream.close().await {
                    error!("{}", e);
                }
            }
        }
    }
}

pub fn start_conn_gc_thread(
    metadata_cache: Arc<MetadataCache>,
    connection_manager: Arc<ConnectionManager>,
    mut stop_recv: Receiver<bool>,
) {
    tokio::spawn(async move {
        loop {
            select! {
                val = stop_recv.recv() =>{
                    if let Err(flag) = val {

                    }
                },
                val = gc_conn(metadata_cache.clone(),connection_manager.clone())=>{
                    sleep(Duration::from_secs(10)).await;
                }
            }
        }
    });
}

async fn gc_conn(metadata_cache: Arc<MetadataCache>, connection_manager: Arc<ConnectionManager>) {
    for raw in connection_manager.get_inactive_conn() {
        // connection_manager.close_conn_by_node(raw.0, &raw.1).await;
    }
}
