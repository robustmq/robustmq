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

use protocol::nats::packet::{ClientConnect, NatsPacket};

/// Handle a CONNECT packet from the client.
///
/// Responsibilities (all TODO):
/// - Validate auth_token / user+pass / nkey / jwt fields if auth_required
/// - Record verbose / pedantic flags on the connection state
/// - Enforce tls_required if configured
/// - Reply +OK when verbose = true
pub fn process_connect(_req: &ClientConnect) -> Option<NatsPacket> {
    None
}
