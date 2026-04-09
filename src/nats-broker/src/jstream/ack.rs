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
use crate::handler::command::NatsProcessContext;
use crate::jstream::protocol::{AckNextRequest, NakRequest};

/// `$JS.ACK.*` with empty payload or `+ACK` — positive acknowledgement.
pub async fn process_ack(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<(), NatsBrokerError> {
    todo!("ACK")
}

/// `$JS.ACK.*` with `-NAK` — negative acknowledgement, optional delay.
pub async fn process_nak(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _req: NakRequest,
) -> Result<(), NatsBrokerError> {
    todo!("NAK")
}

/// `$JS.ACK.*` with `-WPI` — in-progress signal, resets ack_wait timer.
pub async fn process_ack_progress(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<(), NatsBrokerError> {
    todo!("ACK progress (+WPI)")
}

/// `$JS.ACK.*` with `-TERM` — terminate, no redelivery.
pub async fn process_ack_term(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<(), NatsBrokerError> {
    todo!("ACK term (+TERM)")
}

/// `$JS.ACK.*` with `-NXT` — ack and request next message (Pull Consumer).
pub async fn process_ack_next(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _req: AckNextRequest,
) -> Result<(), NatsBrokerError> {
    todo!("ACK next (+NXT)")
}
