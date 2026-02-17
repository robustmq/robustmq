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

use common_base::tools::now_millis;
use protocol::{
    mqtt::common::MqttPacket,
    robust::{
        MqttWrapperExtend, RobustMQPacket, RobustMQPacketWrapper, RobustMQProtocol,
        RobustMQWrapperExtend,
    },
};
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct RequestPackage {
    pub connection_id: u64,
    pub addr: SocketAddr,
    pub packet: RobustMQPacket,
    pub receive_ms: u128,
}

impl RequestPackage {
    pub fn new(connection_id: u64, addr: SocketAddr, packet: RobustMQPacket) -> Self {
        Self {
            connection_id,
            addr,
            packet,
            receive_ms: now_millis(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResponsePackage {
    pub connection_id: u64,
    pub packet: RobustMQPacket,
}

impl ResponsePackage {
    pub fn new(connection_id: u64, packet: RobustMQPacket) -> Self {
        Self {
            connection_id,
            packet,
        }
    }
}

pub fn build_mqtt_packet_wrapper(
    protocol: RobustMQProtocol,
    packet: MqttPacket,
) -> RobustMQPacketWrapper {
    RobustMQPacketWrapper {
        protocol: protocol.clone(),
        extend: RobustMQWrapperExtend::MQTT(MqttWrapperExtend {
            protocol_version: protocol.to_u8(),
        }),
        packet: RobustMQPacket::MQTT(packet),
    }
}
