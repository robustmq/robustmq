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

use amq_protocol::frame::AMQPFrame;
use amq_protocol::protocol::connection::AMQPMethod;

pub fn process_protocol_header() -> Option<AMQPFrame> {
    None
}

pub fn process_heartbeat(channel_id: u16) -> Option<AMQPFrame> {
    Some(AMQPFrame::Heartbeat(channel_id))
}

/// Handle connection-level methods sent by the client to the server.
/// Server-initiated methods (Start, Tune, OpenOk, UpdateSecret) are not listed here.
pub fn process_connection(channel_id: u16, method: &AMQPMethod) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::StartOk(_) => process_start_ok(channel_id),
        AMQPMethod::SecureOk(_) => process_secure_ok(channel_id),
        AMQPMethod::TuneOk(_) => process_tune_ok(channel_id),
        AMQPMethod::Open(_) => process_open(channel_id),
        AMQPMethod::Close(_) => process_close(channel_id),
        AMQPMethod::CloseOk(_) => process_close_ok(channel_id),
        AMQPMethod::Blocked(_) => process_blocked(channel_id),
        AMQPMethod::Unblocked(_) => process_unblocked(channel_id),
        AMQPMethod::UpdateSecretOk(_) => process_update_secret_ok(channel_id),
        _ => None,
    }
}

fn process_start_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_secure_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_tune_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_open(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_close(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_close_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_blocked(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_unblocked(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_update_secret_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}
