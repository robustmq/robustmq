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

use common_base::tools::now_second;
use network_server::common::connection_manager::ConnectionManager;
use protocol::nats::packet::NatsPacket;
use std::sync::Arc;

/// Handle a PING from the client — always reply PONG and refresh heartbeat.
pub fn process_ping(
    connection_id: u64,
    connection_manager: &Arc<ConnectionManager>,
) -> Option<NatsPacket> {
    connection_manager.report_heartbeat(connection_id, now_second());
    Some(NatsPacket::Pong)
}

/// Handle a PONG from the client (response to a server-initiated PING).
/// Refreshes the connection's heartbeat timestamp so the keep-alive monitor
/// does not time it out.
pub fn process_pong(
    connection_id: u64,
    connection_manager: &Arc<ConnectionManager>,
) -> Option<NatsPacket> {
    connection_manager.report_heartbeat(connection_id, now_second());
    None
}
