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
use crate::core::error::StorageEngineError;
use crate::core::offset::ShardOffset;
use crate::filesegment::SegmentIdentity;
use crate::isr::fetcher::SegmentFetchState;
use crate::isr::fetcher_manager::ReplicaFetcherManager;
use crate::isr::follower::FollowerProgress;
use crate::isr::leader_epoch::LeaderEpochCache;
use common_base::tools::now_second;
use common_config::broker::broker_config;
use metadata_struct::storage::segment::EngineSegment;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn apply_leader_and_isr(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    segment: &EngineSegment,
) -> Result<(), StorageEngineError> {
    let broker_id = broker_config().broker_id;
    let shard = &segment.shard_name;
    let segment_seq = segment.segment_seq;

    if !segment.is_replica() {
        fetcher_manager.remove_segment(shard, segment_seq);
        cache_manager.remove_segment_replica(shard, segment_seq);
        return Ok(());
    }

    cache_manager.add_segment_replica(shard, segment_seq);
    let Some(state) = cache_manager.get_segment_replica(shard, segment_seq) else {
        return Ok(());
    };

    let segment_iden = SegmentIdentity::new(shard, segment_seq);
    let local_leader_epoch = cache_manager
        .get_segment(&segment_iden)
        .ok_or_else(|| StorageEngineError::SegmentNotExist(segment_iden.name()))?
        .leader_epoch;
    let leader_epoch_changed = segment.leader_epoch > local_leader_epoch;

    if segment.leader == broker_id {
        apply_as_leader(
            cache_manager,
            rocksdb_engine_handler,
            fetcher_manager,
            &state,
            segment,
            leader_epoch_changed,
        )?;
    } else {
        apply_as_follower(
            rocksdb_engine_handler,
            fetcher_manager,
            segment,
            leader_epoch_changed,
        )?;
    }
    Ok(())
}

fn apply_as_leader(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    state: &crate::isr::follower::SegmentReplicaState,
    segment: &EngineSegment,
    leader_epoch_changed: bool,
) -> Result<(), StorageEngineError> {
    let shard = &segment.shard_name;
    let segment_seq = segment.segment_seq;

    fetcher_manager.remove_segment(shard, segment_seq);

    // Use ShardOffset (cache + RocksDB fallback) so that nodes which restarted
    // and had their cache_manager.offset_state repopulated only from RocksDB
    // (via the reconcile path, which never calls save_offset_state) still get
    // the correct LEO here.  cache_manager.get_offset_state is cache-only and
    // returns None when the cache was never populated after a restart.
    let shard_offset = ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
    let leo = shard_offset.get_latest_offset(shard)?;

    // Always keep the epoch cache current.  assign() is idempotent (no-op when
    // the epoch is already recorded), so calling it on every apply_as_leader
    // invocation is safe.  We CANNOT gate this on leader_epoch_changed because
    // of a reconcile-vs-notification race: reconcile calls set_segment() before
    // apply_leader_and_isr(), so by the time we read local_leader_epoch from the
    // cache it already equals segment.leader_epoch → leader_epoch_changed=false
    // even though this is the very first time this node is leader for this epoch.
    {
        let mut epoch_cache =
            LeaderEpochCache::load(rocksdb_engine_handler.clone(), shard, segment_seq)?;
        epoch_cache.assign(segment.leader_epoch, leo)?;
    }

    if leader_epoch_changed {
        state.clear();
    }

    // When the state is empty this node just became leader (or restarted as leader) and
    // no follower has fetched from it yet. Seed ISR members with last_fetch_ts = now() so
    // that ISR maintenance gives them replica_lag_time_max_ms to reconnect before evicting
    // them. Without this, the new leader starts with an empty SegmentReplicaState and the
    // first ISR maintenance pass immediately removes all followers — even though they are
    // alive and will start fetching once they learn the new leader address. The leader-switch
    // gap (heartbeat_timeout_ms ≫ replica_lag_time_max_ms) makes this race reliable.
    //
    // Use the leader's current leo as the seeded leo: all ISR members were caught up
    // before the switch, so this is their expected position (until their first real fetch
    // updates the entry with the actual value).
    if state.is_empty() {
        let now = now_second();
        let broker_id = broker_config().broker_id;
        for &follower_id in &segment.isr {
            if follower_id != broker_id {
                state.insert(
                    follower_id,
                    FollowerProgress {
                        last_fetch_ts: now,
                        leo,
                        follower_broker_epoch: 0,
                    },
                );
            }
        }
    }

    Ok(())
}

fn apply_as_follower(
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    segment: &EngineSegment,
    leader_epoch_changed: bool,
) -> Result<(), StorageEngineError> {
    let shard = &segment.shard_name;
    let segment_seq = segment.segment_seq;
    let max_bytes = broker_config().storage_runtime.max_segment_size as u64;

    let cache = LeaderEpochCache::load(rocksdb_engine_handler.clone(), shard, segment_seq)?;
    fetcher_manager.remove_segment(shard, segment_seq);
    fetcher_manager.assign_segment(SegmentFetchState {
        shard: shard.clone(),
        segment_seq,
        leader_node_id: segment.leader,
        current_leader_epoch: segment.leader_epoch,
        max_bytes,
        cache,
        needs_truncation: leader_epoch_changed,
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::manager::ClientConnectionManager;
    use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
    use crate::core::test_tool::test_build_memory_engine;
    use crate::isr::fetcher_manager::build_engine_fetcher_manager;
    use metadata_struct::storage::segment::Replica;

    fn manager(
        cm: &Arc<StorageCacheManager>,
        db: &Arc<RocksDBEngine>,
    ) -> Arc<ReplicaFetcherManager> {
        let memory = Arc::new(crate::commitlog::memory::engine::MemoryStorageEngine::new(
            db.clone(),
            cm.clone(),
            Default::default(),
        ));
        let rocksdb = Arc::new(RocksDBStorageEngine::new(cm.clone(), db.clone()));
        let client = Arc::new(ClientConnectionManager::new(cm.clone(), 1));
        Arc::new(build_engine_fetcher_manager(
            cm.clone(),
            memory,
            rocksdb,
            db.clone(),
            client,
        ))
    }

    fn setup(cm: &Arc<StorageCacheManager>) {
        use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};
        cm.set_shard(EngineShard {
            shard_name: "s".to_string(),
            config: EngineShardConfig::default(),
            ..Default::default()
        });
        cm.set_segment(&EngineSegment {
            shard_name: "s".to_string(),
            segment_seq: 0,
            leader: 1,
            leader_epoch: 0,
            replicas: vec![Replica {
                node_id: 1,
                ..Default::default()
            }],
            isr: vec![1],
            ..Default::default()
        });
        cm.save_offset_state(
            "s".to_string(),
            crate::core::offset::ShardOffsetState::default(),
        );
    }

    fn segment(leader: u64, replicas: &[u64], leader_epoch: u32) -> EngineSegment {
        EngineSegment {
            shard_name: "s".to_string(),
            segment_seq: 0,
            leader,
            leader_epoch,
            segment_epoch: 5,
            replicas: replicas
                .iter()
                .map(|id| Replica {
                    node_id: *id,
                    ..Default::default()
                })
                .collect(),
            isr: replicas.to_vec(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn becomes_leader_and_assigns_epoch() {
        let engine = test_build_memory_engine();
        let cm = engine.cache_manager.clone();
        let db = rocksdb_engine::test::test_rocksdb_instance();
        setup(&cm);

        apply_leader_and_isr(&cm, &db, &manager(&cm, &db), &segment(1, &[1, 2], 3))
            .await
            .unwrap();

        let cache = LeaderEpochCache::load(db, "s", 0).unwrap();
        assert_eq!(cache.latest_epoch(), 3);
    }

    #[tokio::test]
    async fn becomes_follower() {
        let engine = test_build_memory_engine();
        let cm = engine.cache_manager.clone();
        let db = rocksdb_engine::test::test_rocksdb_instance();
        setup(&cm);
        let mgr = manager(&cm, &db);

        apply_leader_and_isr(&cm, &db, &mgr, &segment(2, &[1, 2], 3))
            .await
            .unwrap();

        // follower path assigns a fetch for the leader, and must NOT record a leader epoch
        assert!(mgr.is_fetching("s", 0));
        assert_eq!(
            LeaderEpochCache::load(db, "s", 0).unwrap().latest_epoch(),
            0
        );
    }

    #[tokio::test]
    async fn not_a_replica_clears_fetcher() {
        let engine = test_build_memory_engine();
        let cm = engine.cache_manager.clone();
        let db = rocksdb_engine::test::test_rocksdb_instance();
        setup(&cm);
        let mgr = manager(&cm, &db);
        mgr.assign_segment(crate::isr::test_util::seg_state("s", 2));
        assert!(mgr.is_fetching("s", 0));

        // broker 1 is not in the replica set -> drop the fetcher and the replica state
        apply_leader_and_isr(&cm, &db, &mgr, &segment(2, &[2, 3], 3))
            .await
            .unwrap();

        assert!(!mgr.is_fetching("s", 0));
        assert!(cm.get_segment_replica("s", 0).is_none());
    }
}
