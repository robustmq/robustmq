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
use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
use crate::core::cache::StorageCacheManager;
use crate::isr::fetcher::{
    fetcher_index, FetchTransport, ReplicaFetcherThread, SegmentFetchState, SegmentMap,
};
use crate::isr::log::ReplicaLog;
use crate::isr::log_replica::EngineReplicaLog;
use crate::isr::packet_transport::PacketFetchTransport;
use broker_core::cache::NodeCacheManager;
use dashmap::DashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

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

    #[cfg(test)]
    pub fn is_fetching(&self, shard: &str, segment_seq: u32) -> bool {
        let key = (shard.to_string(), segment_seq);
        self.slots.iter().any(|s| s.segments.contains_key(&key))
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
    use crate::core::test_tool::test_build_memory_engine;
    use crate::isr::test_util::{
        configure_follower_broker_cache, leader_with, record, seg_state, InProcLeader,
    };
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::time::Duration;

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
        configure_follower_broker_cache(&broker_cache);

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
        configure_follower_broker_cache(&broker_cache);
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
