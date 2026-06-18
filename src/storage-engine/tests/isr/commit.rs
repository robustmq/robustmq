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

// T5 — acks_all_commit
//
// Verifies acks=all semantics end-to-end through batch_write:
//   - single-leader ISR: write commits immediately (HW advances)
//   - multi-replica ISR with lagging follower: write times out

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use bytes::Bytes;
    use common_config::storage::StorageType;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::storage::segment::EngineSegment;
    use metadata_struct::storage::segment::Replica;
    use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};
    use rocksdb_engine::test::test_rocksdb_instance;
    use storage_engine::clients::manager::ClientConnectionManager;
    use storage_engine::commitlog::memory::engine::MemoryStorageEngine;
    use storage_engine::commitlog::offset::ShardOffsetState;
    use storage_engine::commitlog::rocksdb::engine::RocksDBStorageEngine;
    use storage_engine::core::cache::StorageCacheManager;
    use storage_engine::core::write::batch_write;
    use storage_engine::filesegment::write_manager::WriteManager;
    use storage_engine::isr::follower::update_follower_progress;
    use tokio::sync::broadcast;

    const ACKS_ONE: i8 = 1;
    const ACKS_ALL: i8 = -1;

    struct Env {
        write_manager: Arc<WriteManager>,
        cache_manager: Arc<StorageCacheManager>,
        memory: Arc<MemoryStorageEngine>,
        rocksdb: Arc<RocksDBStorageEngine>,
        client: Arc<ClientConnectionManager>,
        shard: String,
    }

    async fn make_env(leader_only: bool) -> Env {
        let db = test_rocksdb_instance();
        let shard = "t5-shard".to_string();

        let memory = Arc::new(MemoryStorageEngine::new(
            db.clone(),
            Arc::new(storage_engine::core::cache::StorageCacheManager::new(
                Arc::new(broker_core::cache::NodeCacheManager::new(Default::default())),
            )),
            Default::default(),
        ));
        let cm = memory.cache_manager.clone();

        let replicas = if leader_only {
            vec![Replica {
                replica_seq: 0,
                node_id: 1,
                fold: String::new(),
            }]
        } else {
            vec![
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
            ]
        };
        let isr: Vec<u64> = replicas.iter().map(|r| r.node_id).collect();

        cm.set_shard(EngineShard {
            shard_name: shard.clone(),
            config: EngineShardConfig {
                storage_type: StorageType::EngineMemory,
                min_in_sync_replicas: if leader_only { 1 } else { 2 },
                ..Default::default()
            },
            ..Default::default()
        });

        cm.set_segment(&EngineSegment {
            shard_name: shard.clone(),
            segment_seq: 0,
            leader: 1,
            leader_epoch: 1,
            isr: isr.clone(),
            replicas: replicas.clone(),
            ..Default::default()
        });
        cm.save_offset_state(shard.clone(), ShardOffsetState::default());

        {
            let bc = cm.broker_cache.clone();
            let mut cfg = bc.get_cluster_config();
            cfg.broker_id = 1;
            cfg.storage_runtime.replica_fetch_max_wait_ms = 200;
            bc.set_cluster_config(cfg);
        }

        cm.add_segment_replica(&shard, 0);
        if !leader_only {
            // Follower 2 exists but has LEO=0 → will not let HW advance
            let state = cm.get_segment_replica(&shard, 0).unwrap();
            update_follower_progress(&state, 2, 1, 0, 0, 0).unwrap();
        }

        let rocksdb = Arc::new(RocksDBStorageEngine::new(cm.clone(), db.clone()));
        let pool = Arc::new(ClientPool::new(100));
        let wm = Arc::new(WriteManager::new(db.clone(), cm.clone(), pool, 3));
        let (stop, _) = broadcast::channel(2);
        wm.start(stop);
        let client = Arc::new(ClientConnectionManager::new(cm.clone(), 8));

        Env {
            write_manager: wm,
            cache_manager: cm,
            memory,
            rocksdb,
            client,
            shard,
        }
    }

    fn records(n: usize) -> Vec<AdapterWriteRecord> {
        (0..n)
            .map(|i| AdapterWriteRecord::new("t5-shard".to_string(), Bytes::from(format!("v{i}"))))
            .collect()
    }

    // T5a: ISR = [leader], acks=all → commits immediately, HW = 3
    #[tokio::test]
    async fn acks_all_single_leader_commits_immediately() {
        let e = make_env(true).await;
        let rows = batch_write(
            &e.write_manager,
            &e.cache_manager,
            &e.memory,
            &e.rocksdb,
            &e.client,
            &e.shard,
            &records(3),
            ACKS_ALL,
            1000,
        )
        .await
        .unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(
            e.cache_manager
                .get_offset_state(&e.shard)
                .unwrap()
                .high_watermark_offset,
            3,
            "HW should advance to 3 immediately with single-leader ISR"
        );
    }

    // T5b: acks=1 always succeeds regardless of follower state
    #[tokio::test]
    async fn acks_one_succeeds_without_waiting() {
        let e = make_env(false).await;
        let rows = batch_write(
            &e.write_manager,
            &e.cache_manager,
            &e.memory,
            &e.rocksdb,
            &e.client,
            &e.shard,
            &records(3),
            ACKS_ONE,
            1000,
        )
        .await
        .unwrap();
        assert_eq!(rows.len(), 3);
    }

    // T5c: ISR = [leader, follower(leo=0)], acks=all → times out
    #[tokio::test]
    async fn acks_all_times_out_when_follower_lags() {
        let e = make_env(false).await;
        let result = tokio::time::timeout(
            Duration::from_secs(5),
            batch_write(
                &e.write_manager,
                &e.cache_manager,
                &e.memory,
                &e.rocksdb,
                &e.client,
                &e.shard,
                &records(3),
                ACKS_ALL,
                1000,
            ),
        )
        .await
        .unwrap();
        assert!(
            result.is_err(),
            "acks=all should time out when follower is behind"
        );
    }
}
