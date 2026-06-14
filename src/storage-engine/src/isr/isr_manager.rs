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

use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
use crate::core::cache::StorageCacheManager;
use crate::filesegment::SegmentIdentity;
use crate::isr::follower::SegmentReplicaState;
use crate::isr::log::ReplicaLog;
use common_base::error::ResultCommonError;
use common_base::tools::{loop_select_ticket, now_second};
use common_config::broker::broker_config;
use common_config::storage::StorageType;
use grpc_clients::meta::storage::call::update_segment_isr;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::segment::EngineSegment;
use protocol::meta::meta_service_journal::UpdateSegmentIsrRequest;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{info, warn};

pub async fn start_isr_manager_thread(
    client_pool: Arc<ClientPool>,
    cache_manager: Arc<StorageCacheManager>,
    memory: Arc<MemoryStorageEngine>,
    rocksdb: Arc<RocksDBStorageEngine>,
    stop_sx: &broadcast::Sender<bool>,
) {
    let interval = broker_config().storage_runtime.isr_maintain_interval_ms;
    let ac_fn = async || -> ResultCommonError {
        maintain_once(&client_pool, &cache_manager, &memory, &rocksdb).await;
        Ok(())
    };
    loop_select_ticket(ac_fn, interval, stop_sx).await;
}

// Number of leader segments handled by a single maintain task. Leader segments are split into
// chunks of this size and processed concurrently; maintain_once returns once all tasks complete.
const MAINTAIN_TASK_CHUNK_SIZE: usize = 1000;

async fn maintain_once(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
    memory: &Arc<MemoryStorageEngine>,
    rocksdb: &Arc<RocksDBStorageEngine>,
) {
    let leader_segments: Vec<_> = cache_manager
        .leader_segments
        .iter()
        .map(|e| e.value().clone())
        .collect();

    let mut handles = Vec::new();
    for chunk in leader_segments.chunks(MAINTAIN_TASK_CHUNK_SIZE) {
        let chunk = chunk.to_vec();
        let client_pool = client_pool.clone();
        let cache_manager = cache_manager.clone();
        let memory = memory.clone();
        let rocksdb = rocksdb.clone();
        handles.push(tokio::spawn(async move {
            maintain_segments(&client_pool, &cache_manager, &memory, &rocksdb, chunk).await;
        }));
    }

    for handle in handles {
        if let Err(e) = handle.await {
            warn!("ISR maintain task failed: {}", e);
        }
    }
}

async fn maintain_segments(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
    memory: &Arc<MemoryStorageEngine>,
    rocksdb: &Arc<RocksDBStorageEngine>,
    segments: Vec<SegmentIdentity>,
) {
    let conf = broker_config();

    for segment_iden in segments {
        let Some(segment) = cache_manager.get_segment(&segment_iden) else {
            continue;
        };

        let shard = segment.shard_name.clone();
        let segment_seq = segment.segment_seq;

        let Some(state) = cache_manager.get_segment_replica(&shard, segment_seq) else {
            continue;
        };

        let Some(leader_leo) = leo_of(cache_manager, memory, rocksdb, &shard, segment_seq) else {
            continue;
        };

        let replicas: Vec<u64> = segment.replicas.iter().map(|r| r.node_id).collect();
        let lag_ms = conf.storage_runtime.replica_lag_time_max_ms;
        let new_isr = compute_new_isr(
            &state,
            &segment.isr,
            &replicas,
            segment.leader,
            leader_leo,
            lag_ms,
            now_second(),
        );

        if let Some(new_isr) = new_isr {
            let broker_epoch = cache_manager.broker_cache.get_broker_epoch();
            propose_isr(client_pool, conf.broker_id, broker_epoch, &segment, new_isr).await;
        }
    }
}

async fn propose_isr(
    client_pool: &Arc<ClientPool>,
    broker_id: u64,
    broker_epoch: u64,
    segment: &EngineSegment,
    new_isr: Vec<u64>,
) {
    let conf = broker_config();
    let req = UpdateSegmentIsrRequest {
        shard_name: segment.shard_name.clone(),
        segment: segment.segment_seq,
        new_isr: new_isr.clone(),
        requester_node_id: broker_id,
        requester_broker_epoch: broker_epoch,
        expected_leader_epoch: segment.leader_epoch,
        expected_segment_epoch: segment.segment_epoch,
    };
    match update_segment_isr(client_pool, &conf.get_meta_service_addr(), req).await {
        Ok(_) => {
            info!(
                "ISR updated for {}/{}: {:?}",
                segment.shard_name, segment.segment_seq, new_isr
            );
        }
        Err(e) => {
            warn!(
                "ISR maintain propose failed for {}/{}: {}",
                segment.shard_name, segment.segment_seq, e
            );
        }
    }
}

fn leo_of(
    cache_manager: &Arc<StorageCacheManager>,
    memory: &Arc<MemoryStorageEngine>,
    rocksdb: &Arc<RocksDBStorageEngine>,
    shard: &str,
    segment_seq: u32,
) -> Option<u64> {
    match cache_manager
        .shards
        .get(shard)
        .map(|s| s.config.storage_type)
    {
        Some(StorageType::EngineRocksDB) => rocksdb.latest_offset(shard, segment_seq).ok(),
        Some(StorageType::EngineMemory) => memory.latest_offset(shard, segment_seq).ok(),
        _ => None,
    }
}

pub fn compute_new_isr(
    state: &SegmentReplicaState,
    current_isr: &[u64],
    replicas: &[u64],
    leader_id: u64,
    _leader_leo: u64,
    lag_time_max_ms: u64,
    now_sec: u64,
) -> Option<Vec<u64>> {
    let lag_max_sec = lag_time_max_ms.div_ceil(1000);

    let mut new_isr: Vec<u64> = Vec::with_capacity(replicas.len());
    new_isr.push(leader_id);

    for replica_id in replicas {
        if *replica_id == leader_id {
            continue;
        }
        let progress = state.get(replica_id);

        // In-sync is determined purely by fetch liveness: the follower must have
        // caught up to the leader's LEO recently (within replica_lag_time_max_ms).
        // `last_fetch_ts` is refreshed only when a fetch reaches the leader LEO, so an
        // actively-fetching, caught-up follower keeps it fresh, while a follower that
        // stopped fetching (e.g. it died) has a frozen `last_fetch_ts` and is dropped
        // once the lag window passes.
        //
        // NOTE: we must NOT also keep a follower whose last-known `leo >= leader_leo`.
        // A follower that died while fully caught up keeps that frozen LEO equal to a
        // no-longer-advancing leader LEO forever, which would pin a dead replica in the
        // ISR indefinitely (blocking acks=all and leaving the ISR stale).
        let caught_up = match &progress {
            Some(p) => now_sec.saturating_sub(p.last_fetch_ts) <= lag_max_sec,
            None => false,
        };

        // expand: caught up → add to ISR
        // shrink: not caught up → do not push (removed from ISR regardless of current membership)
        if caught_up {
            new_isr.push(*replica_id);
        }
    }

    let mut sorted_current = current_isr.to_vec();
    sorted_current.sort_unstable();
    new_isr.sort_unstable();
    if new_isr == sorted_current {
        None
    } else {
        Some(new_isr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isr::follower::update_follower_progress;

    fn state() -> SegmentReplicaState {
        SegmentReplicaState::new()
    }

    #[test]
    fn shrink_drops_lagging_follower() {
        let st = state();
        update_follower_progress(&st, 2, 1, 100, 100, 100).unwrap();
        update_follower_progress(&st, 3, 1, 50, 100, 0).unwrap();

        let new_isr = compute_new_isr(&st, &[1, 2, 3], &[1, 2, 3], 1, 100, 10000, 100);
        assert_eq!(new_isr, Some(vec![1, 2]));
    }

    #[test]
    fn no_change_when_all_caught_up() {
        let st = state();
        update_follower_progress(&st, 2, 1, 100, 100, 100).unwrap();

        let new_isr = compute_new_isr(&st, &[1, 2], &[1, 2], 1, 100, 10000, 100);
        assert_eq!(new_isr, None);
    }

    #[test]
    fn expand_adds_caught_up_replica() {
        let st = state();
        update_follower_progress(&st, 2, 1, 100, 100, 100).unwrap();

        let new_isr = compute_new_isr(&st, &[1], &[1, 2], 1, 100, 10000, 100);
        assert_eq!(new_isr, Some(vec![1, 2]));
    }

    #[test]
    fn never_caught_up_replica_excluded() {
        let st = state();
        let new_isr = compute_new_isr(&st, &[1], &[1, 2], 1, 100, 10000, 100);
        assert_eq!(new_isr, None);
    }

    // Regression: a follower that was fully caught up (leo == leader_leo) but then
    // stopped fetching must be dropped after the lag window — even though the leader
    // LEO never advanced past its frozen leo. Previously a `leo >= leader_leo` check
    // pinned such a dead replica in the ISR forever.
    #[test]
    fn drops_caught_up_follower_that_stopped_fetching() {
        let st = state();
        // n2 caught up at t=100 (leo=100 == leader_leo), then went silent.
        update_follower_progress(&st, 2, 1, 100, 100, 100).unwrap();
        // Now is 200s; lag_max=10s. n2's last-known leo (100) still equals leader_leo,
        // but it hasn't fetched in 100s, so it must be removed.
        let new_isr = compute_new_isr(&st, &[1, 2], &[1, 2], 1, 100, 10000, 200);
        assert_eq!(new_isr, Some(vec![1]));
    }
}
