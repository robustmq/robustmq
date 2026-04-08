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
use bytes::Bytes;

/// Positive acknowledgement — message processing succeeded.
pub async fn process_ack(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _payload: &Bytes,
) -> Result<(), NatsBrokerError> {
    todo!("ACK")
}

/// Negative acknowledgement — re-deliver the message, optionally after a delay.
pub async fn process_nak(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _payload: &Bytes,
) -> Result<(), NatsBrokerError> {
    todo!("NAK")
}

/// In-progress signal — resets the ack wait timer without acknowledging.
pub async fn process_ack_progress(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<(), NatsBrokerError> {
    todo!("ACK progress (+WPI)")
}

/// Terminate — mark message as permanently failed, no re-delivery.
pub async fn process_ack_term(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
) -> Result<(), NatsBrokerError> {
    todo!("ACK term (+TERM)")
}

/// Ack + request next message in a single operation.
pub async fn process_ack_next(
    _ctx: &NatsProcessContext,
    _stream: &str,
    _consumer: &str,
    _payload: &Bytes,
) -> Result<(), NatsBrokerError> {
    todo!("ACK next (+NXT)")
}
