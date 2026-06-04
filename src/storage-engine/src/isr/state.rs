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

use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use tokio::sync::watch;

/// Local runtime role of a segment (authoritative leader/epoch is in meta).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplicaRole {
    /// No SegmentLeaderAndIsr yet; rejects all read/write/fetch.
    Initializing,
    /// Elected leader, not yet ready (LeaderEpochCache persist pending, I11);
    /// rejects writes and fetch.
    LeaderInitializing,
    LeaderActive,
    /// Demoting from leader: draining in-flight writes before becoming follower.
    LeaderDemoting,
    /// Following, but must run OffsetsForLeaderEpoch truncation before fetch (I9).
    FollowerInitializing,
    FollowerActive,
}

#[derive(Clone, Debug, Default)]
pub struct FollowerProgress {
    pub broker_epoch: u64,
    pub last_known_leader_epoch: u32,
    pub leo: u64,
    pub last_fetch_ts: u64,
    pub last_caught_up_ts: u64,
    pub first_caught_up_after_oos: Option<u64>,
}

pub struct SegmentReplicaState {
    pub shard_name: String,
    pub segment_seq: u32,
    leader_epoch: AtomicU32,
    segment_epoch: AtomicU32,
    role: RwLock<ReplicaRole>,
    pub follower_progress: DashMap<u64, FollowerProgress>,
}

impl SegmentReplicaState {
    pub fn new(shard_name: String, segment_seq: u32) -> Self {
        SegmentReplicaState {
            shard_name,
            segment_seq,
            leader_epoch: AtomicU32::new(0),
            segment_epoch: AtomicU32::new(0),
            role: RwLock::new(ReplicaRole::Initializing),
            follower_progress: DashMap::new(),
        }
    }

    pub fn role(&self) -> ReplicaRole {
        *self.role.read().unwrap()
    }

    pub fn set_role(&self, role: ReplicaRole) {
        *self.role.write().unwrap() = role;
    }

    pub fn leader_epoch(&self) -> u32 {
        self.leader_epoch.load(Ordering::SeqCst)
    }

    pub fn set_leader_epoch(&self, epoch: u32) {
        self.leader_epoch.store(epoch, Ordering::SeqCst);
    }

    pub fn segment_epoch(&self) -> u32 {
        self.segment_epoch.load(Ordering::SeqCst)
    }

    pub fn set_segment_epoch(&self, epoch: u32) {
        self.segment_epoch.store(epoch, Ordering::SeqCst);
    }
}

/// HW / LEO / log_start are continuous across segments, so they live on the
/// shard. HW advance and the watcher are wired up in T11.
pub struct ShardReplicaState {
    pub shard_name: String,
    pub local_leo: AtomicU64,
    pub local_hw: AtomicU64,
    pub log_start_offset: AtomicU64,
    pub hw_watcher: watch::Sender<u64>,
}

impl ShardReplicaState {
    pub fn new(shard_name: String) -> Self {
        let (hw_watcher, _) = watch::channel(0);
        ShardReplicaState {
            shard_name,
            local_leo: AtomicU64::new(0),
            local_hw: AtomicU64::new(0),
            log_start_offset: AtomicU64::new(0),
            hw_watcher,
        }
    }
}

#[derive(Default)]
pub struct ReplicaStateRegistry {
    pub shard_states: DashMap<String, Arc<ShardReplicaState>>,
    pub segment_states: DashMap<(String, u32), Arc<SegmentReplicaState>>,
}

impl ReplicaStateRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_create_segment(&self, shard: &str, segment_seq: u32) -> Arc<SegmentReplicaState> {
        self.segment_states
            .entry((shard.to_string(), segment_seq))
            .or_insert_with(|| Arc::new(SegmentReplicaState::new(shard.to_string(), segment_seq)))
            .clone()
    }

    pub fn get_segment(&self, shard: &str, segment_seq: u32) -> Option<Arc<SegmentReplicaState>> {
        self.segment_states
            .get(&(shard.to_string(), segment_seq))
            .map(|s| s.clone())
    }

    pub fn get_or_create_shard(&self, shard: &str) -> Arc<ShardReplicaState> {
        self.shard_states
            .entry(shard.to_string())
            .or_insert_with(|| Arc::new(ShardReplicaState::new(shard.to_string())))
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_state_role_and_epochs() {
        let reg = ReplicaStateRegistry::new();
        let seg = reg.get_or_create_segment("s", 0);
        assert_eq!(seg.role(), ReplicaRole::Initializing);

        seg.set_role(ReplicaRole::LeaderActive);
        seg.set_leader_epoch(3);
        seg.set_segment_epoch(7);

        // same registry returns the same Arc
        let seg2 = reg.get_or_create_segment("s", 0);
        assert_eq!(seg2.role(), ReplicaRole::LeaderActive);
        assert_eq!(seg2.leader_epoch(), 3);
        assert_eq!(seg2.segment_epoch(), 7);
    }

    #[test]
    fn follower_progress_tracking() {
        let reg = ReplicaStateRegistry::new();
        let seg = reg.get_or_create_segment("s", 0);
        seg.follower_progress.insert(
            2,
            FollowerProgress {
                leo: 10,
                last_known_leader_epoch: 3,
                ..Default::default()
            },
        );
        assert_eq!(seg.follower_progress.get(&2).unwrap().leo, 10);
    }
}
