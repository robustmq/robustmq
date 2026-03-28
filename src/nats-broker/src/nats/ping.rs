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

use protocol::nats::packet::NatsPacket;

/// Handle a PING from the client — always reply PONG.
pub fn process_ping() -> Option<NatsPacket> {
    Some(NatsPacket::Pong)
}

/// Handle a PONG from the client (response to a server-initiated PING).
///
/// Responsibilities (all TODO):
/// - Reset the connection's ping-timeout timer
pub fn process_pong() -> Option<NatsPacket> {
    None
}
