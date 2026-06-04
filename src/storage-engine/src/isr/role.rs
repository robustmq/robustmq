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
use crate::isr::leader_epoch::LeaderEpochCache;
use crate::isr::state::ReplicaRole;
use common_config::broker::broker_config;
use metadata_struct::storage::segment::EngineSegment;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn apply_leader_and_isr(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment: &EngineSegment,
) -> Result<ReplicaRole, StorageEngineError> {
    let broker_id = broker_config().broker_id;
    let shard = &segment.shard_name;
    let segment_seq = segment.segment_seq;

    let state = cache_manager.get_or_create_segment_replica(shard, segment_seq);
    let _guard = state.lock_state().await;

    if !segment.is_replica() {
        state.set_role(ReplicaRole::Initializing);
        return Ok(ReplicaRole::Initializing);
    }

    if segment.leader == broker_id {
        state.set_role(ReplicaRole::LeaderInitializing);

        let leo = cache_manager
            .get_offset_state(shard)
            .map(|s| s.latest_offset)
            .unwrap_or(0);
        let mut epoch_cache =
            LeaderEpochCache::load(rocksdb_engine_handler.clone(), shard, segment_seq)?;
        epoch_cache.assign(segment.leader_epoch, leo)?;

        state.reset_follower_progress();
        state.set_leader_epoch(segment.leader_epoch);
        state.set_segment_epoch(segment.segment_epoch);
        state.set_role(ReplicaRole::LeaderActive);
        Ok(ReplicaRole::LeaderActive)
    } else {
        state.set_role(ReplicaRole::FollowerInitializing);
        state.set_leader_epoch(segment.leader_epoch);
        state.set_segment_epoch(segment.segment_epoch);
        state.set_role(ReplicaRole::FollowerActive);
        Ok(ReplicaRole::FollowerActive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_build_memory_engine;
    use metadata_struct::storage::segment::Replica;

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

        let role = apply_leader_and_isr(&cm, &db, &segment(1, &[1, 2], 3))
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

        let role = apply_leader_and_isr(&cm, &db, &segment(2, &[1, 2], 3))
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

        let role = apply_leader_and_isr(&cm, &db, &segment(2, &[2, 3], 3))
            .await
            .unwrap();
        assert_eq!(role, ReplicaRole::Initializing);
    }
}
