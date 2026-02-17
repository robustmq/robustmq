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

use crate::common::tool::is_ignore_print;
use crate::quic::stream::QuicFramedWriteStream;
use axum::extract::ws::{Message, WebSocket};
use common_base::error::{common::CommonError, ResultCommonError};
use common_base::network::broker_not_available;
use common_base::tools::now_second;
use dashmap::DashMap;
use futures::stream::SplitSink;
use futures::SinkExt;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::codec::{RobustMQCodec, RobustMQCodecWrapper};
use protocol::mqtt::codec::MqttPacketWrapper;
use protocol::robust::{RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::codec::FramedWrite;
use tracing::{debug, info};

type TcpWriter =
    Arc<Mutex<FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, RobustMQCodec>>>;
type TcpTlsWriter = Arc<
    Mutex<
        FramedWrite<
            tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
            RobustMQCodec,
        >,
    >,
>;
type WebSocketWriter = Arc<Mutex<SplitSink<WebSocket, Message>>>;
type QuicWriter = Arc<Mutex<QuicFramedWriteStream>>;

#[derive(Clone)]
pub struct ConnectionManager {
    pub connections: DashMap<u64, NetworkConnection>,
    pub tcp_write_list: DashMap<u64, TcpWriter>,
    pub tcp_tls_write_list: DashMap<u64, TcpTlsWriter>,
    pub websocket_write_list: DashMap<u64, WebSocketWriter>,
    pub quic_write_list: DashMap<u64, QuicWriter>,
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnectionManager {
    pub fn new() -> ConnectionManager {
        let connections = DashMap::with_capacity(64);
        let tcp_write_list = DashMap::with_capacity(64);
        let tcp_tls_write_list = DashMap::with_capacity(64);
        let websocket_write_list = DashMap::with_capacity(64);
        let quic_write_list = DashMap::with_capacity(64);
        ConnectionManager {
            connections,
            tcp_write_list,
            tcp_tls_write_list,
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

    pub async fn close_all_connect(&self) {
        for (connect_id, _) in self.connections.clone() {
            self.close_connect(connect_id).await;
        }
    }

    pub async fn close_connect(&self, connection_id: u64) {
        // todo
        // self.connections.remove(&connection_id);

        if let Some((id, writer)) = self.tcp_write_list.remove(&connection_id) {
            let mut stream = writer.lock().await;
            if stream.close().await.is_ok() {
                debug!(
                    "server closes the tcp connection actively, connection id [{}]",
                    id
                );
            }
        }

        if let Some((id, writer)) = self.tcp_tls_write_list.remove(&connection_id) {
            let mut stream = writer.lock().await;
            if stream.close().await.is_ok() {
                debug!(
                    "server closes the tls connection actively, connection id [{}]",
                    id
                );
            }
        }

        if let Some((id, writer)) = self.websocket_write_list.remove(&connection_id) {
            let mut stream = writer.lock().await;
            if stream.close().await.is_ok() {
                debug!(
                    "server closes the websocket connection actively, connection id [{}]",
                    id
                );
            }
        }

        if let Some((id, _writer)) = self.quic_write_list.remove(&connection_id) {
            debug!(
                "server closes the quic connection actively, connection id [{}]",
                id
            );
        }
    }

    pub fn get_connect(&self, connect_id: u64) -> Option<NetworkConnection> {
        if let Some(connect) = self.connections.get(&connect_id) {
            return Some(connect.clone());
        }
        None
    }

    pub fn get_connect_protocol(&self, connect_id: u64) -> Option<RobustMQProtocol> {
        if let Some(connect) = self.connections.get(&connect_id) {
            return connect.protocol.clone();
        }
        None
    }

    pub fn is_websocket(&self, connect_id: u64) -> bool {
        if let Some(connect) = self.connections.get(&connect_id) {
            return connect.connection_type == NetworkConnectionType::WebSocket;
        }
        false
    }

    pub fn is_quic(&self, connect_id: u64) -> bool {
        if let Some(connect) = self.connections.get(&connect_id) {
            return connect.connection_type == NetworkConnectionType::QUIC;
        }
        false
    }

    pub fn get_network_type(&self, connect_id: u64) -> Option<NetworkConnectionType> {
        if let Some(connect) = self.connections.get(&connect_id) {
            return Some(connect.connection_type.clone());
        }
        None
    }

    pub fn get_tcp_connect_num_check(&self) -> u64 {
        0
    }

    pub fn report_heartbeat(&self, connect_id: u64, time: u64) {
        if let Some(mut connect) = self.connections.get_mut(&connect_id) {
            connect.set_heartbeat_time(time);
        }
    }

    pub async fn connection_gc(&self) {
        for conn in self.connections.iter() {
            if now_second() - conn.last_heartbeat_time > 1800 {
                self.close_connect(conn.connection_id).await;
            }
        }
    }
}

impl ConnectionManager {
    pub async fn write_websocket_frame(
        &self,
        connection_id: u64,
        packet_wrapper: RobustMQPacketWrapper,
        resp: Message,
    ) -> ResultCommonError {
        if !is_ignore_print(&packet_wrapper.packet) {
            debug!("WebSockets response packet:{packet_wrapper:?},connection_id:{connection_id}");
        }

        self.write_websocket_frame0(connection_id, resp).await
    }

    pub async fn write_tcp_frame(
        &self,
        connection_id: u64,
        packet_wrapper: RobustMQPacketWrapper,
    ) -> ResultCommonError {
        if !is_ignore_print(&packet_wrapper.packet) {
            info!("Tcp response packet:{packet_wrapper:?},connection_id:{connection_id}");
        }

        if packet_wrapper.protocol.is_mqtt() {
            if let RobustMQPacket::MQTT(pack) = packet_wrapper.packet {
                let mqtt_packet = MqttPacketWrapper {
                    protocol_version: packet_wrapper.protocol.to_u8(),
                    packet: pack,
                };

                self.write_tcp_frame0(connection_id, RobustMQCodecWrapper::MQTT(mqtt_packet))
                    .await?;
                return Ok(());
            }
        }

        if packet_wrapper.protocol.is_kafka() {
            // todo
        }

        if packet_wrapper.protocol.is_engine() {
            if let RobustMQPacket::StorageEngine(pack) = packet_wrapper.packet {
                self.write_tcp_frame0(connection_id, RobustMQCodecWrapper::StorageEngine(pack))
                    .await?;
                return Ok(());
            }
        }
        Ok(())
    }

    pub async fn write_quic_frame(
        &self,
        connection_id: u64,
        packet_wrapper: RobustMQPacketWrapper,
    ) -> ResultCommonError {
        if !is_ignore_print(&packet_wrapper.packet) {
            debug!("QUIC response packet:{packet_wrapper:?},connection_id:{connection_id}");
        }

        if packet_wrapper.protocol.is_mqtt() {
            if let RobustMQPacket::MQTT(pack) = packet_wrapper.packet {
                let mqtt_packet = MqttPacketWrapper {
                    protocol_version: packet_wrapper.protocol.to_u8(),
                    packet: pack,
                };
                self.write_quic_frame0(connection_id, RobustMQCodecWrapper::MQTT(mqtt_packet))
                    .await?;
                return Ok(());
            }
        }

        if packet_wrapper.protocol.is_kafka() {
            // todo
        }

        if packet_wrapper.protocol.is_engine() {
            if let RobustMQPacket::StorageEngine(pack) = packet_wrapper.packet {
                self.write_quic_frame0(connection_id, RobustMQCodecWrapper::StorageEngine(pack))
                    .await?;
                return Ok(());
            }
        }
        Ok(())
    }
}

impl ConnectionManager {
    pub fn add_tcp_write(
        &self,
        connection_id: u64,
        write: FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, RobustMQCodec>,
    ) {
        self.tcp_write_list
            .insert(connection_id, Arc::new(Mutex::new(write)));
    }

    pub fn add_tcp_tls_write(
        &self,
        connection_id: u64,
        write: FramedWrite<
            tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
            RobustMQCodec,
        >,
    ) {
        self.tcp_tls_write_list
            .insert(connection_id, Arc::new(Mutex::new(write)));
    }

    pub fn add_websocket_write(&self, connection_id: u64, write: SplitSink<WebSocket, Message>) {
        self.websocket_write_list
            .insert(connection_id, Arc::new(Mutex::new(write)));
    }

    pub fn add_mqtt_quic_write(
        &self,
        connection_id: u64,
        quic_framed_write_stream: QuicFramedWriteStream,
    ) {
        self.quic_write_list.insert(
            connection_id,
            Arc::new(Mutex::new(quic_framed_write_stream)),
        );
    }

    async fn write_websocket_frame0(&self, connection_id: u64, resp: Message) -> ResultCommonError {
        let writer = self
            .websocket_write_list
            .get(&connection_id)
            .map(|entry| entry.value().clone());

        match writer {
            Some(writer) => {
                let mut stream = writer.lock().await;
                match stream.send(resp).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        if broker_not_available(&e.to_string()) {
                            return Err(CommonError::CommonError(e.to_string()));
                        }
                        self.close_connect(connection_id).await;
                        Err(CommonError::FailedToWriteClient(
                            "websocket".to_string(),
                            e.to_string(),
                        ))
                    }
                }
            }
            None => {
                debug!(
                    "Write to websocket failed: connection {} not found, message: {:?}",
                    connection_id, resp
                );
                Err(CommonError::NotObtainAvailableConnection(
                    "websocket".to_string(),
                    connection_id,
                ))
            }
        }
    }

    async fn write_tcp_frame0(
        &self,
        connection_id: u64,
        resp: RobustMQCodecWrapper,
    ) -> ResultCommonError {
        if let Some(connection) = self.get_connect(connection_id) {
            if connection.connection_type == NetworkConnectionType::Tls {
                return self.write_tcp_tls_frame(connection_id, resp).await;
            }
        }

        let writer = self
            .tcp_write_list
            .get(&connection_id)
            .map(|entry| entry.value().clone());

        match writer {
            Some(writer) => {
                let mut stream = writer.lock().await;
                match stream.send(resp).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        if broker_not_available(&e.to_string()) {
                            return Err(CommonError::CommonError(e.to_string()));
                        }
                        self.close_connect(connection_id).await;
                        Err(CommonError::FailedToWriteClient(
                            "tcp".to_string(),
                            e.to_string(),
                        ))
                    }
                }
            }
            None => {
                debug!(
                    "Write to tcp skipped: connection {} not found, packet: {}",
                    connection_id, resp
                );
                Err(CommonError::NotObtainAvailableConnection(
                    "tcp".to_string(),
                    connection_id,
                ))
            }
        }
    }

    async fn write_tcp_tls_frame(
        &self,
        connection_id: u64,
        resp: RobustMQCodecWrapper,
    ) -> ResultCommonError {
        let writer = self
            .tcp_tls_write_list
            .get(&connection_id)
            .map(|entry| entry.value().clone());

        match writer {
            Some(writer) => {
                let mut stream = writer.lock().await;
                match stream.send(resp).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        self.close_connect(connection_id).await;
                        Err(CommonError::FailedToWriteClient(
                            "tls".to_string(),
                            e.to_string(),
                        ))
                    }
                }
            }
            None => {
                debug!(
                    "Write to tls skipped: connection {} not found, packet: {}",
                    connection_id, resp
                );
                Err(CommonError::NotObtainAvailableConnection(
                    "tls".to_string(),
                    connection_id,
                ))
            }
        }
    }

    async fn write_quic_frame0(
        &self,
        connection_id: u64,
        resp: RobustMQCodecWrapper,
    ) -> ResultCommonError {
        let writer = self
            .quic_write_list
            .get(&connection_id)
            .map(|entry| entry.value().clone());

        match writer {
            Some(writer) => {
                let mut stream = writer.lock().await;
                match stream.send(resp).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        self.close_connect(connection_id).await;
                        Err(CommonError::FailedToWriteClient(
                            "quic".to_string(),
                            e.to_string(),
                        ))
                    }
                }
            }
            None => {
                debug!(
                    "Write to quic skipped: connection {} not found, packet: {}",
                    connection_id, resp
                );
                Err(CommonError::NotObtainAvailableConnection(
                    "quic".to_string(),
                    connection_id,
                ))
            }
        }
    }

    pub fn set_mqtt_connect_protocol(&self, connect_id: u64, protocol: u8) {
        if let Some(mut connect) = self.connections.get_mut(&connect_id) {
            match protocol {
                3 => connect.set_protocol(RobustMQProtocol::MQTT3),
                4 => connect.set_protocol(RobustMQProtocol::MQTT4),
                5 => connect.set_protocol(RobustMQProtocol::MQTT5),
                10 => connect.set_protocol(RobustMQProtocol::KAFKA),
                11 => connect.set_protocol(RobustMQProtocol::StorageEngine),
                _ => {}
            };
        }
    }

    pub fn set_storage_engine_protocol(&self, connect_id: u64) {
        if let Some(mut connect) = self.connections.get_mut(&connect_id) {
            if connect.protocol.is_none() {
                connect.set_protocol(RobustMQProtocol::StorageEngine);
            }
        }
    }
}
