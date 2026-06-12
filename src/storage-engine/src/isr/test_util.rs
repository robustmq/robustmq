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

//! Shared test fixtures for the ISR replication tests.
//!
//! `fetcher.rs` and `fetcher_manager.rs` both drive a fetcher against an
//! in-process "leader" that answers fetch / offsets-for-leader-epoch requests
//! by calling the real leader-side handlers. These helpers keep that fake (and
//! the segment/record builders around it) in one place.

use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::core::error::StorageEngineError;
use crate::core::test_tool::{test_build_memory_engine, test_build_rocksdb_engine, test_init_conf};
use crate::isr::fetcher::{FetchTransport, SegmentFetchState};
use crate::isr::handle_epoch::handle_offsets_for_leader_epoch;
use crate::isr::handle_fetch::{fetch_one_shard, FetchEngines};
use crate::isr::leader_epoch::LeaderEpochCache;
use crate::isr::log::ReplicaLog;
use async_trait::async_trait;
use broker_core::cache::NodeCacheManager;
use bytes::Bytes;
use metadata_struct::storage::record::{StorageRecord, StorageRecordMetadata};
use metadata_struct::storage::segment::EngineSegment;
use protocol::storage::protocol::{
    FetchReqBody, FetchRespBody, OffsetsForLeaderEpochReqBody, OffsetsForLeaderEpochRespBody,
};
use rocksdb_engine::test::test_rocksdb_instance;
use std::sync::Arc;

pub fn init_offsets(engine: &MemoryStorageEngine, shards: &[&str]) {
    for shard in shards {
        engine.cache_manager.save_offset_state(
            shard.to_string(),
            crate::commitlog::offset::ShardOffsetState::default(),
        );
    }
}

/// Build a single record carrying `data` at `offset`.
pub fn record(offset: u64, data: &str) -> StorageRecord {
    StorageRecord {
        metadata: StorageRecordMetadata {
            offset,
            ..Default::default()
        },
        protocol_data: None,
        data: Bytes::from(data.to_string()),
    }
}

/// A leader that serves fetch / epoch requests in-process by invoking the real
/// leader-side handlers against a memory engine.
#[derive(Clone)]
pub struct InProcLeader {
    pub engine: Arc<MemoryStorageEngine>,
}

#[async_trait]
impl FetchTransport for InProcLeader {
    async fn fetch(
        &self,
        _leader_node_id: u64,
        req: FetchReqBody,
    ) -> Result<FetchRespBody, StorageEngineError> {
        let mut shards = Vec::with_capacity(req.shards.len());
        for s in &req.shards {
            shards.push(
                fetch_one_shard(
                    &self.engine.cache_manager,
                    self.engine.as_ref(),
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
        _leader_node_id: u64,
        req: OffsetsForLeaderEpochReqBody,
    ) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError> {
        let engines = FetchEngines {
            memory: self.engine.clone(),
            rocksdb: Arc::new(test_build_rocksdb_engine()),
        };
        Ok(handle_offsets_for_leader_epoch(
            &engines,
            &self.engine.cache_manager,
            &test_rocksdb_instance(),
            &req,
        )
        .await)
    }
}

/// Build an in-process leader (broker 1, epoch 1) seeded with the given shards
/// and their records.
pub async fn leader_with(shards: &[(&str, Vec<StorageRecord>)]) -> InProcLeader {
    test_init_conf();
    let engine = Arc::new(test_build_memory_engine());
    for (shard, records) in shards {
        engine.cache_manager.set_segment(&EngineSegment {
            shard_name: shard.to_string(),
            segment_seq: 0,
            leader: 1,
            leader_epoch: 1,
            ..Default::default()
        });
        engine.cache_manager.add_segment_replica(shard, 0);
        engine.cache_manager.save_offset_state(
            shard.to_string(),
            crate::commitlog::offset::ShardOffsetState::default(),
        );
        if !records.is_empty() {
            engine
                .append_at(shard, 0, 0, records.clone())
                .await
                .unwrap();
        }
    }
    InProcLeader { engine }
}

/// A fetch state for segment 0 of `shard` pointing at `leader_node_id`.
pub fn seg_state(shard: &str, leader_node_id: u64) -> SegmentFetchState {
    SegmentFetchState {
        shard: shard.to_string(),
        segment_seq: 0,
        leader_node_id,
        current_leader_epoch: 1,
        max_bytes: 1024 * 1024,
        cache: LeaderEpochCache::load(test_rocksdb_instance(), shard, 0).unwrap(),
    }
}

/// Configure a follower broker cache: broker id 2, no fetch wait, short backoff,
/// broker epoch 1.
pub fn configure_follower_broker_cache(broker_cache: &Arc<NodeCacheManager>) {
    let mut config = broker_cache.get_cluster_config();
    config.broker_id = 2;
    config.storage_runtime.replica_fetch_max_wait_ms = 0;
    config.storage_runtime.replica_fetch_backoff_ms = 5;
    broker_cache.set_cluster_config(config);
    broker_cache.set_broker_epoch(1);
}
