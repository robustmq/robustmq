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
use crate::core::error::StorageEngineError;
use crate::core::offset::{ShardOffset, ShardOffsetState};
use crate::filesegment::SegmentIdentity;
use crate::isr::leader_epoch::LeaderEpochCache;
use common_config::storage::StorageType;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tracing::warn;

pub async fn recover_local_segments(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) {
    let segments: Vec<(String, u32)> = cache_manager
        .shards
        .iter()
        .filter(|s| {
            matches!(
                s.config.storage_type,
                StorageType::EngineMemory | StorageType::EngineRocksDB | StorageType::EngineSegment
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
        if let Err(e) =
            recover_one_segment(cache_manager, rocksdb_engine_handler, &shard, segment_seq).await
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

async fn recover_one_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &str,
    segment_seq: u32,
) -> Result<(), StorageEngineError> {
    cache_manager.add_segment_replica(shard, segment_seq);

    let shard_offset = ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
    let state = match shard_offset.get_shard_offsets(shard) {
        Ok(s) => s,
        Err(StorageEngineError::NotOffsetState(_)) => {
            let default = ShardOffsetState::default();
            cache_manager.save_offset_state(shard.to_string(), default.clone());
            default
        }
        Err(e) => return Err(e),
    };

    let mut epoch_cache =
        LeaderEpochCache::load(rocksdb_engine_handler.clone(), shard, segment_seq)?;
    recover_leader_epoch_cache(&mut epoch_cache, state.latest_offset, state.earliest_offset)?;

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

    #[tokio::test]
    async fn recover_local_segments_trims_phantom_epoch() {
        use crate::core::offset::ShardOffset;
        use crate::core::test_tool::test_init_conf;
        use broker_core::cache::NodeCacheManager;
        use common_config::config::BrokerConfig;
        use metadata_struct::storage::segment::Replica;
        use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};

        test_init_conf();
        let db = test_rocksdb_instance();
        let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
        let cm = Arc::new(crate::core::cache::StorageCacheManager::new(broker_cache));

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

        let shard_offset = ShardOffset::new(cm.clone(), db.clone());
        shard_offset.save_earliest_offset("s", 0).unwrap();
        shard_offset.save_latest_offset("s", 3).unwrap();
        shard_offset.save_high_watermark_offset("s", 0).unwrap();

        {
            let mut c = LeaderEpochCache::load(db.clone(), "s", 0).unwrap();
            c.assign(1, 0).unwrap();
            c.assign(2, 9).unwrap();
        }

        recover_local_segments(&cm, &db).await;

        let c = LeaderEpochCache::load(db, "s", 0).unwrap();
        assert_eq!(c.latest_epoch(), 1);
    }
}
