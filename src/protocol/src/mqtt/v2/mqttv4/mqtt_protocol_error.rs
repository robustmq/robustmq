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

use crate::mqtt::v2::utils::code_error::CodeError;

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum MQTTProtocolError {
    #[error("Malformed packet")]
    MalformedPacket,

    #[error("Invalid packet type: reserved bits are forbidden to use")]
    InvalidPacketType,

    #[error("This Control Packet type reserved flag is invalid")]
    InvalidFixedHeaderFlags,

    #[error("QoS can support 0, 1, 2, the specified QoS {0} level is not supported")]
    QoSLevelNotSupported(u8),

    #[error("Remaining Length field is malformed")]
    MalformedRemainingLength,

    #[error("Packet does not have enough bytes")]
    PacketTooShort,

    #[error("Protocol Name error: {0}")]
    ProtocolNameError(String),

    #[error("from CodeError: {0}")]
    CodeError(#[from] CodeError),
}
