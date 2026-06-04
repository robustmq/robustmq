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

use crate::core::error::StorageEngineError;
use crate::isr::leader_epoch::LeaderEpochCache;
use crate::isr::log::ReplicaLog;
use async_trait::async_trait;
use broker_core::cache::NodeCacheManager;
use dashmap::DashMap;
use metadata_struct::storage::record::StorageRecord;
use protocol::storage::protocol::{FetchErrorCode, FetchReqBody, FetchRespBody, FetchShardReq};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::warn;

#[async_trait]
pub trait FetchTransport: Send + Sync {
    async fn fetch(
        &self,
        leader_node_id: u64,
        req: FetchReqBody,
    ) -> Result<FetchRespBody, StorageEngineError>;
}

pub struct SegmentFetchState {
    pub shard: String,
    pub segment_seq: u32,
    pub leader_node_id: u64,
    pub current_leader_epoch: u32,
    pub max_bytes: u64,
    pub cache: LeaderEpochCache,
}

pub type SegmentMap = Arc<DashMap<(String, u32), SegmentFetchState>>;

pub struct ReplicaFetcherThread<T: FetchTransport, L: ReplicaLog> {
    transport: T,
    log: L,
    broker_cache: Arc<NodeCacheManager>,
    segments: SegmentMap,
}

impl<T: FetchTransport, L: ReplicaLog> ReplicaFetcherThread<T, L> {
    pub fn new(
        transport: T,
        log: L,
        broker_cache: Arc<NodeCacheManager>,
        segments: SegmentMap,
    ) -> Self {
        ReplicaFetcherThread {
            transport,
            log,
            broker_cache,
            segments,
        }
    }

    pub async fn run(&mut self, mut stop: broadcast::Receiver<bool>) {
        loop {
            let progressed = tokio::select! {
                biased;
                _ = stop.recv() => return,
                p = self.fetch_round() => p,
            };
            if !progressed {
                let backoff = Duration::from_millis(
                    self.broker_cache
                        .get_cluster_config()
                        .storage_runtime
                        .replica_fetch_backoff_ms,
                );
                tokio::select! {
                    _ = tokio::time::sleep(backoff) => {}
                    _ = stop.recv() => return,
                }
            }
        }
    }

    pub async fn fetch_round(&mut self) -> bool {
        let mut by_leader: HashMap<u64, Vec<FetchShardReq>> = HashMap::new();
        for entry in self.segments.iter() {
            let state = entry.value();
            let fetch_offset = match self.log.latest_offset(&state.shard, state.segment_seq) {
                Ok(v) => v,
                Err(e) => {
                    warn!(
                        "fetcher latest_offset {}/{}: {}",
                        state.shard, state.segment_seq, e
                    );
                    continue;
                }
            };
            by_leader
                .entry(state.leader_node_id)
                .or_default()
                .push(FetchShardReq {
                    shard_name: state.shard.clone(),
                    segment_seq: state.segment_seq,
                    fetch_offset,
                    current_leader_epoch: state.current_leader_epoch,
                    max_bytes: state.max_bytes,
                });
        }

        let config = self.broker_cache.get_cluster_config();
        let replica_id = config.broker_id;
        let replica_broker_epoch = self.broker_cache.get_broker_epoch();
        let min_bytes = config.storage_runtime.replica_fetch_min_bytes;
        let max_wait_ms = config.storage_runtime.replica_fetch_max_wait_ms;

        let mut progressed = false;
        for (leader, shards) in by_leader {
            let req = FetchReqBody {
                replica_id,
                replica_broker_epoch,
                min_bytes,
                max_wait_ms,
                shards,
            };
            let resp = match self.transport.fetch(leader, req).await {
                Ok(r) => r,
                Err(e) => {
                    warn!("fetcher fetch to leader {}: {}", leader, e);
                    continue;
                }
            };
            for shard_resp in resp.shards {
                match self.apply_shard_resp(shard_resp).await {
                    Ok(true) => progressed = true,
                    Ok(false) => {}
                    Err(e) => warn!("fetcher apply: {}", e),
                }
            }
        }
        progressed
    }

    async fn apply_shard_resp(
        &mut self,
        resp: protocol::storage::protocol::FetchShardResp,
    ) -> Result<bool, StorageEngineError> {
        let key = (resp.shard_name.clone(), resp.segment_seq);
        if !self.segments.contains_key(&key) {
            return Ok(false);
        }
        let shard = &resp.shard_name;
        let segment_seq = resp.segment_seq;
        let fetch_offset = self.log.latest_offset(shard, segment_seq)?;

        if resp.error_code == FetchErrorCode::OffsetOutOfRange.as_u32() {
            if fetch_offset < resp.leader_log_start {
                self.log.clear(shard, segment_seq).await?;
                if let Some(mut state) = self.segments.get_mut(&key) {
                    state.cache.clear()?;
                }
            }
            return Ok(false);
        }
        if resp.error_code != FetchErrorCode::None.as_u32() {
            return Ok(false);
        }

        if let Some(mut state) = self.segments.get_mut(&key) {
            if resp.leader_epoch > state.cache.latest_epoch() {
                state.cache.assign(resp.leader_epoch, fetch_offset)?;
            }
        }

        let records = decode_records(&resp.records)?;
        let applied = records.len();
        if applied > 0 {
            self.log
                .append_at(shard, segment_seq, fetch_offset, records)
                .await?;
        }
        Ok(applied > 0)
    }
}

fn decode_records(raw: &[Vec<u8>]) -> Result<Vec<StorageRecord>, StorageEngineError> {
    raw.iter()
        .map(|b| StorageRecord::decode(b).map_err(StorageEngineError::from))
        .collect()
}

pub fn fetcher_index(leader_node_id: u64, num_fetchers: u32) -> u32 {
    if num_fetchers == 0 {
        return 0;
    }
    (leader_node_id % num_fetchers as u64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitlog::memory::engine::MemoryStorageEngine;
    use crate::core::test_tool::test_build_memory_engine;
    use crate::isr::fetch::fetch_one_shard;
    use crate::isr::state::ReplicaRole;
    use bytes::Bytes;
    use metadata_struct::storage::record::StorageRecord;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::sync::Arc;

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

    async fn leader_with_shards(shards: &[(&str, Vec<StorageRecord>)]) -> InProcLeader {
        let engine = Arc::new(test_build_memory_engine());
        for (shard, records) in shards {
            let st = engine.cache_manager.get_or_create_segment_replica(shard, 0);
            st.set_role(ReplicaRole::LeaderActive);
            st.set_leader_epoch(1);
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

    fn thread(
        leader: InProcLeader,
        follower: MemoryStorageEngine,
    ) -> (
        ReplicaFetcherThread<InProcLeader, MemoryStorageEngine>,
        SegmentMap,
    ) {
        let broker_cache = follower.cache_manager.broker_cache.clone();
        let mut config = broker_cache.get_cluster_config();
        config.broker_id = 2;
        config.storage_runtime.replica_fetch_max_wait_ms = 0;
        config.storage_runtime.replica_fetch_backoff_ms = 10;
        broker_cache.set_cluster_config(config);
        broker_cache.set_broker_epoch(1);
        let segments: SegmentMap = Arc::new(DashMap::new());
        let th = ReplicaFetcherThread::new(leader, follower, broker_cache, segments.clone());
        (th, segments)
    }

    fn add(segments: &SegmentMap, state: SegmentFetchState) {
        segments.insert((state.shard.clone(), state.segment_seq), state);
    }

    #[tokio::test]
    async fn one_thread_serves_many_shards_in_one_round() {
        let leader = leader_with_shards(&[
            ("s1", vec![record(0, "a"), record(1, "b")]),
            ("s2", vec![record(0, "c")]),
            ("s3", vec![]),
        ])
        .await;
        let follower = test_build_memory_engine();
        let (mut th, segments) = thread(leader, follower);
        add(&segments, seg_state("s1", 7));
        add(&segments, seg_state("s2", 7));
        add(&segments, seg_state("s3", 7));

        let progressed = th.fetch_round().await;
        assert!(progressed);
        assert_eq!(th.log.latest_offset("s1", 0).unwrap(), 2);
        assert_eq!(th.log.latest_offset("s2", 0).unwrap(), 1);
        assert_eq!(th.log.latest_offset("s3", 0).unwrap(), 0);
    }

    #[tokio::test]
    async fn run_loop_catches_up_then_stops() {
        let leader =
            leader_with_shards(&[("s1", vec![record(0, "a"), record(1, "b"), record(2, "c")])])
                .await;
        let follower = test_build_memory_engine();
        let (mut th, segments) = thread(leader, follower);
        add(&segments, seg_state("s1", 7));

        let (stop_tx, stop_rx) = broadcast::channel(1);
        let stopper = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(60)).await;
            let _ = stop_tx.send(true);
        });
        th.run(stop_rx).await;
        stopper.await.unwrap();

        assert_eq!(th.log.latest_offset("s1", 0).unwrap(), 3);
    }

    #[tokio::test]
    async fn remove_segment_stops_fetching_it() {
        let leader =
            leader_with_shards(&[("s1", vec![record(0, "a")]), ("s2", vec![record(0, "b")])]).await;
        let follower = test_build_memory_engine();
        let (mut th, segments) = thread(leader, follower);
        add(&segments, seg_state("s1", 7));
        add(&segments, seg_state("s2", 7));
        segments.remove(&("s2".to_string(), 0));
        assert_eq!(segments.len(), 1);

        th.fetch_round().await;
        assert_eq!(th.log.latest_offset("s1", 0).unwrap(), 1);
        assert_eq!(th.log.latest_offset("s2", 0).unwrap(), 0);
    }

    #[test]
    fn fetcher_index_groups_by_leader() {
        assert_eq!(fetcher_index(7, 4), 3);
        assert_eq!(fetcher_index(11, 4), 3);
        assert_eq!(fetcher_index(8, 4), 0);
        assert_eq!(fetcher_index(5, 0), 0);
    }
}
