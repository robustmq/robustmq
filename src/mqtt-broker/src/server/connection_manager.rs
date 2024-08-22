// Copyright 2023 RobustMQ Team
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

use crate::handler::cache_manager::CacheManager;
use axum::extract::ws::{Message, WebSocket};
use common_base::errors::RobustMQError;
use dashmap::DashMap;
use futures::{stream::SplitSink, SinkExt};
use log::{error, info};
use protocol::mqtt::{
    codec::{MQTTPacketWrapper, MqttCodec},
    common::MQTTProtocol,
};
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use tokio_util::codec::FramedWrite;

use super::connection::{NetworkConnection, NetworkConnectionType};

pub struct ConnectionManager {
    connections: DashMap<u64, NetworkConnection>,
    tcp_write_list:
        DashMap<u64, FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, MqttCodec>>,
    tcp_tls_write_list: DashMap<
        u64,
        FramedWrite<
            tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
            MqttCodec,
        >,
    >,
    websocket_write_list: DashMap<u64, SplitSink<WebSocket, Message>>,
    cache_manager: Arc<CacheManager>,
}

impl ConnectionManager {
    pub fn new(cache_manager: Arc<CacheManager>) -> ConnectionManager {
        let connections = DashMap::with_capacity(64);
        let tcp_write_list = DashMap::with_capacity(64);
        let tcp_tls_write_list = DashMap::with_capacity(64);
        let websocket_write_list = DashMap::with_capacity(64);
        ConnectionManager {
            connections,
            tcp_write_list,
            tcp_tls_write_list,
            cache_manager,
            websocket_write_list,
        }
    }

    pub fn add_connection(&self, connection: NetworkConnection) -> u64 {
        let connection_id = connection.connection_id();
        self.connections.insert(connection_id, connection);
        return connection_id;
    }

    pub fn add_tcp_write(
        &self,
        connection_id: u64,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, MqttCodec>,
    ) {
        self.tcp_write_list.insert(connection_id, write);
    }

    pub fn add_tcp_tls_write(
        &self,
        connection_id: u64,
        write: FramedWrite<
            tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
            MqttCodec,
        >,
    ) {
        self.tcp_tls_write_list.insert(connection_id, write);
    }

    pub fn add_websocket_write(&self, connection_id: u64, write: SplitSink<WebSocket, Message>) {
        self.websocket_write_list.insert(connection_id, write);
    }

    pub async fn close_all_connect(&self) {
        for (connect_id, _) in self.connections.clone() {
            self.clonse_connect(connect_id).await;
        }
    }

    pub async fn clonse_connect(&self, connection_id: u64) {
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

        if let Some((id, mut stream)) = self.websocket_write_list.remove(&connection_id) {
            match stream.close().await {
                Ok(_) => {
                    info!(
                        "server closes the websocket connection actively, connection id [{}]",
                        id
                    );
                }
                Err(e) => error!("{}", e),
            }
        }
    }

    pub async fn write_websocket_frame(
        &self,
        connection_id: u64,
        resp: Message,
    ) -> Result<(), RobustMQError> {
        let mut times = 0;
        let cluster = self.cache_manager.get_cluster_info();
        loop {
            match self.websocket_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            if times > cluster.send_max_try_mut_times {
                                return Err(RobustMQError::CommmonError(format!(
                                    "Failed to write data to the mqtt websocket client, error message: {:?}",
                                    e
                                )));
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > cluster.send_max_try_mut_times {
                        return Err(RobustMQError::CommmonError(format!(
                            "[write_frame]Connection management could not obtain an available websocket connection. Connection ID: {},len:{}",
                            connection_id,
                            self.tcp_write_list.len()
                        )
                    ));
                    }
                }
                dashmap::try_result::TryResult::Locked => {
                    if times > cluster.send_max_try_mut_times {
                        return Err(RobustMQError::CommmonError(
                            format!("[write_frame]Connection management failed to get websocket connection variable reference, connection ID: {connection_id}")
                        ));
                    }
                }
            }
            times = times + 1;
            sleep(Duration::from_millis(cluster.send_try_mut_sleep_time_ms)).await
        }
        return Ok(());
    }

    pub async fn write_tcp_frame(
        &self,
        connection_id: u64,
        resp: MQTTPacketWrapper,
    ) -> Result<(), RobustMQError> {
        info!("response packet:{resp:?},connection_id:{connection_id}");

        // write tls stream
        if let Some(connection) = self.get_connect(connection_id) {
            if connection.connection_type == NetworkConnectionType::TCPS {
                return self.write_tcp_tls_frame(connection_id, resp).await;
            }
        }

        let mut times = 0;
        let cluster = self.cache_manager.get_cluster_info();
        loop {
            match self.tcp_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            if times > cluster.send_max_try_mut_times {
                                return Err(RobustMQError::CommmonError(format!(
                                    "Failed to write data to the mqtt tcp client, error message: {e:?}"
                                )));
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > cluster.send_max_try_mut_times {
                        return Err(RobustMQError::CommmonError(
                            format!(
                                "[write_frame]Connection management could not obtain an available tcp connection. Connection ID: {},len:{}",
                                connection_id,
                                self.tcp_write_list.len()
                            )
                        ));
                    }
                }
                dashmap::try_result::TryResult::Locked => {
                    if times > cluster.send_max_try_mut_times {
                        return Err(RobustMQError::CommmonError(
                            format!(
                                "[write_frame]Connection management failed to get tcp connection variable reference, connection ID: {}",connection_id
                            )
                        ));
                    }
                }
            }
            times = times + 1;
            sleep(Duration::from_millis(cluster.send_try_mut_sleep_time_ms)).await
        }
        return Ok(());
    }

    pub async fn write_tcp_tls_frame(
        &self,
        connection_id: u64,
        resp: MQTTPacketWrapper,
    ) -> Result<(), RobustMQError> {
        let mut times = 0;
        let cluster = self.cache_manager.get_cluster_info();
        loop {
            match self.tcp_tls_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            if times > cluster.send_max_try_mut_times {
                                return Err(RobustMQError::CommmonError(format!(
                                    "Failed to write data to the mqtt tcp client, error message: {e:?}"
                                )));
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > cluster.send_max_try_mut_times {
                        return Err(RobustMQError::CommmonError(
                            format!(
                                "[write_frame]Connection management could not obtain an available tcp connection. Connection ID: {},len:{}",
                                connection_id,
                                self.tcp_write_list.len()
                            )
                        ));
                    }
                }
                dashmap::try_result::TryResult::Locked => {
                    if times > cluster.send_max_try_mut_times {
                        return Err(RobustMQError::CommmonError(
                            format!(
                                "[write_frame]Connection management failed to get tcp connection variable reference, connection ID: {}",connection_id
                            )
                        ));
                    }
                }
            }
            times = times + 1;
            sleep(Duration::from_millis(cluster.send_try_mut_sleep_time_ms)).await
        }
        return Ok(());
    }

    pub fn tcp_connect_num_check(&self) -> bool {
        let cluster = self.cache_manager.get_cluster_info();
        if self.connections.len() >= cluster.tcp_max_connection_num as usize {
            return true;
        }
        return false;
    }

    pub fn get_connect(&self, connect_id: u64) -> Option<NetworkConnection> {
        if let Some(connec) = self.connections.get(&connect_id) {
            return Some(connec.clone());
        }
        return None;
    }

    pub fn get_connect_protocol(&self, connect_id: u64) -> Option<MQTTProtocol> {
        if let Some(connec) = self.connections.get(&connect_id) {
            return connec.protocol.clone();
        }
        return None;
    }

    pub fn set_connect_protocol(&self, connect_id: u64, protocol: u8) {
        if let Some(mut connec) = self.connections.get_mut(&connect_id) {
            match protocol {
                3 => connec.set_protocol(MQTTProtocol::MQTT3),
                4 => connec.set_protocol(MQTTProtocol::MQTT4),
                5 => connec.set_protocol(MQTTProtocol::MQTT5),
                _ => {}
            };
        }
    }

    pub fn is_websocket(&self, connect_id: u64) -> bool {
        if let Some(connec) = self.connections.get(&connect_id) {
            return connec.connection_type == NetworkConnectionType::WebSocket;
        }
        return false;
    }
}
