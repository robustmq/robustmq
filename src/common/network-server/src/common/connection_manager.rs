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
use dashmap::DashMap;
use futures::stream::SplitSink;
use futures::SinkExt;
use metadata_struct::connection::{NetworkConnection, NetworkConnectionType};
use protocol::codec::{RobustMQCodec, RobustMQCodecWrapper};
use protocol::mqtt::codec::MqttPacketWrapper;
use protocol::robust::{RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol};
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::codec::FramedWrite;
use tracing::{debug, info};

pub struct ConnectionManager {
    pub connections: DashMap<u64, NetworkConnection>,
    pub lock_max_try_mut_times: i32,
    pub lock_try_mut_sleep_time_ms: u64,
    pub tcp_write_list:
        DashMap<u64, FramedWrite<tokio::io::WriteHalf<tokio::net::TcpStream>, RobustMQCodec>>,
    pub tcp_tls_write_list: DashMap<
        u64,
        FramedWrite<
            tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
            RobustMQCodec,
        >,
    >,
    pub websocket_write_list: DashMap<u64, SplitSink<WebSocket, Message>>,
    pub quic_write_list: DashMap<u64, QuicFramedWriteStream>,
}

impl ConnectionManager {
    pub fn new(lock_max_try_mut_times: i32, lock_try_mut_sleep_time_ms: u64) -> ConnectionManager {
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
            lock_max_try_mut_times,
            lock_try_mut_sleep_time_ms,
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
}

impl ConnectionManager {
    pub async fn write_websocket_frame(
        &self,
        connection_id: u64,
        packet_wrapper: RobustMQPacketWrapper,
        resp: Message,
    ) -> ResultCommonError {
        if !is_ignore_print(&packet_wrapper.packet) {
            info!("WebSockets response packet:{packet_wrapper:?},connection_id:{connection_id}");
        }

        let _network_type = if let Some(connection) = self.get_connect(connection_id) {
            connection.connection_type.to_string()
        } else {
            "".to_string()
        };

        if packet_wrapper.protocol.is_mqtt() {
            match self.write_mqtt_websocket_frame(connection_id, resp).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        if packet_wrapper.protocol.is_kafka() {
            // todo
        }

        Ok(())
    }

    pub async fn write_tcp_frame(
        &self,
        connection_id: u64,
        packet_wrapper: RobustMQPacketWrapper,
    ) -> ResultCommonError {
        if !is_ignore_print(&packet_wrapper.packet) {
            debug!("Tcp response packet:{packet_wrapper:?},connection_id:{connection_id}");
        }

        let _network_type = if let Some(connection) = self.get_connect(connection_id) {
            connection.connection_type.to_string()
        } else {
            "".to_string()
        };

        if packet_wrapper.protocol.is_mqtt() {
            if let RobustMQPacket::MQTT(pack) = packet_wrapper.packet {
                let mqtt_packet = MqttPacketWrapper {
                    protocol_version: packet_wrapper.protocol.to_u8(),
                    packet: pack,
                };
                self.write_mqtt_tcp_frame(connection_id, mqtt_packet)
                    .await?;
            }
        }

        if packet_wrapper.protocol.is_kafka() {
            // todo
        }
        Ok(())
    }

    pub async fn write_quic_frame(
        &self,
        connection_id: u64,
        packet_wrapper: RobustMQPacketWrapper,
    ) -> ResultCommonError {
        if !is_ignore_print(&packet_wrapper.packet) {
            info!("Quic response packet:{packet_wrapper:?},connection_id:{connection_id}");
        }

        let _network_type = if let Some(connection) = self.get_connect(connection_id) {
            connection.connection_type.to_string()
        } else {
            "".to_string()
        };

        if packet_wrapper.protocol.is_mqtt() {
            if let RobustMQPacket::MQTT(pack) = packet_wrapper.packet {
                let mqtt_packet = MqttPacketWrapper {
                    protocol_version: packet_wrapper.protocol.to_u8(),
                    packet: pack,
                };
                self.write_mqtt_quic_frame(connection_id, mqtt_packet)
                    .await?;
            }
        }

        if packet_wrapper.protocol.is_kafka() {
            // todo
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
        self.tcp_write_list.insert(connection_id, write);
    }

    pub fn add_tcp_tls_write(
        &self,
        connection_id: u64,
        write: FramedWrite<
            tokio::io::WriteHalf<tokio_rustls::server::TlsStream<tokio::net::TcpStream>>,
            RobustMQCodec,
        >,
    ) {
        self.tcp_tls_write_list.insert(connection_id, write);
    }

    pub fn add_websocket_write(&self, connection_id: u64, write: SplitSink<WebSocket, Message>) {
        self.websocket_write_list.insert(connection_id, write);
    }

    pub fn add_mqtt_quic_write(
        &self,
        connection_id: u64,
        quic_framed_write_stream: QuicFramedWriteStream,
    ) {
        self.quic_write_list
            .insert(connection_id, quic_framed_write_stream);
    }

    pub async fn write_mqtt_websocket_frame(
        &self,
        connection_id: u64,
        resp: Message,
    ) -> ResultCommonError {
        let mut times = 0;
        loop {
            match self.websocket_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(resp.clone()).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            if broker_not_available(&e.to_string()) {
                                return Err(CommonError::CommonError(e.to_string()));
                            }
                            if times > self.lock_max_try_mut_times {
                                return Err(CommonError::FailedToWriteClient(
                                    "websocket".to_string(),
                                    e.to_string(),
                                ));
                            }
                        }
                    }
                }

                dashmap::try_result::TryResult::Absent => {
                    if times > self.lock_max_try_mut_times {
                        return Err(CommonError::NotObtainAvailableConnection(
                            "websocket".to_string(),
                            connection_id,
                        ));
                    }
                }

                dashmap::try_result::TryResult::Locked => {}
            }
            times += 1;
            sleep(Duration::from_millis(self.lock_try_mut_sleep_time_ms)).await
        }
        Ok(())
    }

    pub async fn write_mqtt_tcp_frame(
        &self,
        connection_id: u64,
        resp: MqttPacketWrapper,
    ) -> ResultCommonError {
        if let Some(connection) = self.get_connect(connection_id) {
            if connection.connection_type == NetworkConnectionType::Tls {
                return self.write_mqtt_tcp_tls_frame(connection_id, resp).await;
            }
        }

        let mut times = 0;
        loop {
            match self.tcp_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(RobustMQCodecWrapper::MQTT(resp.clone())).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            if broker_not_available(&e.to_string()) {
                                return Err(CommonError::CommonError(e.to_string()));
                            }

                            if times > self.lock_max_try_mut_times {
                                return Err(CommonError::FailedToWriteClient(
                                    "tcp".to_string(),
                                    e.to_string(),
                                ));
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > self.lock_max_try_mut_times {
                        return Err(CommonError::NotObtainAvailableConnection(
                            "tcp".to_string(),
                            connection_id,
                        ));
                    }
                }
                dashmap::try_result::TryResult::Locked => {}
            }
            times += 1;
            sleep(Duration::from_millis(self.lock_try_mut_sleep_time_ms)).await
        }
        Ok(())
    }

    async fn write_mqtt_tcp_tls_frame(
        &self,
        connection_id: u64,
        resp: MqttPacketWrapper,
    ) -> ResultCommonError {
        let mut times = 0;
        loop {
            match self.tcp_tls_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da.send(RobustMQCodecWrapper::MQTT(resp.clone())).await {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            if times > self.lock_max_try_mut_times {
                                return Err(CommonError::FailedToWriteClient(
                                    "tls".to_string(),
                                    e.to_string(),
                                ));
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > self.lock_max_try_mut_times {
                        return Err(CommonError::NotObtainAvailableConnection(
                            "tcp".to_string(),
                            connection_id,
                        ));
                    }
                }
                dashmap::try_result::TryResult::Locked => {}
            }
            times += 1;
            sleep(Duration::from_millis(self.lock_try_mut_sleep_time_ms)).await
        }
        Ok(())
    }

    async fn write_mqtt_quic_frame(
        &self,
        connection_id: u64,
        resp: MqttPacketWrapper,
    ) -> ResultCommonError {
        let mut times = 0;
        loop {
            match self.quic_write_list.try_get_mut(&connection_id) {
                dashmap::try_result::TryResult::Present(mut da) => {
                    match da
                        .send(RobustMQCodecWrapper::MQTT(resp.clone()).clone())
                        .await
                    {
                        Ok(_) => {
                            break;
                        }
                        Err(e) => {
                            if times > self.lock_max_try_mut_times {
                                return Err(CommonError::FailedToWriteClient(
                                    "quic".to_string(),
                                    e.to_string(),
                                ));
                            }
                        }
                    }
                }
                dashmap::try_result::TryResult::Absent => {
                    if times > self.lock_max_try_mut_times {
                        return Err(CommonError::NotObtainAvailableConnection(
                            "quic".to_string(),
                            connection_id,
                        ));
                    }
                }
                dashmap::try_result::TryResult::Locked => {}
            }
            times += 1;
            sleep(Duration::from_millis(self.lock_try_mut_sleep_time_ms)).await
        }
        Ok(())
    }

    pub fn set_mqtt_connect_protocol(&self, connect_id: u64, protocol: u8) {
        if let Some(mut connect) = self.connections.get_mut(&connect_id) {
            match protocol {
                3 => connect.set_protocol(RobustMQProtocol::MQTT3),
                4 => connect.set_protocol(RobustMQProtocol::MQTT4),
                5 => connect.set_protocol(RobustMQProtocol::MQTT5),
                _ => {}
            };
        }
    }
}
