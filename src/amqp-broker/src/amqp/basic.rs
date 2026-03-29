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
use amq_protocol::protocol::basic::{AMQPMethod, CancelOk, ConsumeOk, QosOk, RecoverOk};
use amq_protocol::protocol::confirm;
use amq_protocol::protocol::AMQPClass;

/// Handle Basic class methods from client.
/// Basic.Get is handled in command.rs where storage access is available.
pub fn process_basic(channel_id: u16, method: &AMQPMethod) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Qos(_) => process_qos(channel_id),
        AMQPMethod::Consume(m) => process_consume(channel_id, m.consumer_tag.as_str()),
        AMQPMethod::Cancel(m) => process_cancel(channel_id, m.consumer_tag.as_str()),
        AMQPMethod::Publish(_) => None, // fire-and-forget, no response
        AMQPMethod::Get(_) => None,     // handled in command.rs
        AMQPMethod::Ack(_) => None,     // no response
        AMQPMethod::Reject(_) => None,  // no response
        AMQPMethod::RecoverAsync(_) => None,
        AMQPMethod::Recover(_) => process_recover(channel_id),
        AMQPMethod::Nack(_) => None, // no response
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

fn process_qos(channel_id: u16) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::QosOk(QosOk {})),
    ))
}

fn process_consume(channel_id: u16, consumer_tag: &str) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::ConsumeOk(ConsumeOk {
            consumer_tag: consumer_tag.into(),
        })),
    ))
}

fn process_cancel(channel_id: u16, consumer_tag: &str) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::CancelOk(CancelOk {
            consumer_tag: consumer_tag.into(),
        })),
    ))
}

fn process_recover(channel_id: u16) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Basic(AMQPMethod::RecoverOk(RecoverOk {})),
    ))
}

fn process_confirm_select(channel_id: u16) -> Option<AMQPFrame> {
    use amq_protocol::protocol::confirm::{AMQPMethod as ConfirmMethod, SelectOk};
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Confirm(ConfirmMethod::SelectOk(SelectOk {})),
    ))
}
