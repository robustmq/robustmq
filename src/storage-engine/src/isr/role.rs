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
use crate::isr::fetcher::SegmentFetchState;
use crate::isr::fetcher_manager::ReplicaFetcherManager;
use crate::isr::leader_epoch::LeaderEpochCache;
use crate::isr::state::ReplicaRole;
use common_config::broker::broker_config;
use metadata_struct::storage::segment::EngineSegment;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn apply_leader_and_isr(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    segment: &EngineSegment,
) -> Result<ReplicaRole, StorageEngineError> {
    let broker_id = broker_config().broker_id;
    let shard = &segment.shard_name;
    let segment_seq = segment.segment_seq;

    let state = cache_manager.get_or_create_segment_replica(shard, segment_seq);
    let _guard = state.lock_state().await;

    let local_leader_epoch = state.leader_epoch();
    let local_segment_epoch = state.segment_epoch();
    if segment.leader_epoch < local_leader_epoch
        || (segment.leader_epoch == local_leader_epoch
            && segment.segment_epoch < local_segment_epoch)
    {
        return Ok(state.role());
    }
    let leader_epoch_changed = segment.leader_epoch > local_leader_epoch;

    if !segment.is_replica() {
        state.set_role(ReplicaRole::Initializing);
        fetcher_manager.remove_segment(shard, segment_seq);
        return Ok(ReplicaRole::Initializing);
    }

    if segment.leader == broker_id {
        let prev_role = state.role();
        state.set_role(ReplicaRole::LeaderInitializing);
        fetcher_manager.remove_segment(shard, segment_seq);

        let leo = cache_manager
            .get_offset_state(shard)
            .map(|s| s.latest_offset)
            .unwrap_or(0);
        let assign = (|| {
            let mut epoch_cache =
                LeaderEpochCache::load(rocksdb_engine_handler.clone(), shard, segment_seq)?;
            epoch_cache.assign(segment.leader_epoch, leo)
        })();
        if let Err(e) = assign {
            state.set_role(prev_role);
            return Err(e);
        }

        if leader_epoch_changed {
            state.reset_follower_progress();
        }
        state.set_leader_epoch(segment.leader_epoch);
        state.set_segment_epoch(segment.segment_epoch);
        state.set_role(ReplicaRole::LeaderActive);
        Ok(ReplicaRole::LeaderActive)
    } else {
        let prev_role = state.role();
        state.set_role(ReplicaRole::FollowerInitializing);

        let max_bytes = broker_config().storage_runtime.max_segment_size as u64;
        let cache = match LeaderEpochCache::load(rocksdb_engine_handler.clone(), shard, segment_seq)
        {
            Ok(c) => c,
            Err(e) => {
                state.set_role(prev_role);
                return Err(e);
            }
        };

        state.set_leader_epoch(segment.leader_epoch);
        state.set_segment_epoch(segment.segment_epoch);
        fetcher_manager.remove_segment(shard, segment_seq);
        fetcher_manager.assign_segment(SegmentFetchState {
            shard: shard.clone(),
            segment_seq,
            leader_node_id: segment.leader,
            current_leader_epoch: segment.leader_epoch,
            max_bytes,
            cache,
        });
        state.set_role(ReplicaRole::FollowerActive);
        Ok(ReplicaRole::FollowerActive)
    }
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
            client,
        ))
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

        let role = apply_leader_and_isr(&cm, &db, &manager(&cm, &db), &segment(1, &[1, 2], 3))
            .await
            .unwrap();
        assert_eq!(role, ReplicaRole::LeaderActive);

        let state = cm.get_segment_replica("s", 0).unwrap();
        assert_eq!(state.leader_epoch(), 3);
        assert_eq!(state.segment_epoch(), 5);
        let cache = LeaderEpochCache::load(db, "s", 0).unwrap();
        assert_eq!(cache.latest_epoch(), 3);
    }

    #[tokio::test]
    async fn becomes_follower() {
        let engine = test_build_memory_engine();
        let cm = engine.cache_manager.clone();
        let db = rocksdb_engine::test::test_rocksdb_instance();

        let role = apply_leader_and_isr(&cm, &db, &manager(&cm, &db), &segment(2, &[1, 2], 3))
            .await
            .unwrap();
        assert_eq!(role, ReplicaRole::FollowerActive);
        assert_eq!(cm.get_segment_replica("s", 0).unwrap().leader_epoch(), 3);
    }

    #[tokio::test]
    async fn not_a_replica_stays_initializing() {
        let engine = test_build_memory_engine();
        let cm = engine.cache_manager.clone();
        let db = rocksdb_engine::test::test_rocksdb_instance();

        let role = apply_leader_and_isr(&cm, &db, &manager(&cm, &db), &segment(2, &[2, 3], 3))
            .await
            .unwrap();
        assert_eq!(role, ReplicaRole::Initializing);
    }

    #[tokio::test]
    async fn stale_leader_epoch_is_ignored() {
        let engine = test_build_memory_engine();
        let cm = engine.cache_manager.clone();
        let db = rocksdb_engine::test::test_rocksdb_instance();
        let mgr = manager(&cm, &db);

        apply_leader_and_isr(&cm, &db, &mgr, &segment(2, &[1, 2], 5))
            .await
            .unwrap();
        let role = apply_leader_and_isr(&cm, &db, &mgr, &segment(1, &[1, 2], 3))
            .await
            .unwrap();
        assert_eq!(role, ReplicaRole::FollowerActive);
        assert_eq!(cm.get_segment_replica("s", 0).unwrap().leader_epoch(), 5);
    }
}
