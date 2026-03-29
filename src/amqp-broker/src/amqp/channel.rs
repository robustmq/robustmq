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
use amq_protocol::protocol::channel::{AMQPMethod, CloseOk, OpenOk};
use amq_protocol::protocol::AMQPClass;

pub fn process_channel(channel_id: u16, method: &AMQPMethod) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Open(_) => process_open(channel_id),
        AMQPMethod::Flow(_) => process_flow(channel_id),
        AMQPMethod::FlowOk(_) => process_flow_ok(channel_id),
        AMQPMethod::Close(_) => process_close(channel_id),
        AMQPMethod::CloseOk(_) => process_close_ok(channel_id),
        _ => None,
    }
}

fn process_open(channel_id: u16) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Channel(AMQPMethod::OpenOk(OpenOk {})),
    ))
}

fn process_flow(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_flow_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_close(channel_id: u16) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Channel(AMQPMethod::CloseOk(CloseOk {})),
    ))
}

fn process_close_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}
