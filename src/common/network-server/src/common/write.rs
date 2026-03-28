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

use super::connection_manager::ConnectionManager;
use crate::common::tool::is_ignore_print;
use axum::extract::ws::Message;
use common_base::error::{common::CommonError, ResultCommonError};
use common_base::network::broker_not_available;
use common_base::tools::now_millis;
use common_metrics::network::{metrics_write_client_ms, metrics_write_timeout_count};
use futures::SinkExt;
use metadata_struct::connection::NetworkConnectionType;
use protocol::codec::RobustMQCodecWrapper;
use protocol::mqtt::codec::MqttPacketWrapper;
use protocol::robust::{RobustMQPacket, RobustMQPacketWrapper};
use std::time::Duration;
use tracing::{debug, info, warn};

const WRITE_TIMEOUT_SECS: u64 = 30;

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

        let codec = match packet_wrapper.packet {
            RobustMQPacket::MQTT(pack) => RobustMQCodecWrapper::MQTT(MqttPacketWrapper {
                protocol_version: packet_wrapper.protocol.to_u8(),
                packet: pack,
            }),
            RobustMQPacket::KAFKA(pack) => RobustMQCodecWrapper::KAFKA(pack),
            RobustMQPacket::AMQP(frame) => RobustMQCodecWrapper::AMQP(frame),
            RobustMQPacket::StorageEngine(pack) => RobustMQCodecWrapper::StorageEngine(pack),
            RobustMQPacket::NATS(pkt) => RobustMQCodecWrapper::NATS(pkt),
        };

        self.write_tcp_frame0(connection_id, codec).await
    }

    pub async fn write_quic_frame(
        &self,
        connection_id: u64,
        packet_wrapper: RobustMQPacketWrapper,
    ) -> ResultCommonError {
        if !is_ignore_print(&packet_wrapper.packet) {
            debug!("QUIC response packet:{packet_wrapper:?},connection_id:{connection_id}");
        }

        let codec = match packet_wrapper.packet {
            RobustMQPacket::MQTT(pack) => RobustMQCodecWrapper::MQTT(MqttPacketWrapper {
                protocol_version: packet_wrapper.protocol.to_u8(),
                packet: pack,
            }),
            RobustMQPacket::KAFKA(pack) => RobustMQCodecWrapper::KAFKA(pack),
            RobustMQPacket::AMQP(frame) => RobustMQCodecWrapper::AMQP(frame),
            RobustMQPacket::StorageEngine(pack) => RobustMQCodecWrapper::StorageEngine(pack),
            RobustMQPacket::NATS(pkt) => RobustMQCodecWrapper::NATS(pkt),
        };

        self.write_quic_frame0(connection_id, codec).await
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
                return self.write_tls_frame0(connection_id, resp).await;
            }
        }

        let writer = self
            .tcp_write_list
            .get(&connection_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| {
                debug!(
                    "Write to tcp skipped: connection {} not found, packet: {}",
                    connection_id, resp
                );
                CommonError::NotObtainAvailableConnection("tcp".to_string(), connection_id)
            })?;

        let mut stream = writer.lock().await;
        let write_start = now_millis();
        let result =
            tokio::time::timeout(Duration::from_secs(WRITE_TIMEOUT_SECS), stream.send(resp)).await;
        metrics_write_client_ms(
            &NetworkConnectionType::Tcp,
            now_millis().saturating_sub(write_start) as f64,
        );

        match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => {
                self.close_connect(connection_id).await;
                Err(CommonError::FailedToWriteClient(
                    "tcp".to_string(),
                    e.to_string(),
                ))
            }
            Err(_) => {
                metrics_write_timeout_count(&NetworkConnectionType::Tcp);
                warn!(
                    connection_id = connection_id,
                    timeout_secs = WRITE_TIMEOUT_SECS,
                    "TCP write timeout: socket send blocked beyond {}s, closing connection",
                    WRITE_TIMEOUT_SECS
                );
                self.close_connect(connection_id).await;
                Err(CommonError::FailedToWriteClient(
                    "tcp".to_string(),
                    format!("write timeout after {WRITE_TIMEOUT_SECS}s"),
                ))
            }
        }
    }

    async fn write_tls_frame0(
        &self,
        connection_id: u64,
        resp: RobustMQCodecWrapper,
    ) -> ResultCommonError {
        let writer = self
            .tcp_tls_write_list
            .get(&connection_id)
            .map(|entry| entry.value().clone())
            .ok_or_else(|| {
                debug!(
                    "Write to tls skipped: connection {} not found, packet: {}",
                    connection_id, resp
                );
                CommonError::NotObtainAvailableConnection("tls".to_string(), connection_id)
            })?;

        let mut stream = writer.lock().await;
        let write_start = now_millis();
        let result =
            tokio::time::timeout(Duration::from_secs(WRITE_TIMEOUT_SECS), stream.send(resp)).await;
        metrics_write_client_ms(
            &NetworkConnectionType::Tls,
            now_millis().saturating_sub(write_start) as f64,
        );

        match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => {
                self.close_connect(connection_id).await;
                Err(CommonError::FailedToWriteClient(
                    "tls".to_string(),
                    e.to_string(),
                ))
            }
            Err(_) => {
                metrics_write_timeout_count(&NetworkConnectionType::Tls);
                warn!(
                    connection_id = connection_id,
                    timeout_secs = WRITE_TIMEOUT_SECS,
                    "TLS write timeout: socket send blocked beyond {}s, closing connection",
                    WRITE_TIMEOUT_SECS
                );
                self.close_connect(connection_id).await;
                Err(CommonError::FailedToWriteClient(
                    "tls".to_string(),
                    format!("write timeout after {WRITE_TIMEOUT_SECS}s"),
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
            .map(|entry| entry.value().clone())
            .ok_or_else(|| {
                debug!(
                    "Write to quic skipped: connection {} not found, packet: {}",
                    connection_id, resp
                );
                CommonError::NotObtainAvailableConnection("quic".to_string(), connection_id)
            })?;

        let mut stream = writer.lock().await;
        let write_start = now_millis();
        let result =
            tokio::time::timeout(Duration::from_secs(WRITE_TIMEOUT_SECS), stream.send(resp)).await;
        metrics_write_client_ms(
            &NetworkConnectionType::QUIC,
            now_millis().saturating_sub(write_start) as f64,
        );

        match result {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => {
                self.close_connect(connection_id).await;
                Err(CommonError::FailedToWriteClient(
                    "quic".to_string(),
                    e.to_string(),
                ))
            }
            Err(_) => {
                metrics_write_timeout_count(&NetworkConnectionType::QUIC);
                warn!(
                    connection_id = connection_id,
                    timeout_secs = WRITE_TIMEOUT_SECS,
                    "QUIC write timeout: socket send blocked beyond {}s, closing connection",
                    WRITE_TIMEOUT_SECS
                );
                self.close_connect(connection_id).await;
                Err(CommonError::FailedToWriteClient(
                    "quic".to_string(),
                    format!("write timeout after {WRITE_TIMEOUT_SECS}s"),
                ))
            }
        }
    }
}
