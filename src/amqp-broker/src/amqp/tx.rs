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
use amq_protocol::protocol::tx::AMQPMethod;

pub fn process_tx(channel_id: u16, method: &AMQPMethod) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Select(_) => process_select(channel_id),
        AMQPMethod::Commit(_) => process_commit(channel_id),
        AMQPMethod::Rollback(_) => process_rollback(channel_id),
        _ => None,
    }
}

fn process_select(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_commit(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_rollback(_channel_id: u16) -> Option<AMQPFrame> {
    None
}
