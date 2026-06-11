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
use crate::isr::follower::advance_hw;
use crate::isr::follower::update_follower_progress;
use crate::isr::log::ReplicaLog;
use common_base::tools::now_second;
use common_config::broker::broker_config;
use common_config::storage::StorageType;
use metadata_struct::storage::record::StorageRecord;
use protocol::storage::protocol::{
    FetchErrorCode, FetchReqBody, FetchRespBody, FetchShardReq, FetchShardResp,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct FetchEngines {
    pub memory: Arc<MemoryStorageEngine>,
    pub rocksdb: Arc<RocksDBStorageEngine>,
}

pub async fn handle_fetch(
    engines: &FetchEngines,
    cache_manager: &Arc<StorageCacheManager>,
    req: &FetchReqBody,
) -> FetchRespBody {
    let resp = collect(engines, cache_manager, req).await;

    let has_data = resp
        .shards
        .iter()
        .any(|s| records_bytes(&s.records) >= req.min_bytes && !s.records.is_empty());
    if has_data || req.max_wait_ms == 0 {
        return resp;
    }

    sleep(Duration::from_millis(req.max_wait_ms)).await;
    collect(engines, cache_manager, req).await
}

async fn collect(
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
                    cache_manager,
                    engines.memory.as_ref(),
                    req.replica_id,
                    req.replica_broker_epoch,
                    shard_req,
                )
                .await
            }
            Some(StorageType::EngineRocksDB) => {
                fetch_one_shard(
                    cache_manager,
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

pub async fn fetch_one_shard<L: ReplicaLog>(
    cache_manager: &Arc<StorageCacheManager>,
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

    let broker_id = broker_config().broker_id;
    let segment_iden = crate::filesegment::SegmentIdentity::new(&req.shard_name, req.segment_seq);
    if cache_manager
        .get_segment(&segment_iden)
        .is_none_or(|s| s.leader != broker_id)
    {
        resp.error_code = FetchErrorCode::NotLeaderForPartition.as_u32();
        return resp;
    }

    let Some(state) = cache_manager.get_segment_replica(&req.shard_name, req.segment_seq) else {
        resp.error_code = FetchErrorCode::NotLeaderForPartition.as_u32();
        return resp;
    };

    let leader_epoch = cache_manager
        .get_segment(&segment_iden)
        .map(|s| s.leader_epoch)
        .unwrap_or(0);
    resp.leader_epoch = leader_epoch;
    if req.current_leader_epoch < leader_epoch {
        resp.error_code = FetchErrorCode::FencedLeaderEpoch.as_u32();
        return resp;
    }
    if req.current_leader_epoch > leader_epoch {
        resp.error_code = FetchErrorCode::UnknownLeaderEpoch.as_u32();
        cache_manager.mark_reconcile_needed(&req.shard_name, req.segment_seq, 1);
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

    if req.fetch_offset < log_start || req.fetch_offset > leo {
        resp.leader_hw = cache_manager
            .get_offset_state(&req.shard_name)
            .map(|s| s.high_watermark_offset)
            .unwrap_or(0);
        resp.error_code = FetchErrorCode::OffsetOutOfRange.as_u32();
        return resp;
    }

    if !update_follower_progress(
        &state,
        replica_id,
        replica_broker_epoch,
        req.fetch_offset,
        leo,
        now_second(),
    ) {
        resp.leader_hw = cache_manager
            .get_offset_state(&req.shard_name)
            .map(|s| s.high_watermark_offset)
            .unwrap_or(0);
        resp.error_code = FetchErrorCode::StaleBrokerEpoch.as_u32();
        return resp;
    }

    resp.leader_hw = cache_manager
        .get_active_segment(&req.shard_name)
        .and_then(|segment| {
            advance_hw(
                cache_manager,
                &req.shard_name,
                req.segment_seq,
                &segment.isr,
                segment.leader,
                leo,
            )
        })
        .or_else(|| {
            cache_manager
                .get_offset_state(&req.shard_name)
                .map(|s| s.high_watermark_offset)
        })
        .unwrap_or(0);

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
    use crate::commitlog::memory::engine::MemoryStorageEngine;
    use crate::core::test_tool::{test_build_memory_engine, test_init_conf};
    use bytes::Bytes;
    use metadata_struct::storage::record::StorageRecord;
    use metadata_struct::storage::segment::EngineSegment;

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

    async fn setup_leader() -> MemoryStorageEngine {
        test_init_conf();
        let engine = test_build_memory_engine();
        engine.cache_manager.set_segment(&EngineSegment {
            shard_name: "s".to_string(),
            segment_seq: 0,
            leader: 1,
            leader_epoch: 3,
            ..Default::default()
        });
        engine.cache_manager.add_segment_replica("s", 0);
        engine
            .append_at(
                "s",
                0,
                0,
                vec![record(0, "a"), record(1, "b"), record(2, "c")],
            )
            .await
            .unwrap();
        engine
    }

    #[tokio::test]
    async fn leader_serves_records_and_empty_tail() {
        let engine = setup_leader().await;
        let cm = &engine.cache_manager;

        let resp = fetch_one_shard(cm, &engine, 2, 1, &shard_req(3, 1)).await;
        assert_eq!(resp.error_code, FetchErrorCode::None.as_u32());
        assert_eq!(resp.records.len(), 2);
        assert_eq!(resp.leader_leo, 3);

        let resp = fetch_one_shard(cm, &engine, 2, 1, &shard_req(3, 3)).await;
        assert_eq!(resp.error_code, FetchErrorCode::None.as_u32());
        assert!(resp.records.is_empty());
    }

    #[tokio::test]
    async fn fences_reject() {
        let engine = setup_leader().await;
        let cm = &engine.cache_manager;
        let code = |r: FetchShardResp| r.error_code;

        let mut missing = shard_req(3, 0);
        missing.shard_name = "missing".to_string();
        assert_eq!(
            code(fetch_one_shard(cm, &engine, 2, 1, &missing).await),
            FetchErrorCode::NotLeaderForPartition.as_u32()
        );
        assert_eq!(
            code(fetch_one_shard(cm, &engine, 2, 1, &shard_req(2, 1)).await),
            FetchErrorCode::FencedLeaderEpoch.as_u32()
        );
        assert_eq!(
            code(fetch_one_shard(cm, &engine, 2, 1, &shard_req(9, 1)).await),
            FetchErrorCode::UnknownLeaderEpoch.as_u32()
        );
        assert_eq!(
            code(fetch_one_shard(cm, &engine, 2, 1, &shard_req(3, 99)).await),
            FetchErrorCode::OffsetOutOfRange.as_u32()
        );
        fetch_one_shard(cm, &engine, 2, 5, &shard_req(3, 1)).await;
        assert_eq!(
            code(fetch_one_shard(cm, &engine, 2, 3, &shard_req(3, 1)).await),
            FetchErrorCode::StaleBrokerEpoch.as_u32()
        );
    }

    #[tokio::test]
    async fn batched_fetch_returns_per_segment() {
        use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};

        let mem = Arc::new(test_build_memory_engine());
        let engines = FetchEngines {
            memory: mem.clone(),
            rocksdb: Arc::new(crate::core::test_tool::test_build_rocksdb_engine()),
        };

        test_init_conf();
        for shard in ["s1", "s2"] {
            mem.cache_manager.set_shard(EngineShard {
                shard_name: shard.to_string(),
                config: EngineShardConfig {
                    storage_type: StorageType::EngineMemory,
                    ..Default::default()
                },
                ..Default::default()
            });
            mem.cache_manager.set_segment(&EngineSegment {
                shard_name: shard.to_string(),
                segment_seq: 0,
                leader: 1,
                leader_epoch: 1,
                ..Default::default()
            });
            mem.cache_manager.add_segment_replica(shard, 0);
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
        let resp = handle_fetch(&engines, &mem.cache_manager, &req).await;
        assert_eq!(resp.shards.len(), 2);
        assert_eq!(resp.shards[0].records.len(), 2);
        assert!(resp.shards[1].records.is_empty());
        assert_eq!(resp.shards[1].error_code, FetchErrorCode::None.as_u32());
    }
}
