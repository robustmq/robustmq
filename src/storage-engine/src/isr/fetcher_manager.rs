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
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::isr::fetcher::{
    fetcher_index, FetchTransport, ReplicaFetcherThread, SegmentFetchState, SegmentMap,
};
use crate::isr::log::ReplicaLog;
use async_trait::async_trait;
use broker_core::cache::NodeCacheManager;
use common_config::storage::StorageType;
use dashmap::DashMap;
use metadata_struct::storage::record::StorageRecord;
use protocol::storage::codec::StorageEnginePacket;
use protocol::storage::protocol::{
    FetchReqBody, FetchRespBody, OffsetsForLeaderEpochReq, OffsetsForLeaderEpochReqBody,
    OffsetsForLeaderEpochRespBody,
};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

#[derive(Clone)]
pub struct EngineReplicaLog {
    memory: Arc<MemoryStorageEngine>,
    rocksdb: Arc<RocksDBStorageEngine>,
    cache_manager: Arc<StorageCacheManager>,
}

impl EngineReplicaLog {
    pub fn new(
        memory: Arc<MemoryStorageEngine>,
        rocksdb: Arc<RocksDBStorageEngine>,
        cache_manager: Arc<StorageCacheManager>,
    ) -> Self {
        EngineReplicaLog {
            memory,
            rocksdb,
            cache_manager,
        }
    }

    fn is_rocksdb(&self, shard: &str) -> bool {
        self.cache_manager
            .shards
            .get(shard)
            .map(|s| s.config.storage_type == StorageType::EngineRocksDB)
            .unwrap_or(false)
    }
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
        if self.is_rocksdb(shard) {
            self.rocksdb
                .append_at(shard, segment_seq, base_offset, records)
                .await
        } else {
            self.memory
                .append_at(shard, segment_seq, base_offset, records)
                .await
        }
    }

    async fn read_from(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
        max_bytes: u64,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        if self.is_rocksdb(shard) {
            self.rocksdb
                .read_from(shard, segment_seq, offset, max_bytes)
                .await
        } else {
            self.memory
                .read_from(shard, segment_seq, offset, max_bytes)
                .await
        }
    }

    fn latest_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError> {
        if self.is_rocksdb(shard) {
            self.rocksdb.latest_offset(shard, segment_seq)
        } else {
            self.memory.latest_offset(shard, segment_seq)
        }
    }

    async fn truncate_to(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        if self.is_rocksdb(shard) {
            self.rocksdb.truncate_to(shard, segment_seq, offset).await
        } else {
            self.memory.truncate_to(shard, segment_seq, offset).await
        }
    }

    async fn clear(&self, shard: &str, segment_seq: u32) -> Result<(), StorageEngineError> {
        if self.is_rocksdb(shard) {
            self.rocksdb.clear(shard, segment_seq).await
        } else {
            self.memory.clear(shard, segment_seq).await
        }
    }

    fn log_start_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError> {
        if self.is_rocksdb(shard) {
            self.rocksdb.log_start_offset(shard, segment_seq)
        } else {
            self.memory.log_start_offset(shard, segment_seq)
        }
    }
}

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

    async fn offsets_for_leader_epoch(
        &self,
        leader_node_id: u64,
        req: OffsetsForLeaderEpochReqBody,
    ) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError> {
        let packet =
            StorageEnginePacket::OffsetsForLeaderEpochReq(OffsetsForLeaderEpochReq::new(req));
        match self.client.read_send(leader_node_id, packet).await? {
            StorageEnginePacket::OffsetsForLeaderEpochResp(resp) => Ok(resp.body),
            other => Err(StorageEngineError::CommonErrorStr(format!(
                "offsets_for_leader_epoch to node {leader_node_id} expected resp, got {other}"
            ))),
        }
    }
}

struct FetcherSlot {
    segments: SegmentMap,
    stop: broadcast::Sender<bool>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

pub struct ReplicaFetcherManager {
    slots: Vec<FetcherSlot>,
    transport: Arc<dyn FetchTransport>,
    log: Arc<dyn ReplicaLog>,
    broker_cache: Arc<NodeCacheManager>,
}

impl ReplicaFetcherManager {
    pub fn new(
        num_fetchers: u32,
        transport: Arc<dyn FetchTransport>,
        log: Arc<dyn ReplicaLog>,
        broker_cache: Arc<NodeCacheManager>,
    ) -> Self {
        let n = num_fetchers.max(1);
        let slots = (0..n)
            .map(|_| FetcherSlot {
                segments: Arc::new(DashMap::new()),
                stop: broadcast::channel(1).0,
                handle: Mutex::new(None),
            })
            .collect();
        ReplicaFetcherManager {
            slots,
            transport,
            log,
            broker_cache,
        }
    }

    pub fn start(&self) {
        for idx in 0..self.slots.len() {
            self.spawn_thread(idx);
        }
    }

    fn spawn_thread(&self, idx: usize) {
        let slot = &self.slots[idx];
        let thread = ReplicaFetcherThread::new(
            self.transport.clone(),
            self.log.clone(),
            self.broker_cache.clone(),
            slot.segments.clone(),
        );
        let stop_rx = slot.stop.subscribe();
        let handle = tokio::spawn(async move { thread.run(stop_rx).await });
        *slot.handle.lock().unwrap() = Some(handle);
    }

    pub fn stop_thread(&self, idx: usize) {
        if let Some(slot) = self.slots.get(idx) {
            let _ = slot.stop.send(true);
            slot.handle.lock().unwrap().take();
        }
    }

    pub fn restart_thread(&self, idx: usize) {
        self.stop_thread(idx);
        if idx < self.slots.len() {
            self.spawn_thread(idx);
        }
    }

    fn map_for(&self, leader_node_id: u64) -> &SegmentMap {
        let idx = fetcher_index(leader_node_id, self.slots.len() as u32);
        &self.slots[idx as usize].segments
    }

    pub fn assign_segment(&self, state: SegmentFetchState) {
        self.map_for(state.leader_node_id)
            .insert((state.shard.clone(), state.segment_seq), state);
    }

    pub fn remove_segment(&self, shard: &str, segment_seq: u32) {
        let key = (shard.to_string(), segment_seq);
        for slot in &self.slots {
            slot.segments.remove(&key);
        }
    }

    pub fn thread_count(&self) -> usize {
        self.slots.len()
    }

    pub fn shutdown(&self) {
        for idx in 0..self.slots.len() {
            self.stop_thread(idx);
        }
    }
}

pub fn build_engine_fetcher_manager(
    cache_manager: Arc<StorageCacheManager>,
    memory: Arc<MemoryStorageEngine>,
    rocksdb: Arc<RocksDBStorageEngine>,
    client: Arc<ClientConnectionManager>,
) -> ReplicaFetcherManager {
    let num_fetchers = cache_manager
        .broker_cache
        .get_cluster_config()
        .storage_runtime
        .num_replica_fetchers;
    let broker_cache = cache_manager.broker_cache.clone();
    let transport: Arc<dyn FetchTransport> = Arc::new(PacketFetchTransport::new(client));
    let log: Arc<dyn ReplicaLog> = Arc::new(EngineReplicaLog::new(memory, rocksdb, cache_manager));
    ReplicaFetcherManager::new(num_fetchers, transport, log, broker_cache)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::{test_build_memory_engine, test_init_conf};
    use crate::isr::fetch::fetch_one_shard;
    use crate::isr::leader_epoch::LeaderEpochCache;
    use bytes::Bytes;
    use metadata_struct::storage::segment::EngineSegment;
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

        async fn offsets_for_leader_epoch(
            &self,
            _leader_node_id: u64,
            req: OffsetsForLeaderEpochReqBody,
        ) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError> {
            let engines = crate::isr::fetch::FetchEngines {
                memory: self.engine.clone(),
                rocksdb: Arc::new(crate::core::test_tool::test_build_rocksdb_engine()),
            };
            Ok(
                crate::isr::offsets_for_leader_epoch::handle_offsets_for_leader_epoch(
                    &engines,
                    &self.engine.cache_manager,
                    &rocksdb_engine::test::test_rocksdb_instance(),
                    &req,
                )
                .await,
            )
        }
    }

    async fn leader_with(shards: &[(&str, Vec<StorageRecord>)]) -> InProcLeader {
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
            if !records.is_empty() {
                engine
                    .append_at(shard, 0, 0, records.clone())
                    .await
                    .unwrap();
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

    fn follower_log(memory: Arc<MemoryStorageEngine>) -> EngineReplicaLog {
        let cache_manager = memory.cache_manager.clone();
        let rocksdb = Arc::new(RocksDBStorageEngine::new(
            cache_manager.clone(),
            test_rocksdb_instance(),
        ));
        EngineReplicaLog::new(memory, rocksdb, cache_manager)
    }

    #[tokio::test]
    async fn manager_fixed_thread_count_and_routing() {
        let leader = leader_with(&[]).await;
        let follower = follower_log(Arc::new(test_build_memory_engine()));
        let broker_cache = leader.engine.cache_manager.broker_cache.clone();
        let mgr = ReplicaFetcherManager::new(4, Arc::new(leader), Arc::new(follower), broker_cache);
        mgr.start();
        assert_eq!(mgr.thread_count(), 4);
        for leader_node in 0u64..100 {
            assert!(Arc::ptr_eq(
                mgr.map_for(leader_node),
                &mgr.slots[(leader_node % 4) as usize].segments
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
        let follower = follower_log(follower_engine.clone());
        let broker_cache = follower_engine.cache_manager.broker_cache.clone();
        let mut config = broker_cache.get_cluster_config();
        config.broker_id = 2;
        config.storage_runtime.replica_fetch_max_wait_ms = 0;
        config.storage_runtime.replica_fetch_backoff_ms = 5;
        broker_cache.set_cluster_config(config);
        broker_cache.set_broker_epoch(1);

        let mgr = ReplicaFetcherManager::new(2, Arc::new(leader), Arc::new(follower), broker_cache);
        mgr.start();
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

    fn follower_manager(
        leader: InProcLeader,
        follower_engine: Arc<MemoryStorageEngine>,
    ) -> ReplicaFetcherManager {
        let follower = follower_log(follower_engine.clone());
        let broker_cache = follower_engine.cache_manager.broker_cache.clone();
        let mut config = broker_cache.get_cluster_config();
        config.broker_id = 2;
        config.storage_runtime.replica_fetch_max_wait_ms = 0;
        config.storage_runtime.replica_fetch_backoff_ms = 5;
        broker_cache.set_cluster_config(config);
        broker_cache.set_broker_epoch(1);
        ReplicaFetcherManager::new(2, Arc::new(leader), Arc::new(follower), broker_cache)
    }

    async fn wait_offset(engine: &Arc<MemoryStorageEngine>, shard: &str, want: u64) -> bool {
        for _ in 0..50 {
            tokio::time::sleep(Duration::from_millis(20)).await;
            if engine.latest_offset(shard, 0).unwrap() == want {
                return true;
            }
        }
        false
    }

    #[tokio::test]
    async fn restart_thread_keeps_assigned_segments() {
        let leader = leader_with(&[("s1", vec![record(0, "a")])]).await;
        let follower_engine = Arc::new(test_build_memory_engine());
        let mgr = follower_manager(leader, follower_engine.clone());
        mgr.start();

        mgr.assign_segment(seg_state("s1", 7));
        assert!(wait_offset(&follower_engine, "s1", 1).await);

        mgr.restart_thread(fetcher_index(7, 2) as usize);

        mgr.assign_segment(seg_state("s2", 7));
        assert!(wait_offset(&follower_engine, "s1", 1).await);
        mgr.shutdown();
    }

    #[tokio::test]
    async fn stop_thread_halts_only_one_thread() {
        let leader = leader_with(&[("s1", vec![record(0, "a")])]).await;
        let follower_engine = Arc::new(test_build_memory_engine());
        let mgr = follower_manager(leader, follower_engine.clone());
        mgr.start();

        mgr.stop_thread(fetcher_index(7, 2) as usize);
        mgr.assign_segment(seg_state("s1", 7));

        assert!(!wait_offset(&follower_engine, "s1", 1).await);

        mgr.restart_thread(fetcher_index(7, 2) as usize);
        assert!(wait_offset(&follower_engine, "s1", 1).await);
        mgr.shutdown();
    }
}
