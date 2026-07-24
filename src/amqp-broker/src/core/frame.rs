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
use amq_protocol::protocol::basic::AMQPProperties;

/// AMQP class id for Basic; the only class whose methods carry message content.
const BASIC_CLASS_ID: u16 = 60;

/// Builds the 3-frame reply every Basic message-carrying event sends: the
/// method frame itself (GetOk/Deliver/Return), followed by its content
/// header and body, in order.
pub(crate) fn build_basic_content_frames(
    channel_id: u16,
    method_frame: AMQPFrame,
    body: Vec<u8>,
    properties: AMQPProperties,
) -> Vec<AMQPFrame> {
    let header_frame = AMQPFrame::Header(
        channel_id,
        BASIC_CLASS_ID,
        Box::new(AMQPContentHeader {
            class_id: BASIC_CLASS_ID,
            body_size: body.len() as u64,
            properties,
        }),
    );
    let body_frame = AMQPFrame::Body(channel_id, body);
    vec![method_frame, header_frame, body_frame]
}
