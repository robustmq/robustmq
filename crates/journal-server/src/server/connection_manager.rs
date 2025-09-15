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

use std::time::Duration;

use common_base::error::common::CommonError;
use dashmap::DashMap;
use futures::SinkExt;
use protocol::journal::codec::{JournalEnginePacket, JournalServerCodec};
use tokio::time::sleep;
use tokio_util::codec::FramedWrite;
use tracing::{debug, error, info};

use super::connection::{NetworkConnection, NetworkConnectionType};

/// a struct that manages all TCP and TLS connections
pub struct ConnectionManager {
    connections: DashMap<u64, NetworkConnection>,
    tcp_write_list:
        DashMap<u64, FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, JournalServerCodec>>,
    tcp_tls_write_list: DashMap<
        u64,
        FramedWrite<
            tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
            JournalServerCodec,
        >,
    >,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> ConnectionManager {
        let connections = DashMap::with_capacity(64);
        let tcp_write_list: DashMap<
            u64,
            FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, JournalServerCodec>,
        > = DashMap::with_capacity(64);
        let tcp_tls_write_list = DashMap::with_capacity(64);
        ConnectionManager {
            connections,
            tcp_write_list,
            tcp_tls_write_list,
        }
    }

    pub fn add_connection(&self, connection: NetworkConnection) -> u64 {
        let connection_id = connection.connection_id();
        self.connections.insert(connection_id, connection);
        connection_id
    }

    pub fn add_tcp_write(
        &self,
        connection_id: u64,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, JournalServerCodec>,
    ) {
        self.tcp_write_list.insert(connection_id, write);
    }

    pub async fn _close_all_connect(&self) {
        for (connect_id, _) in self.connections.clone() {
            self.close_connect(connect_id).await;
        }
    }

    /// stop a connection, note that we first stops all the connection threads, and then close the tcp stream
    pub async fn close_connect(&self, connection_id: u64) {
        if let Some((_, connection)) = self.connections.remove(&connection_id) {
            connection.stop_connection().await;
        }

        if let Some((id, mut stream)) = self.tcp_write_list.remove(&connection_id) {
            match stream.close().await {
                Ok(_) => {
                    info!(
                        "server closes the tcp connection actively, connection id [{}]",
                        id
                    );
                }
                Err(e) => error!("{}", e),
            }
        }

        if let Some((id, mut stream)) = self.tcp_tls_write_list.remove(&connection_id) {
            match stream.close().await {
                Ok(_) => {
                    info!(
                        "server closes the tcp connection actively, connection id [{}]",
                        id
                    );
                }
                Err(e) => error!("{}", e),
            }
        }
    }

    /// write response packet to the corresponding tcp write stream identified by `connection_id`
    pub async fn write_tcp_frame(
        &self,
        connection_id: u64,
        resp: JournalEnginePacket,
    ) -> Result<(), CommonError> {
        debug!("response packet:{resp:?},connection_id:{connection_id}");

        // write tls stream
        if let Some(connection) = self.get_connect(connection_id) {
            if connection.connection_type == NetworkConnectionType::Tls {
                return self.write_tcp_tls_frame(connection_id, resp).await;
            }
        }

        let mut times = 0;
        let response_max_try_mut_times = 5;
        let response_try_mut_sleep_time_ms = 1000;
        loop {
            match self.tcp_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            if times > response_max_try_mut_times {
                                return Err(CommonError::CommonError(format!(
                                    "Failed to write data to the Journal engine client, error message: {e:?}"
                                )));
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > response_max_try_mut_times {
                        return Err(CommonError::CommonError(
                            format!(
                                "[write_frame]Connection management could not obtain an available tcp connection. Connection ID: {},len:{}",
                                connection_id,
                                self.tcp_write_list.len()
                            )
                        ));
                    }
                }
                dashmap::try_result::TryResult::Locked => {
                    if times > response_max_try_mut_times {
                        return Err(CommonError::CommonError(
                            format!(
                                "[write_frame]Connection management failed to get tcp connection variable reference, connection ID: {connection_id}"
                            )
                        ));
                    }
                }
            }
            times += 1;
            sleep(Duration::from_millis(response_try_mut_sleep_time_ms)).await
        }
        Ok(())
    }

    /// similar to [`write_tcp_frame`], but for TLS connections
    async fn write_tcp_tls_frame(
        &self,
        connection_id: u64,
        resp: JournalEnginePacket,
    ) -> Result<(), CommonError> {
        let mut times = 0;
        let response_max_try_mut_times = 5;
        let response_try_mut_sleep_time_ms = 1000;
        loop {
            match self.tcp_tls_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            if times > response_max_try_mut_times {
                                return Err(CommonError::CommonError(format!(
                                    "Failed to write data to the journal engine client, error message: {e:?}"
                                )));
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > response_max_try_mut_times {
                        return Err(CommonError::CommonError(
                            format!(
                                "[write_frame]Connection management could not obtain an available tcp connection. Connection ID: {},len:{}",
                                connection_id,
                                self.tcp_write_list.len()
                            )
                        ));
                    }
                }
                dashmap::try_result::TryResult::Locked => {
                    if times > response_max_try_mut_times {
                        return Err(CommonError::CommonError(
                            format!(
                                "[write_frame]Connection management failed to get tcp connection variable reference, connection ID: {connection_id}"
                            )
                        ));
                    }
                }
            }
            times += 1;
            sleep(Duration::from_millis(response_try_mut_sleep_time_ms)).await
        }
        Ok(())
    }

    pub fn _tcp_connect_num_check(&self) -> bool {
        if self.connections.len() >= 10000 {
            return true;
        }
        false
    }

    pub fn get_connect(&self, connect_id: u64) -> Option<NetworkConnection> {
        if let Some(connec) = self.connections.get(&connect_id) {
            return Some(connec.clone());
        }
        None
    }
}
