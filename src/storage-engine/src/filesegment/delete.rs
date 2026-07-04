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

use super::{
    file::{data_fold_shard, open_segment_write},
    SegmentIdentity,
};
use crate::core::{cache::StorageCacheManager, error::StorageEngineError, offset::ShardOffset};
use crate::filesegment::index::build::delete_shard_index_for_segment;
use common_config::broker::broker_config;
use rocksdb_engine::keys::engine::{segment_prefix, shard_prefix};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_STORAGE_ENGINE;
use std::{fs::remove_dir_all, path::Path, sync::Arc};
use tracing::{error, info};

pub async fn delete_by_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    seg_iden: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    // Segment-level keys (position / timestamp / leader-epoch) under segment_prefix.
    if let Some(cf) = rocksdb_engine_handler.cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE) {
        if let Err(e) = rocksdb_engine_handler
            .delete_prefix(cf, &segment_prefix(&seg_iden.shard_name, seg_iden.segment))
        {
            info!("delete segment index for {}: {}", seg_iden.name(), e);
        }
    }

    // Shard-level key/tag index entries that point into this segment.
    if let Err(e) = delete_shard_index_for_segment(
        rocksdb_engine_handler,
        &seg_iden.shard_name,
        seg_iden.segment,
    ) {
        info!("delete shard index for {}: {}", seg_iden.name(), e);
    }

    match open_segment_write(cache_manager, seg_iden).await {
        Ok(segment_file) => segment_file.delete().await?,
        Err(e) => info!("delete segment file {}, hint: {}", seg_iden.name(), e),
    }

    // Advance earliest_offset to the start of the next segment.
    // start_segment_seq is never updated in the local cache after deletions,
    // so we derive the next seq directly from the segment we just deleted.
    let next_iden = SegmentIdentity::new(&seg_iden.shard_name, seg_iden.segment + 1);
    if let Some(meta) = cache_manager.get_segment_meta(&next_iden) {
        ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone())
            .save_earliest_offset(&seg_iden.shard_name, meta.start_offset.max(0) as u64)?;
    }

    Ok(())
}

/// Delete all fully-consumed segments at the head of a shard whose data is
/// entirely below `target_offset`, then advance the LSO (earliest_offset) as
/// far as `target_offset` allows. The active (still-writable) segment and any
/// sealed segment that still holds data `>= target_offset` are kept.
pub async fn delete_segments_before_offset(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    target_offset: u64,
) -> Result<u64, StorageEngineError> {
    let shard_offset = ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
    let earliest = shard_offset.get_earliest_offset(shard_name)?;
    let latest = shard_offset.get_latest_offset(shard_name)?;
    let target = target_offset.min(latest);
    if target <= earliest {
        return Ok(earliest);
    }

    let mut segments = cache_manager.get_segments_list_by_shard(shard_name);
    segments.sort_by_key(|s| s.segment_seq);
    let active_seq = cache_manager
        .get_active_segment(shard_name)
        .map(|s| s.segment_seq);

    for seg in segments {
        // Never delete the currently-writable segment.
        if Some(seg.segment_seq) == active_seq {
            break;
        }
        let seg_iden = SegmentIdentity::new(shard_name, seg.segment_seq);
        let Some(meta) = cache_manager.get_segment_meta(&seg_iden) else {
            continue;
        };
        // end_offset <= 0 means the segment is still open; not eligible.
        if meta.end_offset <= 0 || meta.end_offset as u64 >= target {
            break;
        }
        delete_by_segment(cache_manager, rocksdb_engine_handler, &seg_iden).await?;
        cache_manager.delete_segment(&seg_iden);
    }

    // delete_by_segment only advances the LSO to the start of the next
    // segment, so if `target` falls inside the first remaining segment, push
    // the LSO the rest of the way explicitly.
    if shard_offset.get_earliest_offset(shard_name)? < target {
        shard_offset.save_earliest_offset(shard_name, target)?;
    }

    Ok(target)
}

pub async fn delete_by_shard(
    _cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
) {
    // Every rocksdb key for the shard nests under shard_prefix: one prefix delete
    // wipes meta, all shard-level indices and every segment's keys.
    if let Some(cf) = rocksdb_engine_handler.cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE) {
        if let Err(e) = rocksdb_engine_handler.delete_prefix(cf, &shard_prefix(shard_name)) {
            error!("delete shard index {}: {}", shard_name, e);
        }
    }

    let conf = broker_config();
    for data_fold in conf.storage_runtime.data_path.iter() {
        let shard_fold = data_fold_shard(shard_name, data_fold);
        if Path::new(&shard_fold).exists() {
            if let Err(e) = remove_dir_all(&shard_fold) {
                error!("remove shard dir {}: {}", shard_fold, e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::test_tool::test_init_segment;
    use common_config::storage::StorageType;
    use metadata_struct::storage::segment::{Replica, SegmentStatus};
    use metadata_struct::storage::segment_meta::EngineSegmentMetadata;

    // Two segments: [0..10) sealed, [10..15) still active/open. Deleting before
    // an offset inside the sealed segment must remove it entirely and advance
    // the LSO to the next segment's start; the active segment must be untouched.
    #[tokio::test]
    async fn delete_segments_before_offset_deletes_fully_consumed_head_segment() {
        let (seg0, cache, _fold, db) = test_init_segment(StorageType::EngineSegment).await;
        let shard_name = seg0.shard_name.clone();

        // seal segment 0: 10 records, offsets [0, 10)
        cache.set_segment_meta(EngineSegmentMetadata {
            shard_name: shard_name.clone(),
            segment_seq: 0,
            start_offset: 0,
            end_offset: 9,
            ..Default::default()
        });

        // segment 1: active/open, offsets starting at 10
        let seg1 = metadata_struct::storage::segment::EngineSegment {
            shard_name: shard_name.clone(),
            segment_seq: 1,
            replicas: vec![Replica {
                replica_seq: 0,
                node_id: 1,
                fold: _fold.clone(),
            }],
            leader: 1,
            leader_epoch: 0,
            status: SegmentStatus::Write,
            isr: vec![1],
            ..Default::default()
        };
        cache.set_segment(&seg1);
        cache.set_segment_meta(EngineSegmentMetadata {
            shard_name: shard_name.clone(),
            segment_seq: 1,
            start_offset: 10,
            end_offset: -1,
            ..Default::default()
        });
        cache.sort_offset_index(&shard_name);

        // make segment 1 the active segment
        let mut shard = cache.shards.get(&shard_name).unwrap().clone();
        shard.active_segment_seq = 1;
        cache.set_shard(shard);

        let shard_offset = ShardOffset::new(cache.clone(), db.clone());
        shard_offset.save_latest_offset(&shard_name, 15).unwrap();

        // target falls inside segment 0 (not fully consumed): kept, LSO advances only to target
        let achieved = delete_segments_before_offset(&cache, &db, &shard_name, 5)
            .await
            .unwrap();
        assert_eq!(achieved, 5);
        assert!(
            cache.get_segment(&seg0).is_some(),
            "segment 0 must survive a partial delete"
        );
        assert_eq!(shard_offset.get_earliest_offset(&shard_name).unwrap(), 5);

        // target now covers all of segment 0: it must be deleted, LSO -> next segment's start
        let achieved = delete_segments_before_offset(&cache, &db, &shard_name, 10)
            .await
            .unwrap();
        assert_eq!(achieved, 10);
        assert!(
            cache.get_segment(&seg0).is_none(),
            "fully-consumed segment must be deleted"
        );
        assert_eq!(shard_offset.get_earliest_offset(&shard_name).unwrap(), 10);

        // active segment is never touched, even if target lands inside it
        let achieved = delete_segments_before_offset(&cache, &db, &shard_name, 12)
            .await
            .unwrap();
        assert_eq!(achieved, 12);
        let seg1_iden = SegmentIdentity::new(&shard_name, 1);
        assert!(
            cache.get_segment(&seg1_iden).is_some(),
            "active segment must never be deleted"
        );
        assert_eq!(shard_offset.get_earliest_offset(&shard_name).unwrap(), 12);
    }
}
