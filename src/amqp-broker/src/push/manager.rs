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

use dashmap::DashMap;
use tokio::sync::broadcast;

/// Marks a shard's cursor as not yet seeded from the last committed offset.
pub(crate) const UNSEEDED: u64 = u64::MAX;

/// Per-shard in-memory read cursors and running push-task registry, for the
/// queues this node currently leads.
#[derive(Default)]
pub struct AmqpPushManager {
    // "{tenant}#{queue}#{shard}" -> next offset to read
    cursors: DashMap<String, Arc<AtomicU64>>,
    // "{tenant}#{queue}" -> that queue's push task's stop channel
    running: DashMap<String, broadcast::Sender<bool>>,
}

fn queue_key(tenant: &str, queue: &str) -> String {
    format!("{tenant}#{queue}")
}

fn shard_key(tenant: &str, queue: &str, shard_name: &str) -> String {
    format!("{tenant}#{queue}#{shard_name}")
}

impl AmqpPushManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn cursor(&self, tenant: &str, queue: &str, shard_name: &str) -> Arc<AtomicU64> {
        self.cursors
            .entry(shard_key(tenant, queue, shard_name))
            .or_insert_with(|| Arc::new(AtomicU64::new(UNSEEDED)))
            .clone()
    }

    pub fn is_running(&self, tenant: &str, queue: &str) -> bool {
        self.running.contains_key(&queue_key(tenant, queue))
    }

    pub fn mark_running(&self, tenant: &str, queue: &str, stop_tx: broadcast::Sender<bool>) {
        self.running.insert(queue_key(tenant, queue), stop_tx);
    }

    /// Also drops this queue's in-memory cursors, so regaining leadership
    /// later reseeds from meta-service instead of resuming a stale value.
    pub fn mark_stopped(&self, tenant: &str, queue: &str) -> Option<broadcast::Sender<bool>> {
        let prefix = queue_key(tenant, queue);
        self.cursors
            .retain(|k, _| !k.starts_with(&format!("{prefix}#")));
        self.running.remove(&prefix).map(|(_, tx)| tx)
    }
}
