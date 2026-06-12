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

// T1 — replication_happy_path
// T2 — leader_switch_triggers_truncation
//
// Both run fully in-process with MemoryStorageEngine; no external services needed.

#[cfg(test)]
mod tests {
    use crate::isr::{make_engine, make_rocksdb, set_broker_id};
    use async_trait::async_trait;
    use bytes::Bytes;
    use common_config::storage::StorageType;
    use dashmap::DashMap;
    use metadata_struct::storage::record::{StorageRecord, StorageRecordMetadata};
    use metadata_struct::storage::segment::EngineSegment;
    use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};
    use protocol::storage::protocol::{
        FetchReqBody, FetchRespBody, OffsetsForLeaderEpochReqBody, OffsetsForLeaderEpochRespBody,
    };
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::sync::Arc;
    use std::time::Duration;
    use storage_engine::commitlog::memory::engine::MemoryStorageEngine;
    use storage_engine::commitlog::offset::ShardOffsetState;
    use storage_engine::core::error::StorageEngineError;
    use storage_engine::isr::fetcher::{
        FetchTransport, ReplicaFetcherThread, SegmentFetchState, SegmentMap,
    };
    use storage_engine::isr::follower::advance_hw;
    use storage_engine::isr::handle_epoch::handle_offsets_for_leader_epoch;
    use storage_engine::isr::handle_fetch::{fetch_one_shard, FetchEngines};
    use storage_engine::isr::leader_epoch::LeaderEpochCache;
    use storage_engine::isr::log::ReplicaLog;

    fn record(data: &str, offset: u64) -> StorageRecord {
        StorageRecord {
            metadata: StorageRecordMetadata {
                offset,
                ..Default::default()
            },
            protocol_data: None,
            data: Bytes::from(data.to_string()),
        }
    }

    /// Build an in-process leader engine (broker 1) for `shard` at `leader_epoch`,
    /// seeded with `records`.
    async fn setup_leader(
        shard: &str,
        leader_epoch: u32,
        records: Vec<StorageRecord>,
    ) -> Arc<MemoryStorageEngine> {
        let leader = make_engine();
        leader.cache_manager.set_shard(EngineShard {
            shard_name: shard.to_string(),
            config: EngineShardConfig {
                storage_type: StorageType::EngineMemory,
                ..Default::default()
            },
            ..Default::default()
        });
        leader.cache_manager.set_segment(&EngineSegment {
            shard_name: shard.to_string(),
            segment_seq: 0,
            leader: 1,
            leader_epoch,
            isr: vec![1],
            ..Default::default()
        });
        leader
            .cache_manager
            .save_offset_state(shard.to_string(), ShardOffsetState::default());
        leader.cache_manager.add_segment_replica(shard, 0);
        set_broker_id(&leader.cache_manager, 1);
        if !records.is_empty() {
            leader
                .as_ref()
                .append_at(shard, 0, 0, records)
                .await
                .unwrap();
        }
        leader
    }

    #[derive(Clone)]
    struct LeaderTransport {
        leader: Arc<MemoryStorageEngine>,
    }

    #[async_trait]
    impl FetchTransport for LeaderTransport {
        async fn fetch(
            &self,
            _: u64,
            req: FetchReqBody,
        ) -> Result<FetchRespBody, StorageEngineError> {
            let mut shards = Vec::new();
            for s in &req.shards {
                shards.push(
                    fetch_one_shard(
                        &self.leader.cache_manager,
                        &self.leader.commit_log_offset.rocksdb_engine_handler,
                        self.leader.as_ref(),
                        req.replica_id,
                        req.replica_broker_epoch,
                        s,
                    )
                    .await,
                );
            }
            Ok(FetchRespBody { shards })
        }

        async fn offsets_for_leader_epoch(
            &self,
            _: u64,
            req: OffsetsForLeaderEpochReqBody,
        ) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError> {
            let engines = FetchEngines {
                memory: self.leader.clone(),
                rocksdb: make_rocksdb(&self.leader),
            };
            Ok(handle_offsets_for_leader_epoch(
                &engines,
                &self.leader.cache_manager,
                &test_rocksdb_instance(),
                &req,
            )
            .await)
        }
    }

    fn make_follower_thread<T: FetchTransport + Clone + 'static>(
        transport: T,
        follower: &Arc<MemoryStorageEngine>,
        shard: &str,
        segment_seq: u32,
        leader_epoch: u32,
    ) -> (ReplicaFetcherThread<T>, SegmentMap) {
        let bc = follower.cache_manager.broker_cache.clone();
        let mut cfg = bc.get_cluster_config();
        cfg.broker_id = 2;
        cfg.storage_runtime.replica_fetch_max_wait_ms = 0;
        cfg.storage_runtime.replica_fetch_backoff_ms = 10;
        bc.set_cluster_config(cfg);
        bc.set_broker_epoch(1);

        let segments: SegmentMap = Arc::new(DashMap::new());
        let cache = LeaderEpochCache::load(test_rocksdb_instance(), shard, segment_seq).unwrap();
        segments.insert(
            (shard.to_string(), segment_seq),
            SegmentFetchState {
                shard: shard.to_string(),
                segment_seq,
                leader_node_id: 1,
                current_leader_epoch: leader_epoch,
                max_bytes: 1024 * 1024,
                cache,
                needs_truncation: false,
            },
        );

        let thread = ReplicaFetcherThread::new(transport, follower.clone(), bc, segments.clone());
        (thread, segments)
    }

    // T1: leader writes 3 records → follower fetch_round catches up → LEO = 3
    #[tokio::test]
    async fn replication_happy_path() {
        let shard = "t1-shard";
        let leader = setup_leader(
            shard,
            1,
            vec![record("a", 0), record("b", 1), record("c", 2)],
        )
        .await;

        let leader_leo = leader.as_ref().latest_offset(shard, 0).unwrap();
        let _ = advance_hw(
            &leader.cache_manager,
            &leader.commit_log_offset,
            shard,
            0,
            &[1],
            1,
            leader_leo,
        );

        let follower = make_engine();
        follower
            .cache_manager
            .save_offset_state(shard.to_string(), ShardOffsetState::default());
        let (thread, _) = make_follower_thread(
            LeaderTransport {
                leader: leader.clone(),
            },
            &follower,
            shard,
            0,
            1,
        );

        for _ in 0..20 {
            thread.fetch_round().await;
            if follower.as_ref().latest_offset(shard, 0).unwrap() == 3 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        assert_eq!(
            follower.as_ref().latest_offset(shard, 0).unwrap(),
            3,
            "follower should catch up to leader LEO"
        );
    }

    // T2: follower has diverged tail (LEO=5), has already received LeaderAndIsr with
    //     new epoch=2. First fetch: fetch_offset=5 > leader LEO=3 → OffsetOutOfRange
    //     → truncate_after_fence → OFLE responds end_offset=3 → follower truncates to 3.
    #[tokio::test]
    async fn leader_switch_triggers_truncation() {
        let shard = "t2-shard";

        // New leader: epoch=2, LEO=3 (offsets 0, 1, 2)
        let new_leader = setup_leader(
            shard,
            2,
            vec![record("a", 0), record("b", 1), record("c", 2)],
        )
        .await;

        // Follower: 5 records (0-4), records 3-4 are the diverged tail from old epoch
        let follower = make_engine();
        follower
            .cache_manager
            .save_offset_state(shard.to_string(), ShardOffsetState::default());
        follower
            .as_ref()
            .append_at(
                shard,
                0,
                0,
                (0u64..5).map(|i| record(&format!("r-{i}"), i)).collect(),
            )
            .await
            .unwrap();
        assert_eq!(follower.as_ref().latest_offset(shard, 0).unwrap(), 5);

        // Follower already received LeaderAndIsr with leader_epoch=2.
        // fetch_offset=5 > leader_leo=3 → OffsetOutOfRange → truncate_after_fence.
        let (thread, _) = make_follower_thread(
            LeaderTransport {
                leader: new_leader.clone(),
            },
            &follower,
            shard,
            0,
            2, // follower knows new leader_epoch=2 (from LeaderAndIsr)
        );

        thread.fetch_round().await;

        assert_eq!(
            follower.as_ref().latest_offset(shard, 0).unwrap(),
            3,
            "follower should truncate diverged tail to offset 3"
        );
    }
}
