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

use bytes::Bytes;
use protocol::nats::packet::NatsPacket;

pub fn process_sub(subject: &str, _queue_group: Option<&str>, sid: &str) -> Option<NatsPacket> {
    Some(NatsPacket::Msg {
        subject: subject.to_string(),
        sid: sid.to_string(),
        reply_to: None,
        payload: Bytes::from("hello from robustmq"),
    })
}

/// Handle an UNSUB packet from the client.
///
/// Responsibilities (all TODO):
/// - Look up the subscription by sid on this connection
/// - If max_msgs is None: remove the subscription immediately
/// - If max_msgs is Some(n): schedule auto-unsubscribe after n more messages are delivered
pub fn process_unsub(_sid: &str, _max_msgs: Option<u32>) -> Option<NatsPacket> {
    None
}
