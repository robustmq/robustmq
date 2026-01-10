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

use crate::mqtt::adapter::common::control_packet_type::ControlPacketType;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PubRecFixedHeader {
    pub control_packet_type: ControlPacketType,
    pub remaining_len: u32,
}

#[allow(dead_code)]
impl PubRecFixedHeader {
    pub fn new() -> Self {
        PubRecFixedHeader {
            control_packet_type: ControlPacketType::PubRec,
            remaining_len: 2,
        }
    }

    pub fn control_packet_type(&self) -> &ControlPacketType {
        &self.control_packet_type
    }

    pub fn set_remaining_len(&mut self, len: u32) {
        self.remaining_len = len;
    }
}
