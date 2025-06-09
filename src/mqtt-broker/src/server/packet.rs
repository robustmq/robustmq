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

use std::net::SocketAddr;

use common_base::tools::now_mills;
use protocol::mqtt::common::MqttPacket;

#[derive(Clone, Debug)]
pub struct RequestPackage {
    pub connection_id: u64,
    pub addr: SocketAddr,
    pub packet: MqttPacket,
    pub receive_ms: u128,
}

impl RequestPackage {
    pub fn new(connection_id: u64, addr: SocketAddr, packet: MqttPacket) -> Self {
        Self {
            connection_id,
            addr,
            packet,
            receive_ms: now_mills(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResponsePackage {
    pub connection_id: u64,
    pub packet: MqttPacket,

    // This field is obtained from [`RequestPackage`] and records the time when the packet is received,
    // the field related to internal record indicators do not need to be exported
    receive_ms: u128,
}

impl ResponsePackage {
    pub fn new(connection_id: u64, packet: MqttPacket) -> Self {
        Self {
            connection_id,
            packet,
            receive_ms: 0,
        }
    }

    pub fn set_receive_ms(&mut self, receive_ms: u128) {
        self.receive_ms = receive_ms;
    }

    pub fn get_receive_ms(&self) -> u128 {
        self.receive_ms
    }
}
