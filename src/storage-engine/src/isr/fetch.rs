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

use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
use crate::core::cache::StorageCacheManager;
use crate::isr::log::ReplicaLog;
use crate::isr::state::{ReplicaRole, ReplicaStateRegistry};
use common_base::tools::now_second;
use common_config::storage::StorageType;
use metadata_struct::storage::record::StorageRecord;
use protocol::storage::protocol::{
    FetchErrorCode, FetchReqBody, FetchRespBody, FetchShardReq, FetchShardResp,
};
use std::sync::Arc;
use std::time::Duration;

pub struct FetchEngines {
    pub memory: Arc<MemoryStorageEngine>,
    pub rocksdb: Arc<RocksDBStorageEngine>,
}

/// Batched fetch across shards: long-poll returns once any shard has
/// >= min_bytes, else waits max_wait_ms and collects once more (§6.2).
pub async fn handle_fetch(
    registry: &Arc<ReplicaStateRegistry>,
    engines: &FetchEngines,
    cache_manager: &Arc<StorageCacheManager>,
    req: &FetchReqBody,
) -> FetchRespBody {
    let resp = collect(registry, engines, cache_manager, req).await;

    let has_data = resp
        .shards
        .iter()
        .any(|s| records_bytes(&s.records) >= req.min_bytes && !s.records.is_empty());
    if has_data || req.max_wait_ms == 0 {
        return resp;
    }

    // T9/T11 will wake this on append; for now just wait out the window.
    tokio::time::sleep(Duration::from_millis(req.max_wait_ms)).await;
    collect(registry, engines, cache_manager, req).await
}

async fn collect(
    registry: &Arc<ReplicaStateRegistry>,
    engines: &FetchEngines,
    cache_manager: &Arc<StorageCacheManager>,
    req: &FetchReqBody,
) -> FetchRespBody {
    let mut shards = Vec::with_capacity(req.shards.len());
    for shard_req in &req.shards {
        let storage_type = cache_manager
            .shards
            .get(&shard_req.shard_name)
            .map(|s| s.config.storage_type);
        let shard_resp = match storage_type {
            Some(StorageType::EngineMemory) => {
                fetch_one_shard(
                    registry,
                    engines.memory.as_ref(),
                    req.replica_id,
                    req.replica_broker_epoch,
                    shard_req,
                )
                .await
            }
            Some(StorageType::EngineRocksDB) => {
                fetch_one_shard(
                    registry,
                    engines.rocksdb.as_ref(),
                    req.replica_id,
                    req.replica_broker_epoch,
                    shard_req,
                )
                .await
            }
            _ => FetchShardResp {
                shard_name: shard_req.shard_name.clone(),
                segment_seq: shard_req.segment_seq,
                error_code: FetchErrorCode::NotLeaderForPartition.as_u32(),
                ..Default::default()
            },
        };
        shards.push(shard_resp);
    }
    FetchRespBody { shards }
}

fn records_bytes(records: &[Vec<u8>]) -> u64 {
    records.iter().map(|r| r.len() as u64).sum()
}

/// Leader-side fetch for one shard with the full five-fence sequence (§6.2).
/// Rejections are carried in `error_code`, not returned as errors. HW advance
/// (step 5) lands in T11b; `last_caught_up_ts` precise semantics land in T9.
pub async fn fetch_one_shard<L: ReplicaLog>(
    registry: &Arc<ReplicaStateRegistry>,
    log: &L,
    replica_id: u64,
    replica_broker_epoch: u64,
    req: &FetchShardReq,
) -> FetchShardResp {
    let mut resp = FetchShardResp {
        shard_name: req.shard_name.clone(),
        segment_seq: req.segment_seq,
        ..Default::default()
    };

    let Some(state) = registry.get_segment(&req.shard_name, req.segment_seq) else {
        resp.error_code = FetchErrorCode::NotLeaderForPartition.as_u32();
        return resp;
    };

    if state.role() != ReplicaRole::LeaderActive {
        resp.error_code = FetchErrorCode::NotLeaderForPartition.as_u32();
        return resp;
    }

    let leader_epoch = state.leader_epoch();
    resp.leader_epoch = leader_epoch;
    if req.current_leader_epoch < leader_epoch {
        resp.error_code = FetchErrorCode::FencedLeaderEpoch.as_u32();
        return resp;
    }
    if req.current_leader_epoch > leader_epoch {
        resp.error_code = FetchErrorCode::UnknownLeaderEpoch.as_u32();
        return resp;
    }

    let leo = match log.latest_offset(&req.shard_name, req.segment_seq) {
        Ok(v) => v,
        Err(_) => {
            resp.error_code = FetchErrorCode::OffsetOutOfRange.as_u32();
            return resp;
        }
    };
    let log_start = log
        .log_start_offset(&req.shard_name, req.segment_seq)
        .unwrap_or(0);
    resp.leader_leo = leo;
    resp.leader_log_start = log_start;
    resp.leader_hw = registry
        .get_or_create_shard(&req.shard_name)
        .local_hw
        .load(std::sync::atomic::Ordering::SeqCst);

    if req.fetch_offset < log_start || req.fetch_offset > leo {
        resp.error_code = FetchErrorCode::OffsetOutOfRange.as_u32();
        return resp;
    }

    let now = now_second();
    {
        let mut progress = state.follower_progress.entry(replica_id).or_default();
        if replica_broker_epoch < progress.broker_epoch {
            resp.error_code = FetchErrorCode::StaleBrokerEpoch.as_u32();
            return resp;
        }
        progress.broker_epoch = replica_broker_epoch;
        progress.last_known_leader_epoch = req.current_leader_epoch;
        progress.leo = req.fetch_offset;
        progress.last_fetch_ts = now;
        if req.fetch_offset >= leo {
            progress.last_caught_up_ts = now;
        }
    }

    match log
        .read_from(
            &req.shard_name,
            req.segment_seq,
            req.fetch_offset,
            req.max_bytes,
        )
        .await
    {
        Ok(records) => resp.records = encode_records(&records),
        Err(_) => resp.error_code = FetchErrorCode::OffsetOutOfRange.as_u32(),
    }
    resp
}

fn encode_records(records: &[StorageRecord]) -> Vec<Vec<u8>> {
    records.iter().filter_map(|r| r.encode().ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_build_memory_engine;
    use crate::isr::state::ReplicaStateRegistry;
    use bytes::Bytes;
    use metadata_struct::storage::record::StorageRecord;

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

    fn shard_req(epoch: u32, fetch_offset: u64) -> FetchShardReq {
        FetchShardReq {
            shard_name: "s".to_string(),
            segment_seq: 0,
            fetch_offset,
            current_leader_epoch: epoch,
            max_bytes: 1024 * 1024,
        }
    }

    async fn setup_leader() -> (
        Arc<ReplicaStateRegistry>,
        crate::commitlog::memory::engine::MemoryStorageEngine,
    ) {
        let reg = Arc::new(ReplicaStateRegistry::new());
        let engine = test_build_memory_engine();
        let state = reg.get_or_create_segment("s", 0);
        state.set_role(ReplicaRole::LeaderActive);
        state.set_leader_epoch(3);
        engine
            .append_at(
                "s",
                0,
                0,
                vec![record(0, "a"), record(1, "b"), record(2, "c")],
            )
            .await
            .unwrap();
        (reg, engine)
    }

    #[tokio::test]
    async fn leader_returns_records_from_offset() {
        let (reg, engine) = setup_leader().await;
        let resp = fetch_one_shard(&reg, &engine, 2, 1, &shard_req(3, 1)).await;
        assert_eq!(resp.error_code, FetchErrorCode::None.as_u32());
        assert_eq!(resp.records.len(), 2);
        assert_eq!(resp.leader_leo, 3);
    }

    #[tokio::test]
    async fn rejects_non_leader() {
        let reg = Arc::new(ReplicaStateRegistry::new());
        let engine = test_build_memory_engine();
        reg.get_or_create_segment("s", 0);
        let resp = fetch_one_shard(&reg, &engine, 2, 1, &shard_req(3, 0)).await;
        assert_eq!(
            resp.error_code,
            FetchErrorCode::NotLeaderForPartition.as_u32()
        );
    }

    #[tokio::test]
    async fn rejects_stale_leader_epoch() {
        let (reg, engine) = setup_leader().await;
        let resp = fetch_one_shard(&reg, &engine, 2, 1, &shard_req(2, 1)).await;
        assert_eq!(resp.error_code, FetchErrorCode::FencedLeaderEpoch.as_u32());
    }

    #[tokio::test]
    async fn follower_ahead_returns_unknown_epoch() {
        let (reg, engine) = setup_leader().await;
        let resp = fetch_one_shard(&reg, &engine, 2, 1, &shard_req(9, 1)).await;
        assert_eq!(resp.error_code, FetchErrorCode::UnknownLeaderEpoch.as_u32());
    }

    #[tokio::test]
    async fn rejects_offset_out_of_range() {
        let (reg, engine) = setup_leader().await;
        let resp = fetch_one_shard(&reg, &engine, 2, 1, &shard_req(3, 99)).await;
        assert_eq!(resp.error_code, FetchErrorCode::OffsetOutOfRange.as_u32());
    }

    #[tokio::test]
    async fn rejects_stale_broker_epoch() {
        let (reg, engine) = setup_leader().await;
        fetch_one_shard(&reg, &engine, 2, 5, &shard_req(3, 1)).await;
        let resp = fetch_one_shard(&reg, &engine, 2, 3, &shard_req(3, 1)).await;
        assert_eq!(resp.error_code, FetchErrorCode::StaleBrokerEpoch.as_u32());
    }

    #[tokio::test]
    async fn empty_tail_at_leo() {
        let (reg, engine) = setup_leader().await;
        let resp = fetch_one_shard(&reg, &engine, 2, 1, &shard_req(3, 3)).await;
        assert_eq!(resp.error_code, FetchErrorCode::None.as_u32());
        assert!(resp.records.is_empty());
    }

    #[tokio::test]
    async fn batched_fetch_returns_per_segment() {
        use crate::isr::state::ReplicaRole;
        use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};

        let reg = Arc::new(ReplicaStateRegistry::new());
        let mem = Arc::new(test_build_memory_engine());
        let engines = FetchEngines {
            memory: mem.clone(),
            rocksdb: Arc::new(crate::core::test_tool::test_build_rocksdb_engine()),
        };

        for shard in ["s1", "s2"] {
            mem.cache_manager.set_shard(EngineShard {
                shard_name: shard.to_string(),
                config: EngineShardConfig {
                    storage_type: StorageType::EngineMemory,
                    ..Default::default()
                },
                ..Default::default()
            });
            let st = reg.get_or_create_segment(shard, 0);
            st.set_role(ReplicaRole::LeaderActive);
            st.set_leader_epoch(1);
        }
        mem.append_at("s1", 0, 0, vec![record(0, "a"), record(1, "b")])
            .await
            .unwrap();

        let req = FetchReqBody {
            replica_id: 2,
            replica_broker_epoch: 1,
            min_bytes: 1,
            max_wait_ms: 0,
            shards: vec![
                FetchShardReq {
                    shard_name: "s1".to_string(),
                    segment_seq: 0,
                    fetch_offset: 0,
                    current_leader_epoch: 1,
                    max_bytes: 1024 * 1024,
                },
                FetchShardReq {
                    shard_name: "s2".to_string(),
                    segment_seq: 0,
                    fetch_offset: 0,
                    current_leader_epoch: 1,
                    max_bytes: 1024 * 1024,
                },
            ],
        };
        let resp = handle_fetch(&reg, &engines, &mem.cache_manager, &req).await;
        assert_eq!(resp.shards.len(), 2);
        assert_eq!(resp.shards[0].records.len(), 2);
        assert!(resp.shards[1].records.is_empty());
        assert_eq!(resp.shards[1].error_code, FetchErrorCode::None.as_u32());
    }
}
