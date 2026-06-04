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

use crate::core::cache::StorageCacheManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

pub fn advance_hw(
    cache_manager: &Arc<StorageCacheManager>,
    shard: &str,
    segment_seq: u32,
    isr: &[u64],
    leader_id: u64,
    leader_leo: u64,
) -> u64 {
    let Some(state) = cache_manager.get_segment_replica(shard, segment_seq) else {
        return 0;
    };
    let new_hw = state.committable_hw(isr, leader_id, leader_leo);
    if cache_manager.update_high_watermark_offset(shard, new_hw) {
        let _ = cache_manager.hw_watcher(shard).send(new_hw);
    }
    new_hw
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
        .unwrap_or(0);
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
    use crate::core::shard::ShardOffsetState;
    use crate::core::test_tool::test_build_memory_engine;

    fn setup(shard: &str, isr: &[u64]) -> Arc<StorageCacheManager> {
        let engine = test_build_memory_engine();
        let cm = engine.cache_manager.clone();
        cm.save_offset_state(shard.to_string(), ShardOffsetState::default());
        let state = cm.get_or_create_segment_replica(shard, 0);
        for id in isr {
            if *id != 1 {
                state.update_follower_progress(*id, 1, 1, 0, 0, 0);
            }
        }
        cm
    }

    #[tokio::test]
    async fn advance_hw_to_min_isr_leo() {
        let cm = setup("s", &[1, 2]);
        let state = cm.get_segment_replica("s", 0).unwrap();
        state.update_follower_progress(2, 1, 1, 5, 10, 0);

        let hw = advance_hw(&cm, "s", 0, &[1, 2], 1, 10);
        assert_eq!(hw, 5);
        assert_eq!(cm.get_offset_state("s").unwrap().high_watermark_offset, 5);
    }

    #[tokio::test]
    async fn wait_for_hw_wakes_on_advance() {
        let cm = setup("s", &[1]);
        let cm2 = cm.clone();
        let waiter = tokio::spawn(async move { wait_for_hw(&cm2, "s", 3, 1000).await });
        tokio::time::sleep(Duration::from_millis(20)).await;
        advance_hw(&cm, "s", 0, &[1], 1, 3);
        assert!(waiter.await.unwrap());
    }

    #[tokio::test]
    async fn wait_for_hw_times_out() {
        let cm = setup("s", &[1, 2]);
        assert!(!wait_for_hw(&cm, "s", 3, 50).await);
    }
}
