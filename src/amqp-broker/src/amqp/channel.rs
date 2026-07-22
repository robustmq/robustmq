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

use std::sync::Arc;

use amq_protocol::frame::AMQPFrame;
use amq_protocol::protocol::channel::{
    AMQPMethod, Close as ChannelClose, CloseOk, Flow, FlowOk, OpenOk,
};
use amq_protocol::protocol::AMQPClass;

use crate::core::cache::AmqpCacheManager;
use crate::core::connection::AmqpChannel;

/// Handles the Channel class, keeping AmqpChannel cache state in sync with
/// Open/Close/CloseOk while delegating the actual ack frame to the plain
/// builder below.
pub(crate) fn process_channel_full(
    channel_id: u16,
    method: &AMQPMethod,
    connection_id: u64,
    amqp_cache: &Arc<AmqpCacheManager>,
) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Open(_) => {
            amqp_cache.set_channel(AmqpChannel::new(connection_id, channel_id));
        }
        AMQPMethod::Close(_) | AMQPMethod::CloseOk(_) => {
            amqp_cache.remove_channel(connection_id, channel_id);
        }
        _ => {}
    }
    process_channel(channel_id, method)
}

/// A server-initiated Channel.Close: signals a channel-level exception (e.g.
/// 404 NOT_FOUND, 406 PRECONDITION_FAILED) in response to a method on some
/// other class. The client must reply CloseOk, which is handled generically
/// above.
pub(crate) fn channel_error_close(
    channel_id: u16,
    reply_code: u16,
    reply_text: &str,
    class_id: u16,
    method_id: u16,
) -> AMQPFrame {
    AMQPFrame::Method(
        channel_id,
        AMQPClass::Channel(AMQPMethod::Close(ChannelClose {
            reply_code,
            reply_text: reply_text.into(),
            class_id,
            method_id,
        })),
    )
}

pub fn process_channel(channel_id: u16, method: &AMQPMethod) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Open(_) => process_open(channel_id),
        AMQPMethod::Flow(flow) => process_flow(channel_id, flow),
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

// Flow is synchronous per spec: no real throttling is implemented, so we just
// echo the requested `active` state back immediately.
fn process_flow(channel_id: u16, flow: &Flow) -> Option<AMQPFrame> {
    Some(AMQPFrame::Method(
        channel_id,
        AMQPClass::Channel(AMQPMethod::FlowOk(FlowOk {
            active: flow.active,
        })),
    ))
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
