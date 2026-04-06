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

use crate::core::error::NatsBrokerError;
use axum::extract::ws::Message;
use bytes::BytesMut;
use metadata_struct::connection::NetworkConnectionType;
use network_server::common::connection_manager::ConnectionManager;
use protocol::nats::codec::NatsCodec;
use protocol::nats::packet::NatsPacket;
use protocol::robust::{
    NatsWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
    RobustMQWrapperExtend,
};
use std::sync::Arc;
use tokio_util::codec::Encoder;

fn build_wrapper(packet: NatsPacket) -> RobustMQPacketWrapper {
    RobustMQPacketWrapper {
        protocol: RobustMQProtocol::NATS,
        extend: RobustMQWrapperExtend::NATS(NatsWrapperExtend {}),
        packet: RobustMQPacket::NATS(packet),
    }
}

/// Write a NATS packet to a client connection, routing by connection type
/// (WebSocket/WebSockets → ws frame, QUIC → quic frame, Tcp/Tls → tcp frame).
pub async fn write_nats_packet(
    cm: &Arc<ConnectionManager>,
    connect_id: u64,
    packet: NatsPacket,
) -> Result<(), NatsBrokerError> {
    let conn_type = cm.get_network_type(connect_id).ok_or_else(|| {
        NatsBrokerError::CommonError(format!("connection {} not found", connect_id))
    })?;

    match conn_type {
        NetworkConnectionType::WebSocket | NetworkConnectionType::WebSockets => {
            let mut codec = NatsCodec::new();
            let mut buf = BytesMut::new();
            codec
                .encode(packet.clone(), &mut buf)
                .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
            let wrapper = build_wrapper(packet);
            cm.write_websocket_frame(connect_id, wrapper, Message::Binary(buf.to_vec().into()))
                .await
                .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
        }
        NetworkConnectionType::QUIC => {
            let wrapper = build_wrapper(packet);
            cm.write_quic_frame(connect_id, wrapper)
                .await
                .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
        }
        NetworkConnectionType::Tcp | NetworkConnectionType::Tls => {
            let wrapper = build_wrapper(packet);
            cm.write_tcp_frame(connect_id, wrapper)
                .await
                .map_err(|e| NatsBrokerError::CommonError(e.to_string()))?;
        }
    }

    Ok(())
}
