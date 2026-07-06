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

// T6 — apply_leader_and_isr reconcile semantics: idempotent re-apply keeps
// follower progress; a new leader_epoch resets it.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use metadata_struct::storage::segment::{EngineSegment, Replica};
    use rocksdb_engine::rocksdb::RocksDBEngine;
    use rocksdb_engine::test::test_rocksdb_instance;
    use storage_engine::clients::manager::ClientConnectionManager;
    use storage_engine::commitlog::memory::engine::MemoryStorageEngine;
    use storage_engine::core::offset::ShardOffsetState;
    use storage_engine::isr::apply::apply_leader_and_isr;
    use storage_engine::isr::fetcher_manager::{
        build_engine_fetcher_manager, ReplicaFetcherManager,
    };
    use storage_engine::isr::follower::update_follower_progress;

    use crate::isr::{make_engine, make_rocksdb, set_broker_id};

    fn make_fetcher_manager(engine: &Arc<MemoryStorageEngine>) -> Arc<ReplicaFetcherManager> {
        let cm = engine.cache_manager.clone();
        let db = engine.commit_log_offset.rocksdb_engine_handler.clone();
        let client = Arc::new(ClientConnectionManager::new(cm.clone(), 1));
        Arc::new(build_engine_fetcher_manager(
            cm,
            engine.clone(),
            make_rocksdb(engine),
            db,
            client,
        ))
    }

    /// Common setup for the reconcile tests: an engine whose node is leader
    /// (broker 1) with the segment replica state and offset state initialized.
    fn leader_setup() -> (
        Arc<MemoryStorageEngine>,
        Arc<RocksDBEngine>,
        Arc<ReplicaFetcherManager>,
    ) {
        use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};
        let engine = make_engine();
        let cm = engine.cache_manager.clone();
        let mgr = make_fetcher_manager(&engine);
        set_broker_id(&cm, 1);
        cm.set_shard(EngineShard {
            shard_name: "t6-shard".to_string(),
            config: EngineShardConfig::default(),
            ..Default::default()
        });
        cm.set_segment(&EngineSegment {
            shard_name: "t6-shard".to_string(),
            segment_seq: 0,
            leader: 1,
            leader_epoch: 0,
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
        });
        cm.add_segment_replica("t6-shard", 0);
        cm.save_offset_state("t6-shard".to_string(), ShardOffsetState::default());
        (engine, test_rocksdb_instance(), mgr)
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

    // T6a: re-applying the same leader_epoch keeps follower_progress intact.
    #[tokio::test]
    async fn reconcile_apply_is_idempotent() {
        let (engine, db, mgr) = leader_setup();
        let cm = engine.cache_manager.clone();

        let seg = segment_v(1, 2);

        apply_leader_and_isr(&cm, &db, &mgr, &seg).await.unwrap();
        cm.set_segment(&seg);

        let state = cm.get_segment_replica("t6-shard", 0).unwrap();
        update_follower_progress(&state, 2, 1, 5, 10, 0).unwrap();
        assert!(state.contains_key(&2), "follower progress should be seeded");

        apply_leader_and_isr(&cm, &db, &mgr, &seg).await.unwrap();

        let state2 = cm.get_segment_replica("t6-shard", 0).unwrap();
        assert!(
            state2.contains_key(&2),
            "follower progress must survive idempotent re-apply"
        );
    }

    // T6b: a new leader_epoch resets follower_progress — the entry is wiped and
    // reseeded from ISR, dropping the previously recorded leo/broker_epoch.
    #[tokio::test]
    async fn reconcile_new_epoch_resets_progress() {
        let (engine, db, mgr) = leader_setup();
        let cm = engine.cache_manager.clone();

        apply_leader_and_isr(&cm, &db, &mgr, &segment_v(1, 0))
            .await
            .unwrap();

        let state = cm.get_segment_replica("t6-shard", 0).unwrap();
        update_follower_progress(&state, 2, 1, 5, 10, 0).unwrap();
        assert_eq!(state.get(&2).unwrap().leo, 5);

        // epoch 1 -> 2 resets progress
        apply_leader_and_isr(&cm, &db, &mgr, &segment_v(2, 0))
            .await
            .unwrap();

        let state2 = cm.get_segment_replica("t6-shard", 0).unwrap();
        let fp = state2
            .get(&2)
            .expect("follower reseeded from ISR after reset");
        assert_eq!(fp.leo, 0, "recorded leo must be reset");
        assert_eq!(fp.follower_broker_epoch, 0, "broker epoch must be reset");
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
