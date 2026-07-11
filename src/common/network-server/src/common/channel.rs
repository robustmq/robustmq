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

use crate::common::packet::RequestPackage;
use tracing::error;

// Requests are sharded across `shard_num` channels by connection id. All packets
// from one connection land on the same shard, and each handler worker drains a
// single shard, so a connection's requests are processed in arrival order and
// its responses are written back in order — as protocols like Kafka require.
#[derive(Clone)]
pub struct RequestChannel {
    senders: Vec<async_channel::Sender<RequestPackage>>,
    receivers: Vec<async_channel::Receiver<RequestPackage>>,
    pub channel_size: usize,
    pub shard_num: usize,
}

impl RequestChannel {
    pub fn new(channel_size: usize, shard_num: usize) -> Self {
        let shard_num = shard_num.max(1);
        let mut senders = Vec::with_capacity(shard_num);
        let mut receivers = Vec::with_capacity(shard_num);
        for _ in 0..shard_num {
            let (sender, receiver) = async_channel::bounded(channel_size);
            senders.push(sender);
            receivers.push(receiver);
        }
        RequestChannel {
            senders,
            receivers,
            channel_size,
            shard_num,
        }
    }

    fn shard_of(&self, connection_id: u64) -> usize {
        (connection_id % self.shard_num as u64) as usize
    }

    pub async fn send(&self, packet: RequestPackage) {
        let shard = self.shard_of(packet.connection_id);
        if let Err(e) = self.senders[shard].send(packet).await {
            error!("Failed to send packet to handler channel: {}", e);
        }
    }

    /// Receiver for a single handler worker. Worker `index` (0-based) owns one shard.
    pub fn receiver(&self, index: usize) -> async_channel::Receiver<RequestPackage> {
        self.receivers[index % self.shard_num].clone()
    }

    pub fn len(&self) -> usize {
        self.senders.iter().map(|s| s.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.senders.iter().all(|s| s.is_empty())
    }
}
