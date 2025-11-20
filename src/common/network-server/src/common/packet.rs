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

use common_base::tools::now_mills;
use protocol::kafka::packet::KafkaPacket;
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
    pub extend: RobustMQWrapperExtend,
    pub receive_ms: u128,
}

impl RequestPackage {
    pub fn new(
        connection_id: u64,
        addr: SocketAddr,
        packet: RobustMQPacket,
        extend: RobustMQWrapperExtend,
    ) -> Self {
        Self {
            connection_id,
            addr,
            packet,
            extend,
            receive_ms: now_mills(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResponsePackage {
    pub connection_id: u64,
    pub packet: RobustMQPacket,

    // This field is obtained from [`RequestPackage`] and records the time when the packet is received,
    // the field related to internal record indicators do not need to be exported
    pub receive_ms: u128,
    pub out_queue_ms: u128,
    pub end_handler_ms: u128,
    pub request_packet: String,
}

impl ResponsePackage {
    pub fn new(
        connection_id: u64,
        packet: RobustMQPacket,
        receive_ms: u128,
        out_queue_ms: u128,
        end_handler_ms: u128,
        request_packet: String,
    ) -> Self {
        Self {
            connection_id,
            packet,
            receive_ms,
            out_queue_ms,
            end_handler_ms,
            request_packet,
        }
    }

    pub fn build(connection_id: u64, packet: RobustMQPacket) -> Self {
        Self {
            connection_id,
            packet,
            receive_ms: 0,
            out_queue_ms: 0,
            end_handler_ms: 0,
            request_packet: "".to_string(),
        }
    }

    pub fn set_receive_ms(&mut self, receive_ms: u128) {
        self.receive_ms = receive_ms;
    }

    pub fn get_receive_ms(&self) -> u128 {
        self.receive_ms
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

pub fn build_kafka_packet_wrapper(
    packet: KafkaPacket,
    extend: RobustMQWrapperExtend,
) -> RobustMQPacketWrapper {
    RobustMQPacketWrapper {
        protocol: RobustMQProtocol::KAFKA,
        extend,
        packet: RobustMQPacket::KAFKA(packet),
    }
}
