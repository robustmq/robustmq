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

use crate::clients::manager::ClientConnectionManager;
use crate::clients::packet::build_fetch_req;
use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
use crate::core::error::StorageEngineError;
use crate::isr::fetcher::{
    fetcher_index, FetchTransport, ReplicaFetcherThread, SegmentFetchState, SegmentMap,
};
use crate::isr::log::ReplicaLog;
use async_trait::async_trait;
use broker_core::cache::NodeCacheManager;
use dashmap::DashMap;
use metadata_struct::storage::record::StorageRecord;
use protocol::storage::codec::StorageEnginePacket;
use protocol::storage::protocol::{FetchReqBody, FetchRespBody};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub enum EngineReplicaLog {
    Memory(Arc<MemoryStorageEngine>),
    RocksDB(Arc<RocksDBStorageEngine>),
}

#[async_trait]
impl ReplicaLog for EngineReplicaLog {
    async fn append_at(
        &self,
        shard: &str,
        segment_seq: u32,
        base_offset: u64,
        records: Vec<StorageRecord>,
    ) -> Result<(), StorageEngineError> {
        match self {
            EngineReplicaLog::Memory(e) => {
                e.append_at(shard, segment_seq, base_offset, records).await
            }
            EngineReplicaLog::RocksDB(e) => {
                e.append_at(shard, segment_seq, base_offset, records).await
            }
        }
    }

    async fn read_from(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
        max_bytes: u64,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        match self {
            EngineReplicaLog::Memory(e) => e.read_from(shard, segment_seq, offset, max_bytes).await,
            EngineReplicaLog::RocksDB(e) => {
                e.read_from(shard, segment_seq, offset, max_bytes).await
            }
        }
    }

    fn latest_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError> {
        match self {
            EngineReplicaLog::Memory(e) => e.latest_offset(shard, segment_seq),
            EngineReplicaLog::RocksDB(e) => e.latest_offset(shard, segment_seq),
        }
    }

    async fn truncate_to(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        match self {
            EngineReplicaLog::Memory(e) => e.truncate_to(shard, segment_seq, offset).await,
            EngineReplicaLog::RocksDB(e) => e.truncate_to(shard, segment_seq, offset).await,
        }
    }

    async fn clear(&self, shard: &str, segment_seq: u32) -> Result<(), StorageEngineError> {
        match self {
            EngineReplicaLog::Memory(e) => e.clear(shard, segment_seq).await,
            EngineReplicaLog::RocksDB(e) => e.clear(shard, segment_seq).await,
        }
    }

    fn log_start_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError> {
        match self {
            EngineReplicaLog::Memory(e) => e.log_start_offset(shard, segment_seq),
            EngineReplicaLog::RocksDB(e) => e.log_start_offset(shard, segment_seq),
        }
    }
}

/// Sends fetch requests to a leader over the storage RPC connection pool, keyed
/// by `leader_node_id`. The follower side of T8b's `handle_fetch`.
#[derive(Clone)]
pub struct PacketFetchTransport {
    client: Arc<ClientConnectionManager>,
}

impl PacketFetchTransport {
    pub fn new(client: Arc<ClientConnectionManager>) -> Self {
        PacketFetchTransport { client }
    }
}

#[async_trait]
impl FetchTransport for PacketFetchTransport {
    async fn fetch(
        &self,
        leader_node_id: u64,
        req: FetchReqBody,
    ) -> Result<FetchRespBody, StorageEngineError> {
        let packet = StorageEnginePacket::FetchReq(build_fetch_req(req));
        match self.client.read_send(leader_node_id, packet).await? {
            StorageEnginePacket::FetchResp(resp) => Ok(resp.body),
            other => Err(StorageEngineError::CommonErrorStr(format!(
                "fetch to node {leader_node_id} expected FetchResp, got {other}"
            ))),
        }
    }
}

/// A fixed pool of `num_replica_fetchers` fetcher threads. A segment is routed
/// to one thread by `leader_node_id % N`, so all segments sharing a leader land
/// on the same thread and batch into one request. The thread count is fixed
/// regardless of shard count, so thousands of shards never explode into
/// thousands of tasks.
pub struct ReplicaFetcherManager {
    segment_maps: Vec<SegmentMap>,
    stop: broadcast::Sender<bool>,
}

impl ReplicaFetcherManager {
    /// Spawn the pool. `transport` and `log` are cloned into each thread (both
    /// are cheap `Arc`-backed handles). Each thread shares one `SegmentMap` with
    /// the manager, which adds/removes segments directly on role changes.
    pub fn spawn<T, L>(
        num_fetchers: u32,
        transport: T,
        log: L,
        broker_cache: Arc<NodeCacheManager>,
    ) -> Self
    where
        T: FetchTransport + Clone + 'static,
        L: ReplicaLog + Clone + 'static,
    {
        let n = num_fetchers.max(1);
        let (stop, _) = broadcast::channel(1);
        let mut segment_maps = Vec::with_capacity(n as usize);
        for _ in 0..n {
            let segments: SegmentMap = Arc::new(DashMap::new());
            segment_maps.push(segments.clone());
            let mut thread = ReplicaFetcherThread::new(
                transport.clone(),
                log.clone(),
                broker_cache.clone(),
                segments,
            );
            let stop_rx = stop.subscribe();
            tokio::spawn(async move { thread.run(stop_rx).await });
        }
        ReplicaFetcherManager { segment_maps, stop }
    }

    fn map_for(&self, leader_node_id: u64) -> &SegmentMap {
        let idx = fetcher_index(leader_node_id, self.segment_maps.len() as u32);
        &self.segment_maps[idx as usize]
    }

    pub fn assign_segment(&self, state: SegmentFetchState) {
        self.map_for(state.leader_node_id)
            .insert((state.shard.clone(), state.segment_seq), state);
    }

    pub fn remove_segment(&self, shard: &str, segment_seq: u32, leader_node_id: u64) {
        self.map_for(leader_node_id)
            .remove(&(shard.to_string(), segment_seq));
    }

    pub fn thread_count(&self) -> usize {
        self.segment_maps.len()
    }

    pub fn shutdown(&self) {
        let _ = self.stop.send(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_build_memory_engine;
    use crate::isr::fetch::fetch_one_shard;
    use crate::isr::leader_epoch::LeaderEpochCache;
    use crate::isr::state::ReplicaRole;
    use bytes::Bytes;
    use protocol::storage::protocol::FetchReqBody;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::time::Duration;

    fn record(offset: u64, data: &str) -> StorageRecord {
        StorageRecord {
            metadata: metadata_struct::storage::record::StorageRecordMetadata {
                offset,
                ..Default::default()
            },
            protocol_data: None,
            data: Bytes::from(data.to_string()),
        }
    }

    #[derive(Clone)]
    struct InProcLeader {
        engine: Arc<MemoryStorageEngine>,
    }

    #[async_trait]
    impl FetchTransport for InProcLeader {
        async fn fetch(
            &self,
            _leader_node_id: u64,
            req: FetchReqBody,
        ) -> Result<FetchRespBody, StorageEngineError> {
            let mut shards = Vec::new();
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
    }

    async fn leader_with(shards: &[(&str, Vec<StorageRecord>)]) -> InProcLeader {
        let engine = Arc::new(test_build_memory_engine());
        for (shard, records) in shards {
            let st = engine.cache_manager.get_or_create_segment_replica(shard, 0);
            st.set_role(ReplicaRole::LeaderActive);
            st.set_leader_epoch(1);
            if !records.is_empty() {
                engine.append_at(shard, 0, 0, records.clone()).await.unwrap();
            }
        }
        InProcLeader { engine }
    }

    fn seg_state(shard: &str, leader_node_id: u64) -> SegmentFetchState {
        SegmentFetchState {
            shard: shard.to_string(),
            segment_seq: 0,
            leader_node_id,
            current_leader_epoch: 1,
            max_bytes: 1024 * 1024,
            cache: LeaderEpochCache::load(test_rocksdb_instance(), shard, 0).unwrap(),
        }
    }

    #[tokio::test]
    async fn manager_fixed_thread_count_and_routing() {
        let leader = leader_with(&[]).await;
        let follower = EngineReplicaLog::Memory(Arc::new(test_build_memory_engine()));
        let broker_cache = leader.engine.cache_manager.broker_cache.clone();
        let mgr = ReplicaFetcherManager::spawn(4, leader, follower, broker_cache);
        assert_eq!(mgr.thread_count(), 4);
        for leader_node in 0u64..100 {
            assert!(Arc::ptr_eq(
                mgr.map_for(leader_node),
                &mgr.segment_maps[(leader_node % 4) as usize]
            ));
        }
        mgr.shutdown();
    }

    #[tokio::test]
    async fn manager_assign_then_catch_up() {
        let leader = leader_with(&[
            ("s1", vec![record(0, "a"), record(1, "b")]),
            ("s2", vec![record(0, "c")]),
        ])
        .await;

        let follower_engine = Arc::new(test_build_memory_engine());
        let follower = EngineReplicaLog::Memory(follower_engine.clone());
        let broker_cache = follower_engine.cache_manager.broker_cache.clone();
        let mut config = broker_cache.get_cluster_config();
        config.broker_id = 2;
        config.storage_runtime.replica_fetch_max_wait_ms = 0;
        config.storage_runtime.replica_fetch_backoff_ms = 5;
        broker_cache.set_cluster_config(config);
        broker_cache.set_broker_epoch(1);

        let mgr = ReplicaFetcherManager::spawn(2, leader, follower, broker_cache);
        mgr.assign_segment(seg_state("s1", 7));
        mgr.assign_segment(seg_state("s2", 7));

        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(20)).await;
            if follower_engine.latest_offset("s1", 0).unwrap() == 2
                && follower_engine.latest_offset("s2", 0).unwrap() == 1
            {
                break;
            }
        }
        mgr.shutdown();

        assert_eq!(follower_engine.latest_offset("s1", 0).unwrap(), 2);
        assert_eq!(follower_engine.latest_offset("s2", 0).unwrap(), 1);
    }
}
