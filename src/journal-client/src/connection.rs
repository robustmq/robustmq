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
use log::error;
use protocol::journal_server::codec::{JournalEnginePacket, JournalServerCodec};
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_util::codec::Framed;

use crate::cache::MetadataCache;
use crate::error::JournalClientError;

pub struct ClientConnection {
    pub stream: Framed<TcpStream, JournalServerCodec>,
    pub last_active_time: u64,
}

pub struct NodeConnection {
    node_id: u64,
    metadata_cache: Arc<MetadataCache>,
    connection: DashMap<String, ClientConnection>,
}

impl NodeConnection {
    pub fn new(node_id: u64, metadata_cache: Arc<MetadataCache>) -> Self {
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
        let addr = if let Some(addr) = self.metadata_cache.get_tcp_addr_by_node_id(self.node_id) {
            addr
        } else {
            return Err(JournalClientError::NodeNoAvailableAddr(self.node_id));
        };

        let socket = TcpStream::connect(&addr).await?;
        Ok(Framed::new(socket, JournalServerCodec::new()))
    }

    pub async fn init_conn(&self) -> Result<(), JournalClientError> {
        self.connection.insert(
            "admin".to_string(),
            ClientConnection {
                stream: self.open().await?,
                last_active_time: now_second(),
            },
        );

        self.connection.insert(
            "read".to_string(),
            ClientConnection {
                stream: self.open().await?,
                last_active_time: now_second(),
            },
        );

        self.connection.insert(
            "write".to_string(),
            ClientConnection {
                stream: self.open().await?,
                last_active_time: now_second(),
            },
        );
        Ok(())
    }
}

pub struct ConnectionManager {
    // node_id, NodeConnection
    node_conns: DashMap<u64, NodeConnection>,
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
        let node_id = self.choose_admin_node();

        if !self.node_conns.contains_key(&node_id) {
            let conn = NodeConnection::new(node_id, self.metadata_cache.clone());
            conn.init_conn().await?;
            self.node_conns.insert(node_id, conn);
        }

        let conn = self.node_conns.get(&node_id).unwrap();
        conn.admin_send(req_packet).await
    }

    fn choose_admin_node(&self) -> u64 {
        let node_ids = self.metadata_cache.all_node_ids();
        let posi = self
            .admin_conn_atom
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let index = posi as usize % node_ids.len();
        let node_id = node_ids.get(index).unwrap();
        *node_id
    }
}
