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
use crate::common::metric::record_packet_handler_info_by_response;
use crate::common::packet::{build_mqtt_packet_wrapper, ResponsePackage};
use crate::{command::ArcCommandAdapter, common::channel::RequestChannel};
use axum::extract::ws::Message;
use bytes::BytesMut;
use common_base::error::client_unavailable_error_by_str;
use common_base::tools::now_millis;
use common_metrics::network::metrics_request_queue_size;
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
                        let out_queue_time = now_millis();
                        record_shared_channel_metrics(index, &receiver, channel_size);
                        match val {
                            Ok(packet) => {
                                if let Some(connect) = raw_connect_manager.get_connect(packet.connection_id) {
                                    let response_data = raw_command
                                        .apply(&connect, &packet.addr, &packet.packet)
                                        .await;

                                    if let Some(mut resp) = response_data {
                                        resp.out_queue_ms = out_queue_time;
                                        resp.receive_ms = packet.receive_ms;
                                        resp.end_handler_ms = now_millis();
                                        process_response(&raw_connect_manager, &raw_network_type, &resp).await;
                                    } else {
                                        debug!("{}","No backpacking is required for this request");
                                    }
                                } else {
                                    warn!(
                                        "Skip request handling because connection is missing. handler_index={}, connection_id={}, addr={}, network_type={:?}",
                                        index,
                                        packet.connection_id,
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

fn record_shared_channel_metrics(
    index: usize,
    receiver: &async_channel::Receiver<crate::common::packet::RequestPackage>,
    channel_size: usize,
) {
    let label = format!("handler-{index}");
    let current_len = receiver.len();
    let remaining = channel_size.saturating_sub(current_len);
    metrics_request_queue_size(&label, current_len, current_len, remaining);
}

async fn process_response(
    connection_manager: &Arc<ConnectionManager>,
    network_type: &NetworkConnectionType,
    response_package: &ResponsePackage,
) {
    let out_response_queue_ms = now_millis();
    if let Some(protocol) = connection_manager.get_connect_protocol(response_package.connection_id)
    {
        let packet_wrapper = match response_package.packet.clone() {
            RobustMQPacket::MQTT(packet) => build_mqtt_packet_wrapper(protocol.clone(), packet),
            RobustMQPacket::KAFKA(_packet) => {
                // todo
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
        record_packet_handler_info_by_response(
            network_type,
            response_package,
            out_response_queue_ms,
        );
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
