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
use amq_protocol::protocol::exchange::AMQPMethod;

pub fn process_exchange(channel_id: u16, method: &AMQPMethod) -> Option<AMQPFrame> {
    match method {
        AMQPMethod::Declare(_) => process_declare(channel_id),
        AMQPMethod::DeclareOk(_) => process_declare_ok(channel_id),
        AMQPMethod::Delete(_) => process_delete(channel_id),
        AMQPMethod::DeleteOk(_) => process_delete_ok(channel_id),
        AMQPMethod::Bind(_) => process_bind(channel_id),
        AMQPMethod::BindOk(_) => process_bind_ok(channel_id),
        AMQPMethod::Unbind(_) => process_unbind(channel_id),
        AMQPMethod::UnbindOk(_) => process_unbind_ok(channel_id),
    }
}

fn process_declare(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_declare_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_delete(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_delete_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_bind(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_bind_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_unbind(_channel_id: u16) -> Option<AMQPFrame> {
    None
}

fn process_unbind_ok(_channel_id: u16) -> Option<AMQPFrame> {
    None
}
