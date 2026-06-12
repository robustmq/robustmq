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

// ISR replication integration tests.
//
// Tests marked `#[ignore]` require a running broker cluster (start with
// `./scripts/run-cluster.sh`) and are run with `cargo test -- --ignored`.
// The in-proc scenario tests run without any external service.

#[cfg(test)]
mod tests {
    // Scenario: leader writes, follower fetches, HW advances, acks=all commits.
    // Full in-proc exercise of the replication happy path.
    #[tokio::test]
    async fn replication_happy_path() {
        use std::sync::Arc;
        use std::time::Duration;

        use async_trait::async_trait;
        use broker_core::cache::NodeCacheManager;
        use bytes::Bytes;
        use common_config::config::BrokerConfig;
        use metadata_struct::storage::record::{StorageRecord, StorageRecordMetadata};
        use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};
        use protocol::storage::protocol::{
            FetchErrorCode, FetchReqBody, FetchRespBody, OffsetsForLeaderEpochReqBody,
            OffsetsForLeaderEpochRespBody,
        };
        use storage_engine::commitlog::memory::engine::MemoryStorageEngine;
        use storage_engine::core::cache::StorageCacheManager;
        use storage_engine::core::error::StorageEngineError;
        use storage_engine::core::shard::ShardOffsetState;
        use storage_engine::isr::fetch::{fetch_one_shard, FetchEngines};
        use storage_engine::isr::fetcher::{
            fetcher_index, FetchTransport, ReplicaFetcherThread, SegmentFetchState, SegmentMap,
        };
        use storage_engine::isr::hw::advance_hw;
        use storage_engine::isr::leader_epoch::LeaderEpochCache;
        use storage_engine::isr::state::ReplicaRole;
        use common_config::storage::StorageType;
        use dashmap::DashMap;
        use rocksdb_engine::test::test_rocksdb_instance;

        fn make_engine() -> Arc<MemoryStorageEngine> {
            let db = test_rocksdb_instance();
            let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
            let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
            Arc::new(MemoryStorageEngine::new(db, cache_manager, Default::default()))
        }

        fn record(offset: u64) -> StorageRecord {
            StorageRecord {
                metadata: StorageRecordMetadata { offset, ..Default::default() },
                protocol_data: None,
                data: Bytes::from(format!("data-{offset}")),
            }
        }

        #[derive(Clone)]
        struct InProcTransport { leader: Arc<MemoryStorageEngine> }
        #[async_trait]
        impl FetchTransport for InProcTransport {
            async fn fetch(&self, _: u64, req: FetchReqBody) -> Result<FetchRespBody, StorageEngineError> {
                let mut shards = Vec::new();
                for s in &req.shards {
                    shards.push(fetch_one_shard(&self.leader.cache_manager, self.leader.as_ref(), req.replica_id, req.replica_broker_epoch, s).await);
                }
                Ok(FetchRespBody { shards })
            }
            async fn offsets_for_leader_epoch(&self, _: u64, req: OffsetsForLeaderEpochReqBody) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError> {
                let engines = FetchEngines { memory: self.leader.clone(), rocksdb: Arc::new(storage_engine::commitlog::rocksdb::engine::RocksDBStorageEngine::new(self.leader.cache_manager.clone(), test_rocksdb_instance())) };
                Ok(storage_engine::isr::offsets_for_leader_epoch::handle_offsets_for_leader_epoch(&engines, &self.leader.cache_manager, &test_rocksdb_instance(), &req).await)
            }
        }

        let leader = make_engine();
        let shard = "test-shard";

        leader.cache_manager.set_shard(EngineShard {
            shard_name: shard.to_string(),
            config: EngineShardConfig { storage_type: StorageType::EngineMemory, ..Default::default() },
            ..Default::default()
        });
        leader.cache_manager.save_offset_state(shard.to_string(), ShardOffsetState::default());
        let leader_state = leader.cache_manager.get_or_create_segment_replica(shard, 0);
        leader_state.set_role(ReplicaRole::LeaderActive);
        leader_state.set_leader_epoch(1);

        leader.append_at(shard, 0, 0, vec![record(0), record(1), record(2)]).await.unwrap();

        let leader_bc = leader.cache_manager.broker_cache.clone();
        let mut cfg = leader_bc.get_cluster_config();
        cfg.broker_id = 1;
        leader_bc.set_cluster_config(cfg);

        let leader_leo = leader.latest_offset(shard, 0).unwrap();
        advance_hw(&leader.cache_manager, shard, 0, &[1], 1, leader_leo);

        let follower = make_engine();
        let follower_bc = follower.cache_manager.broker_cache.clone();
        let mut fcfg = follower_bc.get_cluster_config();
        fcfg.broker_id = 2;
        fcfg.storage_runtime.replica_fetch_max_wait_ms = 0;
        fcfg.storage_runtime.replica_fetch_backoff_ms = 10;
        follower_bc.set_cluster_config(fcfg);
        follower_bc.set_broker_epoch(1);

        let segments: SegmentMap = Arc::new(DashMap::new());
        let mut thread = ReplicaFetcherThread::new(
            InProcTransport { leader: leader.clone() },
            follower.clone(),
            follower_bc,
            segments.clone(),
        );
        let cache = LeaderEpochCache::load(test_rocksdb_instance(), shard, 0).unwrap();
        segments.insert((shard.to_string(), 0), SegmentFetchState {
            shard: shard.to_string(),
            segment_seq: 0,
            leader_node_id: 1,
            current_leader_epoch: 1,
            max_bytes: 1024 * 1024,
            cache,
            needs_truncation: false,
        });

        for _ in 0..20 {
            thread.fetch_round().await;
            if follower.latest_offset(shard, 0).unwrap() == 3 { break; }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        assert_eq!(follower.latest_offset(shard, 0).unwrap(), 3, "follower should catch up");
    }

    // Scenario: leader epoch advances (leader switch), follower gets FencedLeaderEpoch,
    // truncation kicks in, follower re-syncs from the truncation point.
    #[tokio::test]
    async fn leader_switch_triggers_truncation() {
        use std::sync::Arc;
        use std::time::Duration;

        use async_trait::async_trait;
        use broker_core::cache::NodeCacheManager;
        use bytes::Bytes;
        use common_config::config::BrokerConfig;
        use metadata_struct::storage::record::{StorageRecord, StorageRecordMetadata};
        use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};
        use protocol::storage::protocol::{
            FetchErrorCode, FetchReqBody, FetchRespBody, FetchShardResp, OffsetsForLeaderEpochReqBody, OffsetsForLeaderEpochRespBody,
        };
        use storage_engine::commitlog::memory::engine::MemoryStorageEngine;
        use storage_engine::core::cache::StorageCacheManager;
        use storage_engine::core::error::StorageEngineError;
        use storage_engine::core::shard::ShardOffsetState;
        use storage_engine::isr::fetch::FetchEngines;
        use storage_engine::isr::fetcher::{FetchTransport, ReplicaFetcherThread, SegmentFetchState, SegmentMap};
        use storage_engine::isr::leader_epoch::LeaderEpochCache;
        use storage_engine::isr::state::ReplicaRole;
        use common_config::storage::StorageType;
        use dashmap::DashMap;
        use rocksdb_engine::test::test_rocksdb_instance;

        fn make_engine() -> Arc<MemoryStorageEngine> {
            let db = test_rocksdb_instance();
            let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
            let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));
            Arc::new(MemoryStorageEngine::new(db, cache_manager, Default::default()))
        }

        fn record(offset: u64, tag: &str) -> StorageRecord {
            StorageRecord {
                metadata: StorageRecordMetadata { offset, ..Default::default() },
                protocol_data: None,
                data: Bytes::from(format!("{tag}-{offset}")),
            }
        }

        // New leader has epoch=2, epoch 1 data only goes to offset 3.
        #[derive(Clone)]
        struct NewLeader { engine: Arc<MemoryStorageEngine> }
        #[async_trait]
        impl FetchTransport for NewLeader {
            async fn fetch(&self, _: u64, req: FetchReqBody) -> Result<FetchRespBody, StorageEngineError> {
                let shards = req.shards.iter().map(|s| FetchShardResp {
                    shard_name: s.shard_name.clone(),
                    segment_seq: s.segment_seq,
                    error_code: FetchErrorCode::FencedLeaderEpoch.as_u32(),
                    ..Default::default()
                }).collect();
                Ok(FetchRespBody { shards })
            }
            async fn offsets_for_leader_epoch(&self, _: u64, req: OffsetsForLeaderEpochReqBody) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError> {
                let engines = FetchEngines { memory: self.engine.clone(), rocksdb: Arc::new(storage_engine::commitlog::rocksdb::engine::RocksDBStorageEngine::new(self.engine.cache_manager.clone(), test_rocksdb_instance())) };
                Ok(storage_engine::isr::offsets_for_leader_epoch::handle_offsets_for_leader_epoch(&engines, &self.engine.cache_manager, &test_rocksdb_instance(), &req).await)
            }
        }

        let new_leader = make_engine();
        let shard = "ts";
        new_leader.cache_manager.set_shard(EngineShard {
            shard_name: shard.to_string(),
            config: EngineShardConfig { storage_type: StorageType::EngineMemory, ..Default::default() },
            ..Default::default()
        });
        new_leader.cache_manager.save_offset_state(shard.to_string(), ShardOffsetState::default());
        let ls = new_leader.cache_manager.get_or_create_segment_replica(shard, 0);
        ls.set_role(ReplicaRole::LeaderActive);
        ls.set_leader_epoch(2);
        new_leader.append_at(shard, 0, 0, vec![record(0,"a"), record(1,"b"), record(2,"c")]).await.unwrap();
        {
            let mut c = LeaderEpochCache::load(test_rocksdb_instance(), shard, 0).unwrap();
            c.assign(1, 0).unwrap();
            c.assign(2, 3).unwrap();
        }

        let follower = make_engine();
        follower.append_at(shard, 0, 0, vec![record(0,"a"), record(1,"b"), record(2,"c"), record(3,"x"), record(4,"y")]).await.unwrap();
        assert_eq!(follower.latest_offset(shard, 0).unwrap(), 5);

        let follower_bc = follower.cache_manager.broker_cache.clone();
        let mut fcfg = follower_bc.get_cluster_config();
        fcfg.broker_id = 2;
        fcfg.storage_runtime.replica_fetch_max_wait_ms = 0;
        fcfg.storage_runtime.replica_fetch_backoff_ms = 10;
        follower_bc.set_cluster_config(fcfg);
        follower_bc.set_broker_epoch(1);

        let segments: SegmentMap = Arc::new(DashMap::new());
        let mut thread = ReplicaFetcherThread::new(
            NewLeader { engine: new_leader.clone() },
            follower.clone(),
            follower_bc,
            segments.clone(),
        );
        let mut follower_cache = LeaderEpochCache::load(test_rocksdb_instance(), shard, 0).unwrap();
        follower_cache.assign(1, 0).unwrap();
        segments.insert((shard.to_string(), 0), SegmentFetchState {
            shard: shard.to_string(),
            segment_seq: 0,
            leader_node_id: 1,
            current_leader_epoch: 1,
            max_bytes: 1024 * 1024,
            cache: follower_cache,
            needs_truncation: false,
        });

        thread.fetch_round().await;

        assert_eq!(follower.latest_offset(shard, 0).unwrap(), 3,
            "follower should truncate diverged tail to 3");
    }

    // Scenario: elect_recovery_leader selects the replica with the highest LEO.
    // This is a pure unit test of the recovery election logic.
    #[test]
    fn recovery_election_picks_highest_leo() {
        use meta_service::core::isr_recovery::{elect_recovery_leader, ReplicaStateReport};

        let reports = vec![
            ReplicaStateReport { replica_id: 1, segment_leo: 100, latest_leader_epoch: 3, available: true },
            ReplicaStateReport { replica_id: 2, segment_leo: 80,  latest_leader_epoch: 3, available: true },
            ReplicaStateReport { replica_id: 3, segment_leo: 50,  latest_leader_epoch: 3, available: false },
        ];
        assert_eq!(elect_recovery_leader(&reports), Some(1));
    }

    // Placeholder: full cluster kill-leader test (requires running broker cluster).
    #[tokio::test]
    #[ignore = "requires running 3-broker cluster via scripts/run-cluster.sh"]
    async fn kill_leader_triggers_failover_and_truncation() {
        // TODO: start 3-broker cluster, write data with acks=all,
        // kill leader process, verify new leader elected, follower truncates,
        // writes resume, HW advances.
    }

    // Placeholder: reconcile catches up missed LeaderAndIsr notification.
    #[tokio::test]
    #[ignore = "requires running broker+meta cluster"]
    async fn reconcile_recovers_from_missed_notification() {
        // TODO: deliberately drop one LeaderAndIsr broadcast,
        // wait reconcile_interval_ms, verify broker catches up.
    }
}
