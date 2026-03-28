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

use amq_protocol::frame::{AMQPContentHeader, AMQPFrame};
use amq_protocol::protocol::basic::AMQPMethod;
use amq_protocol::protocol::confirm;

pub fn process_basic(channel_id: u16, method: &AMQPMethod) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Qos(_) => process_qos(channel_id),
        AMQPMethod::Consume(_) => process_consume(channel_id),
        AMQPMethod::Cancel(_) => process_cancel(channel_id),
        AMQPMethod::Publish(_) => process_publish(channel_id),
        AMQPMethod::Get(_) => process_get(channel_id),
        AMQPMethod::Ack(_) => process_ack(channel_id),
        AMQPMethod::Reject(_) => process_reject(channel_id),
        AMQPMethod::RecoverAsync(_) => process_recover_async(channel_id),
        AMQPMethod::Recover(_) => process_recover(channel_id),
        AMQPMethod::Nack(_) => process_nack(channel_id),
        _ => None,
    }
}

pub fn process_confirm(channel_id: u16, method: &confirm::AMQPMethod) -> Option<AMQPFrame> {
    match method {
        confirm::AMQPMethod::Select(_) => process_confirm_select(channel_id),
        _ => None,
    }
}

pub fn process_header(
    _channel_id: u16,
    _class_id: u16,
    _header: &AMQPContentHeader,
) -> Option<AMQPFrame> {
    None
}

pub fn process_body(_channel_id: u16, _data: &[u8]) -> Option<AMQPFrame> {
    None
}

fn process_qos(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_qos_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_consume(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_cancel(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_publish(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_get(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_ack(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_reject(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_recover_async(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_recover(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_nack(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_confirm_select(_channel_id: u16) -> Option<AMQPFrame> {
    None
}
