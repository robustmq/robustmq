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
use dashmap::DashMap;
use futures::{SinkExt, StreamExt};
use protocol::storage::codec::{StorageEngineCodec, StorageEnginePacket};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::sleep;
use tokio_util::codec::Framed;
use tracing::error;

pub(crate) const MODULE_WRITE: &str = "write";
pub(crate) const MODULE_READ: &str = "read";

pub struct ClientConnection {
    pub stream: Framed<TcpStream, StorageEngineCodec>,
    pub last_active_time: u64,
}

pub struct NodeConnection {
    pub node_id: u64,
    pub cache_manager: Arc<StorageCacheManager>,
    pub connection: DashMap<String, ClientConnection>,
}

impl NodeConnection {
    pub fn new(node_id: u64, cache_manager: Arc<StorageCacheManager>) -> Self {
        let connection = DashMap::with_capacity(2);
        NodeConnection {
            node_id,
            cache_manager,
            connection,
        }
    }

    pub async fn write_send(
        &self,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        self.send("write", req_packet).await
    }

    pub async fn read_send(
        &self,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
        self.send("read", req_packet).await
    }

    async fn send(
        &self,
        module: &str,
        req_packet: StorageEnginePacket,
    ) -> Result<StorageEnginePacket, StorageEngineError> {
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
                                        return Err(StorageEngineError::ReceivedPacketError(
                                            self.node_id,
                                            e.to_string(),
                                        ));
                                    }
                                }
                            } else {
                                return Err(StorageEngineError::ReceivedPacketIsEmpty(
                                    self.node_id,
                                ));
                            }
                        }
                        Err(e) => {
                            if times > response_max_try_mut_times {
                                return Err(StorageEngineError::SendRequestError(
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
                        return Err(StorageEngineError::NoAvailableConn(self.node_id));
                    }
                }
                dashmap::try_result::TryResult::Locked => {
                    if times > response_max_try_mut_times {
                        return Err(StorageEngineError::ConnectionIsOccupied(self.node_id));
                    }
                }
            }
            times += 1;
            sleep(Duration::from_millis(response_try_mut_sleep_time_ms)).await;
        }
    }

    async fn open(&self) -> Result<Framed<TcpStream, StorageEngineCodec>, StorageEngineError> {
        let Some(node) = self
            .cache_manager
            .broker_cache
            .node_lists
            .get(&(self.node_id as u64))
        else {
            return Err(StorageEngineError::NoAvailableConn(self.node_id));
        };
        let addr = node.node_inner_addr.clone();
        let socket = TcpStream::connect(&addr).await?;
        Ok(Framed::new(socket, StorageEngineCodec::new()))
    }

    pub async fn init_conn(&self, module: &str) -> Result<(), StorageEngineError> {
        match module {
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
