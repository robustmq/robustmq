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

// T6 — reconcile_idempotent
//
// Verifies that apply_leader_and_isr is idempotent: re-applying the same
// segment (same leader_epoch + segment_epoch) does NOT reset follower progress.
// This guards against a bug where a reconcile loop triggered a spurious progress
// wipe every interval.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use grpc_clients::pool::ClientPool;
    use metadata_struct::storage::segment::{EngineSegment, Replica};
    use rocksdb_engine::test::test_rocksdb_instance;
    use storage_engine::clients::manager::ClientConnectionManager;
    use storage_engine::commitlog::memory::engine::MemoryStorageEngine;
    use storage_engine::commitlog::rocksdb::engine::RocksDBStorageEngine;
    use storage_engine::core::shard::ShardOffsetState;
    use storage_engine::isr::fetcher_manager::build_engine_fetcher_manager;
    use storage_engine::isr::follower::update_follower_progress;
    use storage_engine::isr::role::apply_leader_and_isr;

    use crate::isr::make_engine;

    fn make_fetcher_manager(
        engine: &Arc<MemoryStorageEngine>,
    ) -> Arc<storage_engine::isr::fetcher_manager::ReplicaFetcherManager> {
        let db = test_rocksdb_instance();
        let cm = engine.cache_manager.clone();
        let rocksdb = Arc::new(RocksDBStorageEngine::new(cm.clone(), db));
        let _pool = Arc::new(ClientPool::new(1));
        let client = Arc::new(ClientConnectionManager::new(cm.clone(), 1));
        Arc::new(build_engine_fetcher_manager(
            cm,
            engine.clone(),
            rocksdb,
            client,
        ))
    }

    fn segment_v(leader_epoch: u32, segment_epoch: u32) -> EngineSegment {
        EngineSegment {
            shard_name: "t6-shard".to_string(),
            segment_seq: 0,
            leader: 1,
            leader_epoch,
            segment_epoch,
            replicas: vec![
                Replica {
                    replica_seq: 0,
                    node_id: 1,
                    fold: String::new(),
                },
                Replica {
                    replica_seq: 1,
                    node_id: 2,
                    fold: String::new(),
                },
            ],
            isr: vec![1, 2],
            ..Default::default()
        }
    }

    // T6: second apply with same leader_epoch keeps follower_progress intact.
    #[tokio::test]
    async fn reconcile_apply_is_idempotent() {
        let engine = make_engine();
        let cm = engine.cache_manager.clone();
        let db = test_rocksdb_instance();
        let mgr = make_fetcher_manager(&engine);

        // broker_id must match segment.leader so we become LeaderActive
        {
            let bc = cm.broker_cache.clone();
            let mut cfg = bc.get_cluster_config();
            cfg.broker_id = 1;
            bc.set_cluster_config(cfg);
        }

        let seg = segment_v(1, 2);
        cm.add_segment_replica("t6-shard", 0);
        cm.save_offset_state("t6-shard".to_string(), ShardOffsetState::default());

        // First apply: becomes leader, follower_progress is empty
        apply_leader_and_isr(&cm, &db, &mgr, &seg).await.unwrap();
        // simulate dynamic_cache.rs: set_segment is called after apply
        cm.set_segment(&seg);

        // Simulate follower 2 starting to fetch
        let state = cm.get_segment_replica("t6-shard", 0).unwrap();
        update_follower_progress(&state, 2, 1, 5, 10, 0);
        assert!(state.contains_key(&2), "follower progress should be seeded");

        // Second apply with identical segment (same leader_epoch=1, segment_epoch=2)
        apply_leader_and_isr(&cm, &db, &mgr, &seg).await.unwrap();

        // follower_progress must NOT be cleared (leader_epoch did not change)
        let state2 = cm.get_segment_replica("t6-shard", 0).unwrap();
        assert!(
            state2.contains_key(&2),
            "follower progress must survive idempotent re-apply"
        );
    }

    // T6b: new leader_epoch DOES clear follower_progress (expected reset on leader switch)
    #[tokio::test]
    async fn reconcile_new_epoch_resets_progress() {
        let engine = make_engine();
        let cm = engine.cache_manager.clone();
        let db = test_rocksdb_instance();
        let mgr = make_fetcher_manager(&engine);

        {
            let bc = cm.broker_cache.clone();
            let mut cfg = bc.get_cluster_config();
            cfg.broker_id = 1;
            bc.set_cluster_config(cfg);
        }

        cm.add_segment_replica("t6-shard", 0);
        cm.save_offset_state("t6-shard".to_string(), ShardOffsetState::default());

        // First apply: epoch=1
        apply_leader_and_isr(&cm, &db, &mgr, &segment_v(1, 0))
            .await
            .unwrap();

        let state = cm.get_segment_replica("t6-shard", 0).unwrap();
        update_follower_progress(&state, 2, 1, 5, 10, 0);
        assert!(state.contains_key(&2));

        // Second apply: epoch=2 → leader_epoch changed → should reset
        apply_leader_and_isr(&cm, &db, &mgr, &segment_v(2, 0))
            .await
            .unwrap();

        let state2 = cm.get_segment_replica("t6-shard", 0).unwrap();
        assert!(
            state2.is_empty(),
            "leader_epoch change should reset follower progress"
        );
    }

    // Cluster-level placeholder: reconcile catches up a broker that missed a
    // LeaderAndIsr notification. Requires a live 3-broker cluster.
    #[tokio::test]
    #[ignore = "requires running broker+meta cluster (scripts/run-cluster.sh)"]
    async fn reconcile_recovers_from_missed_notification() {
        // TODO: deliberately drop one LeaderAndIsr broadcast,
        // wait reconcile_interval_ms, verify broker catches up via reconcile loop.
    }
}
