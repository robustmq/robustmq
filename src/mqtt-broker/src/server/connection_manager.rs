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

use std::sync::Arc;
use std::time::Duration;

use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures::stream::SplitSink;
use futures::SinkExt;
use protocol::mqtt::codec::{MqttCodec, MqttPacketWrapper};
use protocol::mqtt::common::MqttProtocol;
use tokio::time::sleep;
use tokio_util::codec::FramedWrite;
use tracing::{debug, info};

use super::connection::{NetworkConnection, NetworkConnectionType};
use crate::handler::cache::CacheManager;
use crate::handler::error::MqttBrokerError;
use crate::observability::metrics::packets::record_sent_metrics;
use crate::server::quic::quic_stream_wrapper::QuicFramedWriteStream;

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
    quic_write_list: DashMap<u64, QuicFramedWriteStream>,
    cache_manager: Arc<CacheManager>,
}

impl ConnectionManager {
    pub fn new(cache_manager: Arc<CacheManager>) -> ConnectionManager {
        let connections = DashMap::with_capacity(64);
        let tcp_write_list = DashMap::with_capacity(64);
        let tcp_tls_write_list = DashMap::with_capacity(64);
        let websocket_write_list = DashMap::with_capacity(64);
        let quic_write_list = DashMap::with_capacity(64);
        ConnectionManager {
            connections,
            tcp_write_list,
            tcp_tls_write_list,
            cache_manager,
            websocket_write_list,
            quic_write_list,
        }
    }

    pub fn add_connection(&self, connection: NetworkConnection) -> u64 {
        let connection_id = connection.connection_id();
        self.connections.insert(connection_id, connection);
        connection_id
    }

    pub fn list_connect(&self) -> DashMap<u64, NetworkConnection> {
        self.connections.clone()
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

    pub fn add_quic_write(
        &self,
        connection_id: u64,
        quic_framed_write_stream: QuicFramedWriteStream,
    ) {
        self.quic_write_list
            .insert(connection_id, quic_framed_write_stream);
    }

    pub async fn close_all_connect(&self) {
        for (connect_id, _) in self.connections.clone() {
            self.close_connect(connect_id).await;
        }
    }

    pub async fn close_connect(&self, connection_id: u64) {
        if let Some((_, connection)) = self.connections.remove(&connection_id) {
            connection.stop_connection().await;
        }

        if let Some((id, mut stream)) = self.tcp_write_list.remove(&connection_id) {
            if stream.close().await.is_ok() {
                debug!(
                    "server closes the tcp connection actively, connection id [{}]",
                    id
                );
            }
        }

        if let Some((id, mut stream)) = self.tcp_tls_write_list.remove(&connection_id) {
            if stream.close().await.is_ok() {
                debug!(
                    "server closes the tcp connection actively, connection id [{}]",
                    id
                );
            }
        }

        if let Some((id, mut stream)) = self.websocket_write_list.remove(&connection_id) {
            if stream.close().await.is_ok() {
                debug!(
                    "server closes the websocket connection actively, connection id [{}]",
                    id
                );
            }
        }
    }

    pub async fn write_websocket_frame(
        &self,
        connection_id: u64,
        packet_wrapper: MqttPacketWrapper,
        resp: Message,
    ) -> Result<(), MqttBrokerError> {
        info!("WebSockets response packet:{resp:?},connection_id:{connection_id}");

        let mut times = 0;
        let cluster = self.cache_manager.get_cluster_config();
        loop {
            match self.websocket_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            let network_type =
                                if let Some(connection) = self.get_connect(connection_id) {
                                    connection.connection_type.to_string()
                                } else {
                                    "".to_string()
                                };

                            record_sent_metrics(&packet_wrapper, network_type);
                            break;
                        }
                        Err(e) => {
                            if e.to_string().contains("Broken pipe") {
                                break;
                            }
                            if times > cluster.network_thread.lock_max_try_mut_times {
                                return Err(MqttBrokerError::FailedToWriteClient(
                                    "websocket".to_string(),
                                    e.to_string(),
                                ));
                            }
                        }
                    }
                }

                dashmap::try_result::TryResult::Absent => {
                    if times > cluster.network_thread.lock_max_try_mut_times {
                        return Err(MqttBrokerError::NotObtainAvailableConnection(
                            "websocket".to_string(),
                            connection_id,
                        ));
                    }
                }

                dashmap::try_result::TryResult::Locked => {}
            }
            times += 1;
            sleep(Duration::from_millis(
                cluster.network_thread.lock_try_mut_sleep_time_ms,
            ))
            .await
        }
        Ok(())
    }

    pub async fn write_tcp_frame(
        &self,
        connection_id: u64,
        resp: MqttPacketWrapper,
    ) -> Result<(), MqttBrokerError> {
        info!("Tcp response packet:{resp:?},connection_id:{connection_id}");

        if let Some(connection) = self.get_connect(connection_id) {
            if connection.connection_type == NetworkConnectionType::Tls {
                return self.write_tcp_tls_frame(connection_id, resp).await;
            }
        }

        let mut times = 0;
        let cluster = self.cache_manager.get_cluster_config();
        loop {
            match self.tcp_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            // write tls stream
                            let network_type =
                                if let Some(connection) = self.get_connect(connection_id) {
                                    connection.connection_type.to_string()
                                } else {
                                    "".to_string()
                                };

                            record_sent_metrics(&resp, network_type);
                            break;
                        }
                        Err(e) => {
                            if e.to_string().contains("Broken pipe") {
                                break;
                            }
                            if times > cluster.network_thread.lock_max_try_mut_times {
                                return Err(MqttBrokerError::FailedToWriteClient(
                                    "tcp".to_string(),
                                    e.to_string(),
                                ));
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > cluster.network_thread.lock_max_try_mut_times {
                        return Err(MqttBrokerError::NotObtainAvailableConnection(
                            "tcp".to_string(),
                            connection_id,
                        ));
                    }
                }
                dashmap::try_result::TryResult::Locked => {}
            }
            times += 1;
            sleep(Duration::from_millis(
                cluster.network_thread.lock_try_mut_sleep_time_ms,
            ))
            .await
        }
        Ok(())
    }

    async fn write_tcp_tls_frame(
        &self,
        connection_id: u64,
        resp: MqttPacketWrapper,
    ) -> Result<(), MqttBrokerError> {
        let mut times = 0;
        let cluster = self.cache_manager.get_cluster_config();
        loop {
            match self.tcp_tls_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            let network_type =
                                if let Some(connection) = self.get_connect(connection_id) {
                                    connection.connection_type.to_string()
                                } else {
                                    "".to_string()
                                };

                            record_sent_metrics(&resp, network_type);
                            break;
                        }
                        Err(e) => {
                            if times > cluster.network_thread.lock_max_try_mut_times {
                                return Err(MqttBrokerError::FailedToWriteClient(
                                    "tcp".to_string(),
                                    e.to_string(),
                                ));
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > cluster.network_thread.lock_max_try_mut_times {
                        return Err(MqttBrokerError::NotObtainAvailableConnection(
                            "tcp".to_string(),
                            connection_id,
                        ));
                    }
                }
                dashmap::try_result::TryResult::Locked => {}
            }
            times += 1;
            sleep(Duration::from_millis(
                cluster.network_thread.lock_try_mut_sleep_time_ms,
            ))
            .await
        }
        Ok(())
    }

    pub fn tcp_connect_num_check(&self) -> bool {
        let cluster = self.cache_manager.get_cluster_config();
        if self.connections.len() >= cluster.network_thread.max_connection_num {
            return true;
        }
        false
    }

    pub fn get_connect(&self, connect_id: u64) -> Option<NetworkConnection> {
        if let Some(connect) = self.connections.get(&connect_id) {
            return Some(connect.clone());
        }
        None
    }

    pub fn get_connect_protocol(&self, connect_id: u64) -> Option<MqttProtocol> {
        if let Some(connect) = self.connections.get(&connect_id) {
            return connect.protocol.clone();
        }
        None
    }

    pub fn set_connect_protocol(&self, connect_id: u64, protocol: u8) {
        if let Some(mut connect) = self.connections.get_mut(&connect_id) {
            match protocol {
                3 => connect.set_protocol(MqttProtocol::Mqtt3),
                4 => connect.set_protocol(MqttProtocol::Mqtt4),
                5 => connect.set_protocol(MqttProtocol::Mqtt5),
                _ => {}
            };
        }
    }

    pub fn is_websocket(&self, connect_id: u64) -> bool {
        if let Some(connect) = self.connections.get(&connect_id) {
            return connect.connection_type == NetworkConnectionType::WebSocket;
        }
        false
    }
}
