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

use crate::core::cache::StorageCacheManager;
use crate::isr::fetch::FetchEngines;
use crate::isr::leader_epoch::LeaderEpochCache;
use crate::isr::log::ReplicaLog;
use crate::isr::state::ReplicaRole;
use common_config::storage::StorageType;
use protocol::storage::protocol::{
    FetchErrorCode, OffsetsForLeaderEpochReqBody, OffsetsForLeaderEpochRespBody,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn handle_offsets_for_leader_epoch(
    engines: &FetchEngines,
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req: &OffsetsForLeaderEpochReqBody,
) -> OffsetsForLeaderEpochRespBody {
    let mut resp = OffsetsForLeaderEpochRespBody {
        end_offset_epoch: -1,
        end_offset: 0,
        error_code: FetchErrorCode::None.as_u32(),
        current_leader_epoch: 0,
    };

    let Some(state) = cache_manager.get_segment_replica(&req.shard_name, req.segment_seq) else {
        resp.error_code = FetchErrorCode::NotLeaderForPartition.as_u32();
        return resp;
    };
    if state.role() != ReplicaRole::LeaderActive {
        resp.error_code = FetchErrorCode::NotLeaderForPartition.as_u32();
        return resp;
    }

    let leader_epoch = state.leader_epoch();
    resp.current_leader_epoch = leader_epoch;
    if req.current_leader_epoch < leader_epoch {
        resp.error_code = FetchErrorCode::FencedLeaderEpoch.as_u32();
        return resp;
    }
    if req.current_leader_epoch > leader_epoch {
        resp.error_code = FetchErrorCode::UnknownLeaderEpoch.as_u32();
        return resp;
    }

    let leader_leo = match leo_for(engines, cache_manager, &req.shard_name, req.segment_seq) {
        Some(v) => v,
        None => {
            resp.error_code = FetchErrorCode::OffsetOutOfRange.as_u32();
            return resp;
        }
    };

    let cache = match LeaderEpochCache::load(
        rocksdb_engine_handler.clone(),
        &req.shard_name,
        req.segment_seq,
    ) {
        Ok(c) => c,
        Err(_) => {
            resp.error_code = FetchErrorCode::OffsetOutOfRange.as_u32();
            return resp;
        }
    };

    if req.follower_leader_epoch > cache.latest_epoch() {
        resp.end_offset_epoch = -1;
        resp.end_offset = leader_leo;
        return resp;
    }

    match cache.end_offset_for(req.follower_leader_epoch) {
        Some(next_start) => {
            resp.end_offset_epoch = req.follower_leader_epoch as i32;
            resp.end_offset = next_start;
        }
        None => {
            resp.end_offset_epoch = cache.latest_epoch() as i32;
            resp.end_offset = leader_leo;
        }
    }
    resp
}

fn leo_for(
    engines: &FetchEngines,
    cache_manager: &Arc<StorageCacheManager>,
    shard: &str,
    segment_seq: u32,
) -> Option<u64> {
    match storage_type_of(cache_manager, shard) {
        Some(StorageType::EngineRocksDB) => engines.rocksdb.latest_offset(shard, segment_seq).ok(),
        Some(StorageType::EngineMemory) => engines.memory.latest_offset(shard, segment_seq).ok(),
        _ => None,
    }
}

fn storage_type_of(cache_manager: &Arc<StorageCacheManager>, shard: &str) -> Option<StorageType> {
    cache_manager
        .shards
        .get(shard)
        .map(|s| s.config.storage_type)
}

pub struct LocalReplicaState {
    pub segment_leo: u64,
    pub latest_leader_epoch: u32,
    pub log_start_offset: u64,
    pub available: bool,
}

pub fn query_local_replica_state(
    engines: &FetchEngines,
    cache_manager: &Arc<StorageCacheManager>,
    shard: &str,
    segment_seq: u32,
) -> LocalReplicaState {
    let leo = leo_for(engines, cache_manager, shard, segment_seq);
    let log_start = match storage_type_of(cache_manager, shard) {
        Some(StorageType::EngineRocksDB) => engines
            .rocksdb
            .log_start_offset(shard, segment_seq)
            .unwrap_or(0),
        Some(StorageType::EngineMemory) => engines
            .memory
            .log_start_offset(shard, segment_seq)
            .unwrap_or(0),
        _ => 0,
    };
    let latest_leader_epoch = cache_manager
        .get_segment_replica(shard, segment_seq)
        .map(|s| s.leader_epoch())
        .unwrap_or(0);
    LocalReplicaState {
        segment_leo: leo.unwrap_or(0),
        latest_leader_epoch,
        log_start_offset: log_start,
        available: leo.is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::{test_build_memory_engine, test_build_rocksdb_engine};
    use bytes::Bytes;
    use metadata_struct::storage::record::{StorageRecord, StorageRecordMetadata};
    use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};
    use rocksdb_engine::test::test_rocksdb_instance;

    fn record(offset: u64) -> StorageRecord {
        StorageRecord {
            metadata: StorageRecordMetadata {
                offset,
                ..Default::default()
            },
            protocol_data: None,
            data: Bytes::from("v"),
        }
    }

    async fn leader_with_epochs(
        epochs: &[(u32, u64)],
        leo: u64,
    ) -> (FetchEngines, Arc<StorageCacheManager>, Arc<RocksDBEngine>) {
        let memory = Arc::new(test_build_memory_engine());
        let cm = memory.cache_manager.clone();
        let db = test_rocksdb_instance();
        cm.set_shard(EngineShard {
            shard_name: "s".to_string(),
            config: EngineShardConfig {
                storage_type: StorageType::EngineMemory,
                ..Default::default()
            },
            ..Default::default()
        });
        let st = cm.get_or_create_segment_replica("s", 0);
        st.set_role(ReplicaRole::LeaderActive);
        st.set_leader_epoch(epochs.last().map(|e| e.0).unwrap_or(0));
        let records: Vec<_> = (0..leo).map(record).collect();
        if !records.is_empty() {
            memory.append_at("s", 0, 0, records).await.unwrap();
        }
        let mut cache = LeaderEpochCache::load(db.clone(), "s", 0).unwrap();
        for (e, start) in epochs {
            cache.assign(*e, *start).unwrap();
        }
        let engines = FetchEngines {
            memory,
            rocksdb: Arc::new(test_build_rocksdb_engine()),
        };
        (engines, cm, db)
    }

    fn req(follower_epoch: u32, current_leader_epoch: u32) -> OffsetsForLeaderEpochReqBody {
        OffsetsForLeaderEpochReqBody {
            shard_name: "s".to_string(),
            segment_seq: 0,
            replica_id: 2,
            replica_broker_epoch: 1,
            current_leader_epoch,
            follower_leader_epoch: follower_epoch,
        }
    }

    #[tokio::test]
    async fn returns_next_epoch_start() {
        let (engines, cm, db) = leader_with_epochs(&[(1, 0), (2, 5)], 8).await;
        let resp = handle_offsets_for_leader_epoch(&engines, &cm, &db, &req(1, 2)).await;
        assert_eq!(resp.error_code, FetchErrorCode::None.as_u32());
        assert_eq!(resp.end_offset_epoch, 1);
        assert_eq!(resp.end_offset, 5);
    }

    #[tokio::test]
    async fn latest_epoch_returns_leo() {
        let (engines, cm, db) = leader_with_epochs(&[(1, 0), (2, 5)], 8).await;
        let resp = handle_offsets_for_leader_epoch(&engines, &cm, &db, &req(2, 2)).await;
        assert_eq!(resp.end_offset_epoch, 2);
        assert_eq!(resp.end_offset, 8);
    }

    #[tokio::test]
    async fn follower_epoch_ahead_returns_leo() {
        let (engines, cm, db) = leader_with_epochs(&[(1, 0), (2, 5)], 8).await;
        let resp = handle_offsets_for_leader_epoch(&engines, &cm, &db, &req(9, 2)).await;
        assert_eq!(resp.end_offset_epoch, -1);
        assert_eq!(resp.end_offset, 8);
    }

    #[tokio::test]
    async fn fences_stale_current_epoch() {
        let (engines, cm, db) = leader_with_epochs(&[(1, 0), (2, 5)], 8).await;
        let resp = handle_offsets_for_leader_epoch(&engines, &cm, &db, &req(1, 1)).await;
        assert_eq!(resp.error_code, FetchErrorCode::FencedLeaderEpoch.as_u32());
    }
}
