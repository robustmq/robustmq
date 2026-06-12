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

use crate::commitlog::offset::CommitLogOffset;
use crate::core::cache::StorageCacheManager;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::warn;

#[derive(Clone, Debug, Default)]
pub struct FollowerProgress {
    pub follower_broker_epoch: u64,
    pub leo: u64,
    pub last_fetch_ts: u64,
}

pub type SegmentReplicaState = DashMap<u64, FollowerProgress>;

#[derive(Debug)]
pub struct StaleBrokerEpoch {
    pub replica_id: u64,
    pub received: u64,
    pub current: u64,
}

pub fn update_follower_progress(
    fp: &SegmentReplicaState,
    replica_id: u64,
    follower_broker_epoch: u64,
    fetch_offset: u64,
    leader_leo: u64,
    now: u64,
) -> Result<(), StaleBrokerEpoch> {
    let mut progress = fp.entry(replica_id).or_insert_with(|| FollowerProgress {
        last_fetch_ts: now,
        ..Default::default()
    });
    if follower_broker_epoch < progress.follower_broker_epoch {
        return Err(StaleBrokerEpoch {
            replica_id,
            received: follower_broker_epoch,
            current: progress.follower_broker_epoch,
        });
    }
    progress.follower_broker_epoch = follower_broker_epoch;
    progress.leo = fetch_offset;
    if fetch_offset >= leader_leo {
        progress.last_fetch_ts = now;
    }
    Ok(())
}

fn committable_hw(fp: &SegmentReplicaState, isr: &[u64], leader_id: u64, leader_leo: u64) -> u64 {
    let mut hw = leader_leo;
    for replica_id in isr {
        if *replica_id == leader_id {
            continue;
        }
        if let Some(p) = fp.get(replica_id) {
            hw = hw.min(p.leo);
        }
    }
    hw
}

pub fn advance_hw(
    cache_manager: &Arc<StorageCacheManager>,
    commit_log_offset: &CommitLogOffset,
    shard: &str,
    segment_seq: u32,
    isr: &[u64],
    leader_id: u64,
    leader_leo: u64,
) -> Option<u64> {
    let state = cache_manager.get_segment_replica(shard, segment_seq)?;
    let new_hw = committable_hw(&state, isr, leader_id, leader_leo);
    match commit_log_offset.save_high_watermark_offset(shard, new_hw) {
        Ok(true) => {
            let _ = cache_manager.hw_watcher(shard).send(new_hw);
        }
        Ok(false) => {}
        Err(e) => warn!("persist high watermark for shard {shard}: {e}"),
    }
    Some(new_hw)
}

pub async fn wait_for_hw(
    cache_manager: &Arc<StorageCacheManager>,
    shard: &str,
    target_offset: u64,
    wait_ms: u64,
) -> bool {
    let current = cache_manager
        .get_offset_state(shard)
        .map(|s| s.high_watermark_offset)
        .unwrap_or_else(|| {
            warn!("offset state not found for shard {shard}, treating HW as 0");
            0
        });
    if current >= target_offset {
        return true;
    }

    let mut rx = cache_manager.hw_watcher(shard).subscribe();
    let wait = async {
        loop {
            if *rx.borrow() >= target_offset {
                return true;
            }
            if rx.changed().await.is_err() {
                return false;
            }
        }
    };
    matches!(
        timeout(Duration::from_millis(wait_ms), wait).await,
        Ok(true)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitlog::offset::ShardOffsetState;
    use crate::core::test_tool::test_build_memory_engine;

    #[tokio::test]
    async fn committable_hw_is_min_across_isr() {
        let fp: SegmentReplicaState = DashMap::new();
        assert_eq!(committable_hw(&fp, &[1], 1, 100), 100);

        update_follower_progress(&fp, 2, 1, 80, 100, 0).unwrap();
        assert_eq!(committable_hw(&fp, &[1, 2], 1, 100), 80);
        assert_eq!(committable_hw(&fp, &[1, 2, 3], 1, 100), 80);

        fp.clear();
        assert_eq!(committable_hw(&fp, &[1, 2], 1, 100), 100);
    }

    #[test]
    fn stale_broker_epoch_is_rejected_and_progress_kept() {
        let fp: SegmentReplicaState = DashMap::new();
        update_follower_progress(&fp, 2, 5, 10, 10, 0).unwrap();

        let err = update_follower_progress(&fp, 2, 3, 20, 20, 0).unwrap_err();
        assert_eq!(err.current, 5);
        assert_eq!(err.received, 3);
        // a fenced update must not advance the recorded progress
        assert_eq!(fp.get(&2).unwrap().leo, 10);
    }

    fn setup(shard: &str, isr: &[u64]) -> (Arc<StorageCacheManager>, Arc<CommitLogOffset>) {
        let engine = test_build_memory_engine();
        let cm = engine.cache_manager.clone();
        cm.save_offset_state(shard.to_string(), ShardOffsetState::default());
        cm.add_segment_replica(shard, 0);
        let state = cm.get_segment_replica(shard, 0).unwrap();
        for id in isr {
            if *id != 1 {
                update_follower_progress(&state, *id, 1, 0, 0, 0).unwrap();
            }
        }
        (cm, engine.commit_log_offset.clone())
    }

    #[tokio::test]
    async fn advance_hw_to_min_isr_leo() {
        let (cm, clo) = setup("s", &[1, 2]);
        let state = cm.get_segment_replica("s", 0).unwrap();
        update_follower_progress(&state, 2, 1, 5, 10, 0).unwrap();

        let hw = advance_hw(&cm, &clo, "s", 0, &[1, 2], 1, 10).unwrap();
        assert_eq!(hw, 5);
        assert_eq!(cm.get_offset_state("s").unwrap().high_watermark_offset, 5);
    }

    #[tokio::test]
    async fn wait_for_hw_wakes_on_advance() {
        let (cm, clo) = setup("s", &[1]);
        let cm2 = cm.clone();
        let waiter = tokio::spawn(async move { wait_for_hw(&cm2, "s", 3, 1000).await });
        tokio::time::sleep(Duration::from_millis(20)).await;
        advance_hw(&cm, &clo, "s", 0, &[1], 1, 3);
        assert!(waiter.await.unwrap());
    }

    #[tokio::test]
    async fn wait_for_hw_times_out() {
        let (cm, _clo) = setup("s", &[1, 2]);
        assert!(!wait_for_hw(&cm, "s", 3, 50).await);
    }
}
