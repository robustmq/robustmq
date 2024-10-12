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
use std::sync::atomic::AtomicU64;
use std::time::Duration;

use dashmap::DashMap;
use futures::SinkExt;
use log::error;
use protocol::journal_server::codec::{StorageEngineCodec, JournalEnginePacket};
use tokio::time::sleep;
use tokio_util::codec::FramedWrite;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Description The number of TCP connections on a node exceeded the upper limit. The maximum number of TCP connections was {total:?}")]
    ConnectionExceed { total: usize },
}

pub struct ConnectionManager {
    connections: DashMap<u64, Connection>,
    max_connection_num: usize,
    max_try_mut_times: u64,
    try_mut_sleep_time_ms: u64,
}

impl ConnectionManager {
    pub fn new(
        max_connection_num: usize,
        max_try_mut_times: u64,
        try_mut_sleep_time_ms: u64,
    ) -> ConnectionManager {
        let connections: DashMap<u64, Connection> =
            DashMap::with_capacity_and_shard_amount(1000, 64);
        ConnectionManager {
            connections,
            max_connection_num,
            max_try_mut_times,
            try_mut_sleep_time_ms,
        }
    }

    pub fn add(&self, connection: Connection) -> u64 {
        let connection_id = connection.connection_id;
        self.connections.insert(connection_id, connection);
        connection_id
    }

    #[allow(dead_code)]
    pub fn remove(&self, connection_id: u64) {
        self.connections.remove(&connection_id);
    }

    pub async fn write_frame(&self, connection_id: u64, resp: JournalEnginePacket) {
        let mut times = 0;
        loop {
            match self.connections.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    return da.write_frame(resp).await;
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > self.max_try_mut_times {
                        error!("[write_frame]Connection management could not obtain an available connection. Connection ID: {}",connection_id);
                        break;
                    }
                }
                dashmap::try_result::TryResult::Locked => {
                    if times > self.max_try_mut_times {
                        error!("[write_frame]Connection management failed to get connection variable reference, connection ID: {}",connection_id);
                        break;
                    }
                }
            }
            times += 1;
            sleep(Duration::from_millis(self.try_mut_sleep_time_ms)).await
        }
    }

    pub fn connect_check(&self) -> Result<(), Error> {
        // Verify the connection limit
        if self.connections.len() > self.max_connection_num {
            return Err(Error::ConnectionExceed {
                total: self.max_connection_num,
            });
        }

        // authentication
        Ok(())
    }
}

static CONNECTION_ID_BUILD: AtomicU64 = AtomicU64::new(1);

#[allow(dead_code)]
pub struct Connection {
    pub connection_id: u64,
    pub addr: SocketAddr,
    pub write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, StorageEngineCodec>,
}

impl Connection {
    pub fn new(
        addr: SocketAddr,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, StorageEngineCodec>,
    ) -> Connection {
        let connection_id = CONNECTION_ID_BUILD.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Connection {
            connection_id,
            addr,
            write,
        }
    }

    pub async fn write_frame(&mut self, resp: JournalEnginePacket) {
        match self.write.send(resp).await {
            Ok(_) => {}
            Err(err) => error!(
                "Failed to write data to the response queue, error message: {:?}",
                err
            ),
        }
    }
}
