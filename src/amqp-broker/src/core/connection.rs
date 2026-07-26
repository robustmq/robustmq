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

use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64};
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
    // Confirm.Select state: once set, every Basic.Publish on this channel is
    // assigned the next value from `next_publish_seqno` and acked/nacked by
    // that number once its write to storage resolves.
    pub confirm_mode: Arc<AtomicBool>,
    pub next_publish_seqno: Arc<AtomicU64>,
    // Basic.Qos prefetch_count; 0 means unlimited (spec default).
    pub prefetch_count: Arc<AtomicU32>,
    // Channel.Flow: false pauses Basic.Deliver push to this channel until a
    // Flow{active: true} re-enables it. Basic.Get is unaffected (pull, not push).
    pub flow_active: Arc<AtomicBool>,
}

impl AmqpChannel {
    pub fn new(connection_id: u64, channel_id: u16) -> Self {
        AmqpChannel {
            connection_id,
            channel_id,
            state: AmqpChannelState::Open,
            create_time: now_second(),
            next_delivery_tag: Arc::new(AtomicU64::new(1)),
            confirm_mode: Arc::new(AtomicBool::new(false)),
            next_publish_seqno: Arc::new(AtomicU64::new(1)),
            prefetch_count: Arc::new(AtomicU32::new(0)),
            flow_active: Arc::new(AtomicBool::new(true)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn new_channel_starts_open_unconfirmed_unlimited_and_flowing() {
        let channel = AmqpChannel::new(1, 2);
        assert_eq!(channel.state, AmqpChannelState::Open);
        assert!(!channel.confirm_mode.load(Ordering::SeqCst));
        assert_eq!(channel.next_publish_seqno.load(Ordering::SeqCst), 1);
        assert_eq!(channel.prefetch_count.load(Ordering::SeqCst), 0);
        assert!(channel.flow_active.load(Ordering::SeqCst));
    }

    #[test]
    fn confirm_mode_and_flow_state_are_shared_across_clones() {
        let channel = AmqpChannel::new(1, 2);
        let clone = channel.clone();
        clone.confirm_mode.store(true, Ordering::SeqCst);
        clone.flow_active.store(false, Ordering::SeqCst);
        assert!(channel.confirm_mode.load(Ordering::SeqCst));
        assert!(!channel.flow_active.load(Ordering::SeqCst));
    }
}
