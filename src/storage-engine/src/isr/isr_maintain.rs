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
use crate::isr::log::ReplicaLog;
use crate::isr::state::{ReplicaRole, SegmentReplicaState};
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

pub async fn start_isr_maintain_thread(
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

async fn maintain_once(
    client_pool: &Arc<ClientPool>,
    cache_manager: &Arc<StorageCacheManager>,
    memory: &Arc<MemoryStorageEngine>,
    rocksdb: &Arc<RocksDBStorageEngine>,
) {
    let conf = broker_config();
    let leaders: Vec<(String, u32)> = cache_manager
        .segment_replica_states
        .iter()
        .filter(|e| e.value().role() == ReplicaRole::LeaderActive)
        .map(|e| e.key().clone())
        .collect();

    for (shard, segment_seq) in leaders {
        let Some(segment) = cache_manager.get_segment(&crate::filesegment::SegmentIdentity {
            shard_name: shard.clone(),
            segment: segment_seq,
        }) else {
            continue;
        };

        if segment.leader != conf.broker_id {
            continue;
        }

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
        leader_epoch: segment.leader_epoch,
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
    leader_leo: u64,
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
        let progress = state.follower_progress.get(replica_id);

        // A follower is considered in-sync if:
        // 1. Its LEO has caught up to the leader's LEO, OR
        // 2. It caught up recently enough (within replica_lag_time_max_ms) — allows short-term
        //    lag without thrashing the ISR when the leader writes faster than the follower fetches.
        let caught_up = match &progress {
            Some(p) => {
                p.leo >= leader_leo || now_sec.saturating_sub(p.last_caught_up_ts) <= lag_max_sec
            }
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

    fn state() -> SegmentReplicaState {
        SegmentReplicaState::new("s".to_string(), 0)
    }

    #[test]
    fn shrink_drops_lagging_follower() {
        let st = state();
        st.update_follower_progress(2, 1, 1, 100, 100, 100);
        st.update_follower_progress(3, 1, 1, 50, 100, 0);

        let new_isr = compute_new_isr(&st, &[1, 2, 3], &[1, 2, 3], 1, 100, 10000, 100);
        assert_eq!(new_isr, Some(vec![1, 2]));
    }

    #[test]
    fn no_change_when_all_caught_up() {
        let st = state();
        st.update_follower_progress(2, 1, 1, 100, 100, 100);

        let new_isr = compute_new_isr(&st, &[1, 2], &[1, 2], 1, 100, 10000, 100);
        assert_eq!(new_isr, None);
    }

    #[test]
    fn expand_adds_caught_up_replica() {
        let st = state();
        st.update_follower_progress(2, 1, 1, 100, 100, 100);

        let new_isr = compute_new_isr(&st, &[1], &[1, 2], 1, 100, 10000, 100);
        assert_eq!(new_isr, Some(vec![1, 2]));
    }

    #[test]
    fn never_caught_up_replica_excluded() {
        let st = state();
        let new_isr = compute_new_isr(&st, &[1], &[1, 2], 1, 100, 10000, 100);
        assert_eq!(new_isr, None);
    }
}
