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
use crate::core::error::StorageEngineError;
use crate::filesegment::SegmentIdentity;
use crate::isr::leader_epoch::LeaderEpochCache;
use crate::isr::log::ReplicaLog;
use common_config::storage::StorageType;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tracing::warn;

pub async fn recover_local_segments(
    cache_manager: &Arc<StorageCacheManager>,
    memory: &Arc<MemoryStorageEngine>,
    rocksdb: &Arc<RocksDBStorageEngine>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) {
    let segments: Vec<(String, u32)> = cache_manager
        .shards
        .iter()
        .filter(|s| {
            matches!(
                s.config.storage_type,
                StorageType::EngineMemory | StorageType::EngineRocksDB
            )
        })
        .filter_map(|s| {
            let iden = SegmentIdentity::new(&s.shard_name, s.active_segment_seq);
            let seg = cache_manager.get_segment(&iden)?;
            seg.is_replica()
                .then(|| (s.shard_name.clone(), s.active_segment_seq))
        })
        .collect();

    for (shard, segment_seq) in segments {
        if let Err(e) = recover_one_segment(
            cache_manager,
            memory,
            rocksdb,
            rocksdb_engine_handler,
            &shard,
            segment_seq,
        )
        .await
        {
            warn!(
                "recover segment {}/{} on startup: {}",
                shard, segment_seq, e
            );
        }
    }
}

fn recover_leader_epoch_cache(
    cache: &mut LeaderEpochCache,
    local_leo: u64,
    log_start_offset: u64,
) -> Result<(), StorageEngineError> {
    cache.truncate_from_end(local_leo)?;
    cache.truncate_from_start(log_start_offset)?;
    Ok(())
}

fn recover_hw(persisted_hw: u64, local_leo: u64) -> u64 {
    persisted_hw.min(local_leo)
}

async fn recover_one_segment(
    cache_manager: &Arc<StorageCacheManager>,
    memory: &Arc<MemoryStorageEngine>,
    rocksdb: &Arc<RocksDBStorageEngine>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
    segment_seq: u32,
) -> Result<(), StorageEngineError> {
    cache_manager.add_segment_replica(shard, segment_seq);
    let is_rocksdb = cache_manager
        .shards
        .get(shard)
        .map(|s| s.config.storage_type == StorageType::EngineRocksDB)
        .unwrap_or(false);

    let (leo, log_start) = if is_rocksdb {
        (
            rocksdb.latest_offset(shard, segment_seq)?,
            rocksdb.log_start_offset(shard, segment_seq).unwrap_or(0),
        )
    } else {
        (
            memory.latest_offset(shard, segment_seq)?,
            memory.log_start_offset(shard, segment_seq).unwrap_or(0),
        )
    };

    let mut cache = LeaderEpochCache::load(rocksdb_engine_handler.clone(), shard, segment_seq)?;
    recover_leader_epoch_cache(&mut cache, leo, log_start)?;

    let persisted_hw = cache_manager
        .get_offset_state(shard)
        .map(|s| s.high_watermark_offset)
        .unwrap_or(0);
    let hw = recover_hw(persisted_hw, leo);
    cache_manager.update_high_watermark_offset(shard, hw);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::isr::leader_epoch::LeaderEpochCache;
    use rocksdb_engine::test::test_rocksdb_instance;

    fn cache() -> LeaderEpochCache {
        LeaderEpochCache::load(test_rocksdb_instance(), "s", 0).unwrap()
    }

    #[test]
    fn recover_trims_both_ends() {
        let mut c = cache();
        c.assign(1, 0).unwrap();
        c.assign(2, 5).unwrap();
        c.assign(3, 9).unwrap();

        recover_leader_epoch_cache(&mut c, 8, 3).unwrap();

        assert_eq!(c.latest_epoch(), 2);
        assert_eq!(c.end_offset_for(1), Some(5));
        assert_eq!(c.end_offset_for(2), None);
    }

    #[test]
    fn hw_clamped_to_leo() {
        assert_eq!(recover_hw(8, 5), 5);
        assert_eq!(recover_hw(3, 5), 3);
    }

    #[tokio::test]
    async fn recover_local_segments_trims_phantom_epoch() {
        use crate::core::test_tool::{
            test_build_memory_engine, test_build_rocksdb_engine, test_init_conf,
        };
        use bytes::Bytes;
        use metadata_struct::storage::record::{StorageRecord, StorageRecordMetadata};
        use metadata_struct::storage::segment::Replica;
        use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};

        test_init_conf();
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
        cm.set_segment(&metadata_struct::storage::segment::EngineSegment {
            shard_name: "s".to_string(),
            segment_seq: 0,
            leader: 1,
            isr: vec![1],
            replicas: vec![Replica {
                replica_seq: 0,
                node_id: 1,
                fold: String::new(),
            }],
            ..Default::default()
        });

        cm.save_offset_state(
            "s".to_string(),
            crate::core::offset::ShardOffsetState::default(),
        );

        let recs: Vec<_> = (0..3u64)
            .map(|o| StorageRecord {
                metadata: StorageRecordMetadata {
                    offset: o,
                    ..Default::default()
                },
                protocol_data: None,
                data: Bytes::from("v"),
            })
            .collect();
        memory.append_at("s", 0, 0, recs).await.unwrap();
        assert_eq!(memory.latest_offset("s", 0).unwrap(), 3);

        {
            let mut c = LeaderEpochCache::load(db.clone(), "s", 0).unwrap();
            c.assign(1, 0).unwrap();
            c.assign(2, 9).unwrap();
        }

        let rocksdb = Arc::new(test_build_rocksdb_engine());
        recover_local_segments(&cm, &memory, &rocksdb, &db).await;

        let c = LeaderEpochCache::load(db, "s", 0).unwrap();
        assert_eq!(c.latest_epoch(), 1);
    }
}
