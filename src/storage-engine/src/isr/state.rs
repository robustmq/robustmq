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
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::RwLock;
use tokio::sync::Mutex as AsyncMutex;

/// Local runtime role of a segment (authoritative leader/epoch is in meta).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplicaRole {
    Initializing,
    LeaderInitializing,
    LeaderActive,
    LeaderDemoting,
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
    state_lock: AsyncMutex<()>,
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
            state_lock: AsyncMutex::new(()),
            follower_progress: DashMap::new(),
        }
    }

    pub async fn lock_state(&self) -> tokio::sync::MutexGuard<'_, ()> {
        self.state_lock.lock().await
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

    pub fn update_follower_progress(
        &self,
        replica_id: u64,
        broker_epoch: u64,
        leader_epoch: u32,
        fetch_offset: u64,
        leader_leo: u64,
        now: u64,
    ) -> bool {
        let mut progress =
            self.follower_progress
                .entry(replica_id)
                .or_insert_with(|| FollowerProgress {
                    last_caught_up_ts: now,
                    last_fetch_ts: now,
                    ..Default::default()
                });
        if broker_epoch < progress.broker_epoch {
            return false;
        }
        progress.broker_epoch = broker_epoch;
        progress.last_known_leader_epoch = leader_epoch;
        progress.leo = fetch_offset;
        progress.last_fetch_ts = now;
        if fetch_offset >= leader_leo {
            progress.last_caught_up_ts = now;
        }
        true
    }

    pub fn reset_follower_progress(&self) {
        self.follower_progress.clear();
    }

    pub fn committable_hw(&self, isr: &[u64], leader_id: u64, leader_leo: u64) -> u64 {
        let mut hw = leader_leo;
        for replica_id in isr {
            if *replica_id == leader_id {
                continue;
            }
            if let Some(p) = self.follower_progress.get(replica_id) {
                hw = hw.min(p.leo);
            }
        }
        hw
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn segment_state_role_and_epochs() {
        let seg = SegmentReplicaState::new("s".to_string(), 0);
        assert_eq!(seg.role(), ReplicaRole::Initializing);

        seg.set_role(ReplicaRole::LeaderActive);
        seg.set_leader_epoch(3);
        seg.set_segment_epoch(7);

        assert_eq!(seg.role(), ReplicaRole::LeaderActive);
        assert_eq!(seg.leader_epoch(), 3);
        assert_eq!(seg.segment_epoch(), 7);
    }

    #[test]
    fn follower_progress_tracking() {
        let seg = SegmentReplicaState::new("s".to_string(), 0);
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

    #[test]
    fn committable_hw_is_min_across_isr() {
        let seg = SegmentReplicaState::new("s".to_string(), 0);
        assert_eq!(seg.committable_hw(&[1], 1, 100), 100);

        seg.update_follower_progress(2, 1, 1, 80, 100, 0);
        assert_eq!(seg.committable_hw(&[1, 2], 1, 100), 80);
        assert_eq!(seg.committable_hw(&[1, 2, 3], 1, 100), 80);

        seg.reset_follower_progress();
        assert_eq!(seg.committable_hw(&[1, 2], 1, 100), 100);
    }
}
