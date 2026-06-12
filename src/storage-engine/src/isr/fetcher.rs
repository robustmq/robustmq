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
use protocol::storage::protocol::{
    FetchErrorCode, FetchReqBody, FetchRespBody, FetchShardReq, OffsetsForLeaderEpochReqBody,
    OffsetsForLeaderEpochRespBody,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;
use tracing::{info, warn};

#[async_trait]
pub trait FetchTransport: Send + Sync {
    async fn fetch(
        &self,
        leader_node_id: u64,
        req: FetchReqBody,
    ) -> Result<FetchRespBody, StorageEngineError>;

    async fn offsets_for_leader_epoch(
        &self,
        leader_node_id: u64,
        req: OffsetsForLeaderEpochReqBody,
    ) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError>;
}

#[async_trait]
impl FetchTransport for Arc<dyn FetchTransport> {
    async fn fetch(
        &self,
        leader_node_id: u64,
        req: FetchReqBody,
    ) -> Result<FetchRespBody, StorageEngineError> {
        (**self).fetch(leader_node_id, req).await
    }

    async fn offsets_for_leader_epoch(
        &self,
        leader_node_id: u64,
        req: OffsetsForLeaderEpochReqBody,
    ) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError> {
        (**self).offsets_for_leader_epoch(leader_node_id, req).await
    }
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

#[derive(Clone)]
pub struct ReplicaFetcherThread<
    T: FetchTransport + Clone + 'static,
    L: ReplicaLog + Clone + 'static,
> {
    transport: T,
    log: L,
    broker_cache: Arc<NodeCacheManager>,
    segments: SegmentMap,
}

impl<T: FetchTransport + Clone + 'static, L: ReplicaLog + Clone + 'static>
    ReplicaFetcherThread<T, L>
{
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

    pub async fn run(&self, mut stop: broadcast::Receiver<bool>) {
        let backoff_ms = self
            .broker_cache
            .get_cluster_config()
            .storage_runtime
            .replica_fetch_backoff_ms;

        loop {
            tokio::select! {
                biased;
                _ = stop.recv() => return,
                _ = async {
                    let progressed = self.fetch_round().await;
                    if !progressed {
                        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                    }
                } => {}
            }
        }
    }

    pub async fn fetch_round(&self) -> bool {
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

        // Fan out one task per leader: each task fetches and applies independently.
        // Wait for all tasks before the next round to avoid unbounded task growth.
        let mut join_set = tokio::task::JoinSet::new();
        for (leader, shards) in by_leader {
            let worker = self.clone();
            let req = FetchReqBody {
                replica_id,
                replica_broker_epoch,
                min_bytes,
                max_wait_ms,
                shards,
            };
            join_set.spawn(async move {
                let resp = match worker.transport.fetch(leader, req).await {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("fetcher fetch to leader {}: {}", leader, e);
                        return false;
                    }
                };
                let mut progressed = false;
                for shard_resp in resp.shards {
                    match worker.apply_shard_resp(shard_resp).await {
                        Ok(true) => progressed = true,
                        Ok(false) => {}
                        Err(e) => warn!("fetcher apply: {}", e),
                    }
                }
                progressed
            });
        }

        let mut progressed = false;
        while let Some(result) = join_set.join_next().await {
            if result.unwrap_or(false) {
                progressed = true;
            }
        }
        progressed
    }

    async fn apply_shard_resp(
        &self,
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
                // Follower is too far behind (log compacted on leader): wipe and re-pull from scratch.
                self.log.clear(shard, segment_seq).await?;
                if let Some(mut state) = self.segments.get_mut(&key) {
                    state.cache.clear()?;
                }
            } else {
                // fetch_offset > leader LEO: follower is ahead of leader, truncate to align.
                self.truncate_after_fence(&key).await?;
            }
            return Ok(false);
        }

        if resp.error_code == FetchErrorCode::FencedLeaderEpoch.as_u32() {
            self.truncate_after_fence(&key).await?;
            return Ok(false);
        }

        if resp.error_code != FetchErrorCode::None.as_u32() {
            return Ok(false);
        }

        let records = decode_records(&resp.records)?;
        let applied = records.len();
        if applied > 0 {
            match self
                .log
                .append_at(shard, segment_seq, fetch_offset, records)
                .await
            {
                Ok(()) => {
                    // Update epoch cache only after a successful append.
                    if let Some(mut state) = self.segments.get_mut(&key) {
                        if resp.leader_epoch > state.cache.latest_epoch() {
                            state.cache.assign(resp.leader_epoch, fetch_offset)?;
                        }
                    }
                }
                Err(StorageEngineError::OutOfOrder(..)) => {
                    self.truncate_after_fence(&key).await?;
                    return Ok(false);
                }
                Err(e) => return Err(e),
            }
        }
        Ok(applied > 0)
    }

    async fn truncate_after_fence(&self, key: &(String, u32)) -> Result<(), StorageEngineError> {
        let (leader_node_id, current_leader_epoch, follower_leader_epoch) = {
            let Some(state) = self.segments.get(key) else {
                return Ok(());
            };
            (
                state.leader_node_id,
                state.current_leader_epoch,
                state.cache.latest_epoch(),
            )
        };
        let (shard, segment_seq) = (key.0.as_str(), key.1);

        let config = self.broker_cache.get_cluster_config();
        let req = OffsetsForLeaderEpochReqBody {
            shard_name: shard.to_string(),
            segment_seq,
            replica_id: config.broker_id,
            replica_broker_epoch: self.broker_cache.get_broker_epoch(),
            current_leader_epoch,
            follower_leader_epoch,
        };

        let resp = self
            .transport
            .offsets_for_leader_epoch(leader_node_id, req)
            .await?;
        if resp.error_code != FetchErrorCode::None.as_u32() {
            warn!(
                "truncate_after_fence {}/{}: OffsetsForLeaderEpoch returned error_code={}, skipping truncation",
                shard, segment_seq, resp.error_code
            );
            return Ok(());
        }

        if resp.end_offset_epoch < 0 {
            self.log.clear(shard, segment_seq).await?;
            if let Some(mut state) = self.segments.get_mut(key) {
                state.cache.clear()?;
                if resp.current_leader_epoch > 0 {
                    state.current_leader_epoch = resp.current_leader_epoch;
                }
            }
            return Ok(());
        }

        let local_leo = self.log.latest_offset(shard, segment_seq)?;
        if local_leo > resp.end_offset {
            warn!(
                "truncate {}/{}: local_leo={} > leader_end_offset={}, truncating diverged data",
                shard, segment_seq, local_leo, resp.end_offset
            );
            match resp.end_offset.checked_sub(1) {
                Some(keep_to) => self.log.truncate_to(shard, segment_seq, keep_to).await?,
                None => self.log.clear(shard, segment_seq).await?,
            }
        }
        if let Some(mut state) = self.segments.get_mut(key) {
            state
                .cache
                .truncate_from_end_by_epoch(resp.end_offset_epoch as u32)?;
            if resp.current_leader_epoch > 0 {
                let old_epoch = state.current_leader_epoch;
                state.current_leader_epoch = resp.current_leader_epoch;
                if old_epoch != resp.current_leader_epoch {
                    info!(
                        "truncate_after_fence {}/{}: leader_epoch {} -> {}",
                        shard, segment_seq, old_epoch, resp.current_leader_epoch
                    );
                }
            }
        }
        Ok(())
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
    use crate::isr::test_util::{
        configure_follower_broker_cache, init_offsets, leader_with, record, seg_state, InProcLeader,
    };
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::sync::Arc;

    fn thread(
        leader: InProcLeader,
        follower: MemoryStorageEngine,
    ) -> (
        ReplicaFetcherThread<InProcLeader, MemoryStorageEngine>,
        SegmentMap,
    ) {
        let broker_cache = follower.cache_manager.broker_cache.clone();
        configure_follower_broker_cache(&broker_cache);
        let segments: SegmentMap = Arc::new(DashMap::new());
        let th = ReplicaFetcherThread::new(leader, follower, broker_cache, segments.clone());
        (th, segments)
    }

    fn add(segments: &SegmentMap, state: SegmentFetchState) {
        segments.insert((state.shard.clone(), state.segment_seq), state);
    }

    #[tokio::test]
    async fn one_thread_serves_many_shards_in_one_round() {
        let leader = leader_with(&[
            ("s1", vec![record(0, "a"), record(1, "b")]),
            ("s2", vec![record(0, "c")]),
            ("s3", vec![]),
        ])
        .await;
        let follower = test_build_memory_engine();
        init_offsets(&follower, &["s1", "s2", "s3"]);
        let (th, segments) = thread(leader, follower);
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
            leader_with(&[("s1", vec![record(0, "a"), record(1, "b"), record(2, "c")])]).await;
        let follower = test_build_memory_engine();
        init_offsets(&follower, &["s1"]);
        let (th, segments) = thread(leader, follower);
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
            leader_with(&[("s1", vec![record(0, "a")]), ("s2", vec![record(0, "b")])]).await;
        let follower = test_build_memory_engine();
        init_offsets(&follower, &["s1", "s2"]);
        let (th, segments) = thread(leader, follower);
        add(&segments, seg_state("s1", 7));
        add(&segments, seg_state("s2", 7));
        segments.remove(&("s2".to_string(), 0));
        assert_eq!(segments.len(), 1);

        th.fetch_round().await;
        assert_eq!(th.log.latest_offset("s1", 0).unwrap(), 1);
        assert_eq!(th.log.latest_offset("s2", 0).unwrap(), 0);
    }

    #[derive(Clone)]
    struct FencingLeader {
        end_offset: u64,
    }

    #[async_trait]
    impl FetchTransport for FencingLeader {
        async fn fetch(
            &self,
            _leader_node_id: u64,
            req: FetchReqBody,
        ) -> Result<FetchRespBody, StorageEngineError> {
            let shards = req
                .shards
                .iter()
                .map(|s| protocol::storage::protocol::FetchShardResp {
                    shard_name: s.shard_name.clone(),
                    segment_seq: s.segment_seq,
                    error_code: FetchErrorCode::FencedLeaderEpoch.as_u32(),
                    ..Default::default()
                })
                .collect();
            Ok(FetchRespBody { shards })
        }

        async fn offsets_for_leader_epoch(
            &self,
            _leader_node_id: u64,
            _req: OffsetsForLeaderEpochReqBody,
        ) -> Result<OffsetsForLeaderEpochRespBody, StorageEngineError> {
            Ok(OffsetsForLeaderEpochRespBody {
                end_offset_epoch: 1,
                end_offset: self.end_offset,
                error_code: FetchErrorCode::None.as_u32(),
                current_leader_epoch: 2,
            })
        }
    }

    #[tokio::test]
    async fn fenced_follower_truncates_diverged_tail() {
        let follower = test_build_memory_engine();
        init_offsets(&follower, &["s"]);
        follower
            .append_at(
                "s",
                0,
                0,
                vec![
                    record(0, "a"),
                    record(1, "b"),
                    record(2, "c"),
                    record(3, "x"),
                    record(4, "y"),
                ],
            )
            .await
            .unwrap();
        assert_eq!(follower.latest_offset("s", 0).unwrap(), 5);

        let broker_cache = follower.cache_manager.broker_cache.clone();
        broker_cache.set_broker_epoch(1);
        let segments: SegmentMap = Arc::new(DashMap::new());
        let th = ReplicaFetcherThread::new(
            FencingLeader { end_offset: 3 },
            follower,
            broker_cache,
            segments.clone(),
        );

        let mut follower_cache = LeaderEpochCache::load(test_rocksdb_instance(), "s", 0).unwrap();
        follower_cache.assign(1, 0).unwrap();
        add(
            &segments,
            SegmentFetchState {
                shard: "s".to_string(),
                segment_seq: 0,
                leader_node_id: 7,
                current_leader_epoch: 1,
                max_bytes: 1024 * 1024,
                cache: follower_cache,
            },
        );

        th.fetch_round().await;

        assert_eq!(th.log.latest_offset("s", 0).unwrap(), 3);
    }

    #[test]
    fn fetcher_index_groups_by_leader() {
        assert_eq!(fetcher_index(7, 4), 3);
        assert_eq!(fetcher_index(11, 4), 3);
        assert_eq!(fetcher_index(8, 4), 0);
        assert_eq!(fetcher_index(5, 0), 0);
    }
}
