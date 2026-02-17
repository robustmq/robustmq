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

use crate::common::connection_manager::ConnectionManager;
use crate::common::metric::SLOW_REQUEST_THRESHOLD_MS;
use crate::common::packet::{build_mqtt_packet_wrapper, ResponsePackage};
use crate::{command::ArcCommandAdapter, common::channel::RequestChannel};
use axum::extract::ws::Message;
use bytes::BytesMut;
use common_base::error::client_unavailable_error_by_str;
use common_base::tools::now_millis;
use common_metrics::network::{
    metrics_handler_apply_ms, metrics_handler_queue_state, metrics_handler_queue_wait_ms,
    metrics_handler_request_count, metrics_handler_slow_request_count, metrics_handler_total_ms,
    metrics_handler_write_ms,
};
use metadata_struct::connection::NetworkConnectionType;
use protocol::codec::{RobustMQCodec, RobustMQCodecWrapper};
use protocol::mqtt::codec::MqttPacketWrapper;
use protocol::robust::{
    RobustMQPacket, RobustMQPacketWrapper, RobustMQWrapperExtend, StorageEngineWrapperExtend,
};
use std::sync::Arc;
use tokio::select;
use tokio::sync::broadcast;
use tracing::{debug, error, warn};

pub fn handler_process(
    handler_process_num: usize,
    connection_manager: Arc<ConnectionManager>,
    command: ArcCommandAdapter,
    request_channel: Arc<RequestChannel>,
    network_type: NetworkConnectionType,
    stop_sx: broadcast::Sender<bool>,
) {
    for index in 1..=handler_process_num {
        let raw_connect_manager = connection_manager.clone();
        let raw_command = command.clone();
        let mut raw_stop_rx = stop_sx.subscribe();
        let raw_network_type = network_type.clone();
        let receiver = request_channel.receiver.clone();
        let channel_size = request_channel.channel_size;

        tokio::spawn(async move {
            debug!(
                "Server handler process thread {} start successfully.",
                index
            );
            loop {
                select! {
                    val = raw_stop_rx.recv() =>{
                        match val {
                            Ok(true) => {
                                debug!("Server handler process thread {} stopped successfully.",index);
                                break;
                            }
                            Ok(false) => {}
                            Err(broadcast::error::RecvError::Closed) => {
                                debug!(
                                    "Server handler process thread {} stop channel closed, exiting.",
                                    index
                                );
                                break;
                            }
                            Err(broadcast::error::RecvError::Lagged(skipped)) => {
                                debug!(
                                    "Server handler process thread {} lagged on stop channel, skipped {} messages.",
                                    index, skipped
                                );
                            }
                        }
                    },
                    val = receiver.recv() =>{
                        let dequeue_ms = now_millis();

                        metrics_handler_queue_state(receiver.len(), channel_size);

                        match val {
                            Ok(packet) => {
                                let queue_wait_ms = dequeue_ms.saturating_sub(packet.receive_ms);
                                metrics_handler_queue_wait_ms(&raw_network_type, queue_wait_ms as f64);

                                if let Some(connect) = raw_connect_manager.get_connect(packet.connection_id) {
                                    // apply
                                    let apply_start = now_millis();
                                    let response_data = raw_command
                                        .apply(&connect, &packet.addr, &packet.packet)
                                        .await;
                                    let apply_ms = now_millis().saturating_sub(apply_start);
                                    metrics_handler_apply_ms(&raw_network_type, apply_ms as f64);

                                    // write response
                                    if let Some(resp) = response_data {
                                        let write_start = now_millis();
                                        write_response(&raw_connect_manager, &raw_network_type, &resp).await;
                                        let write_ms = now_millis().saturating_sub(write_start);
                                        metrics_handler_write_ms(&raw_network_type, write_ms as f64);
                                    }

                                    // total
                                    let total_ms = now_millis().saturating_sub(packet.receive_ms);
                                    metrics_handler_total_ms(&raw_network_type, total_ms as f64);
                                    metrics_handler_request_count(&raw_network_type);

                                    if total_ms >= SLOW_REQUEST_THRESHOLD_MS {
                                        warn!(
                                            connection_id = packet.connection_id,
                                            addr = %packet.addr,
                                            total_ms = total_ms,
                                            queue_wait_ms = queue_wait_ms,
                                            apply_ms = apply_ms,
                                            "Slow request detected"
                                        );
                                        metrics_handler_slow_request_count(&raw_network_type);
                                    }
                                } else {
                                    debug!(
                                        "Skip request: connection {} already closed, handler={}, addr={}, network={:?}",
                                        packet.connection_id,
                                        index,
                                        packet.addr,
                                        raw_network_type
                                    );
                                }
                            }
                            Err(_) => {
                                debug!(
                                    "Server handler process thread {} request channel closed, exiting.",
                                    index
                                );
                                break;
                            }
                        }
                    }
                }
            }
        });
    }
}

async fn write_response(
    connection_manager: &Arc<ConnectionManager>,
    network_type: &NetworkConnectionType,
    response_package: &ResponsePackage,
) {
    if let Some(protocol) = connection_manager.get_connect_protocol(response_package.connection_id)
    {
        let packet_wrapper = match response_package.packet.clone() {
            RobustMQPacket::MQTT(packet) => build_mqtt_packet_wrapper(protocol.clone(), packet),
            RobustMQPacket::KAFKA(_packet) => {
                return;
            }
            RobustMQPacket::StorageEngine(packet) => RobustMQPacketWrapper {
                protocol: protocol.clone(),
                extend: RobustMQWrapperExtend::StorageEngine(StorageEngineWrapperExtend {}),
                packet: RobustMQPacket::StorageEngine(packet),
            },
        };

        match network_type.clone() {
            NetworkConnectionType::Tcp | NetworkConnectionType::Tls => {
                if let Err(e) = connection_manager
                    .write_tcp_frame(response_package.connection_id, packet_wrapper)
                    .await
                {
                    if client_unavailable_error_by_str(&e.to_string()) {
                        return;
                    }
                    error!("{}", e);
                };
            }
            NetworkConnectionType::WebSocket | NetworkConnectionType::WebSockets => {
                if let Err(e) = write_websocket_response(
                    connection_manager,
                    response_package.connection_id,
                    &packet_wrapper,
                    &protocol,
                )
                .await
                {
                    if client_unavailable_error_by_str(&e.to_string()) {
                        return;
                    }
                    error!("{}", e);
                };
            }
            NetworkConnectionType::QUIC => {
                if let Err(e) = connection_manager
                    .write_quic_frame(response_package.connection_id, packet_wrapper)
                    .await
                {
                    if client_unavailable_error_by_str(&e.to_string()) {
                        return;
                    }
                    error!("{}", e);
                };
            }
        }
    }
}

async fn write_websocket_response(
    connection_manager: &Arc<ConnectionManager>,
    connection_id: u64,
    packet_wrapper: &RobustMQPacketWrapper,
    protocol: &protocol::robust::RobustMQProtocol,
) -> common_base::error::ResultCommonError {
    let mut codec = RobustMQCodec::new();
    let mut response_buf = BytesMut::new();

    let codec_wrapper = match packet_wrapper.packet.clone() {
        RobustMQPacket::MQTT(pkg) => RobustMQCodecWrapper::MQTT(MqttPacketWrapper {
            protocol_version: protocol.to_u8(),
            packet: pkg,
        }),
        RobustMQPacket::StorageEngine(pkg) => RobustMQCodecWrapper::StorageEngine(pkg),
        RobustMQPacket::KAFKA(_) => {
            return Ok(());
        }
    };

    codec.encode_data(codec_wrapper, &mut response_buf)?;

    connection_manager
        .write_websocket_frame(
            connection_id,
            packet_wrapper.clone(),
            Message::Binary(response_buf.to_vec().into()),
        )
        .await
}
