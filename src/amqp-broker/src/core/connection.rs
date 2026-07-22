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

use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use common_base::tools::now_second;

#[derive(Clone, Debug, Default, PartialEq)]
pub enum AmqpConnectionState {
    #[default]
    Starting,
    Tuning,
    Open,
    Closed,
}

// Runtime-only, per-process connection state — never persisted or replicated
// via meta-service (unlike AmqpExchange/AmqpQueue/AmqpBinding).
#[derive(Clone, Debug)]
pub struct AmqpConnection {
    pub connection_id: u64,
    // AMQP's virtual_host is RobustMQ's tenant; empty until Connection.Open.
    pub tenant: String,
    pub username: String,
    pub state: AmqpConnectionState,
    pub channel_max: u16,
    pub frame_max: u32,
    pub heartbeat: u16,
    pub create_time: u64,
}

impl AmqpConnection {
    pub fn new(connection_id: u64) -> Self {
        AmqpConnection {
            connection_id,
            tenant: String::new(),
            username: String::new(),
            state: AmqpConnectionState::Starting,
            channel_max: 0,
            frame_max: 0,
            heartbeat: 0,
            create_time: now_second(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum AmqpChannelState {
    #[default]
    Open,
    Closed,
}

#[derive(Clone, Debug)]
pub struct AmqpChannel {
    pub connection_id: u64,
    pub channel_id: u16,
    pub state: AmqpChannelState,
    pub create_time: u64,
    // Basic.Deliver/Basic.GetOk delivery_tag, scoped to this channel's
    // lifetime: starts at 1, only increases, never reused. Wrapped in Arc so
    // every clone of this AmqpChannel (AmqpCacheManager::get_channel returns
    // clones) shares the same counter instead of each getting its own.
    pub next_delivery_tag: Arc<AtomicU64>,
}

impl AmqpChannel {
    pub fn new(connection_id: u64, channel_id: u16) -> Self {
        AmqpChannel {
            connection_id,
            channel_id,
            state: AmqpChannelState::Open,
            create_time: now_second(),
            next_delivery_tag: Arc::new(AtomicU64::new(1)),
        }
    }
}
