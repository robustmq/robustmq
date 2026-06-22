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

use super::cache::StorageCacheManager;
use crate::{
    core::{
        error::StorageEngineError,
        offset::{ShardOffset, ShardOffsetState},
    },
    filesegment::{file::open_segment_write, SegmentIdentity},
    isr::{apply::apply_leader_and_isr, fetcher_manager::ReplicaFetcherManager},
};
use common_config::storage::StorageType;
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;
use protocol::broker::broker::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRecord,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tracing::warn;

pub async fn update_storage_cache_metadata(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    record: &UpdateCacheRecord,
) -> Result<(), StorageEngineError> {
    match record.resource_type() {
        BrokerUpdateCacheResourceType::Shard => {
            parse_shard(cache_manager, record.action_type(), &record.data).await?;
        }

        BrokerUpdateCacheResourceType::Segment => {
            parse_segment(
                cache_manager,
                rocksdb_engine_handler,
                fetcher_manager,
                record.action_type(),
                &record.data,
            )
            .await?;
        }

        BrokerUpdateCacheResourceType::SegmentMeta => {
            parse_segment_meta(cache_manager, record.action_type(), &record.data).await?;
        }

        _ => {}
    }

    Ok(())
}

async fn parse_shard(
    cache_manager: &Arc<StorageCacheManager>,
    action_type: BrokerUpdateCacheActionType,
    data: &[u8],
) -> Result<(), StorageEngineError> {
    match action_type {
        BrokerUpdateCacheActionType::Create => {
            let shard = EngineShard::decode(data)?;
            cache_manager.set_shard(shard);
        }
        BrokerUpdateCacheActionType::Update => {
            let shard = EngineShard::decode(data)?;
            cache_manager.set_shard(shard);
        }
        BrokerUpdateCacheActionType::Delete => {
            let shard = EngineShard::decode(data)?;
            if cache_manager.shards.contains_key(&shard.shard_name) {
                cache_manager.push_pending_delete_shard(shard.shard_name);
            }
        }
    }
    Ok(())
}

async fn parse_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    action_type: BrokerUpdateCacheActionType,
    data: &[u8],
) -> Result<(), StorageEngineError> {
    match action_type {
        BrokerUpdateCacheActionType::Create => {
            let segment = EngineSegment::decode(data)?;
            create_segment(
                cache_manager,
                rocksdb_engine_handler,
                fetcher_manager,
                segment,
            )
            .await?;
        }

        BrokerUpdateCacheActionType::Update => {
            let segment = EngineSegment::decode(data)?;
            update_segment(
                cache_manager,
                rocksdb_engine_handler,
                fetcher_manager,
                segment,
            )
            .await?;
        }
        BrokerUpdateCacheActionType::Delete => {
            let segment = EngineSegment::decode(data)?;
            let segment_iden = SegmentIdentity::new(&segment.shard_name, segment.segment_seq);
            cache_manager.push_pending_delete_segment(segment_iden);
        }
    }
    Ok(())
}

async fn update_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    segment: EngineSegment,
) -> Result<(), StorageEngineError> {
    if !cache_manager.shards.contains_key(&segment.shard_name) {
        warn!(
            "Skipping segment update for segment {} in shard '{}': shard not found in cache",
            segment.segment_seq, segment.shard_name
        );
        return Ok(());
    }

    let segment_iden = SegmentIdentity::new(&segment.shard_name, segment.segment_seq);
    if is_outdated_segment_notify(cache_manager, &segment_iden, &segment) {
        return Ok(());
    }

    apply_leader_and_isr(
        cache_manager,
        rocksdb_engine_handler,
        fetcher_manager,
        &segment,
    )
    .await?;

    cache_manager.set_segment(&segment);
    Ok(())
}

async fn create_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    segment: EngineSegment,
) -> Result<(), StorageEngineError> {
    // check
    let shard = if let Some(shard) = cache_manager.shards.get(&segment.shard_name) {
        shard.clone()
    } else {
        warn!(
            "Skipping segment creation for segment {} in shard '{}': shard not found in cache",
            segment.segment_seq, segment.shard_name
        );
        return Ok(());
    };

    let segment_iden = SegmentIdentity::new(&segment.shard_name, segment.segment_seq);
    if is_outdated_segment_notify(cache_manager, &segment_iden, &segment) {
        return Ok(());
    }

    // add segment to cache
    cache_manager.set_segment(&segment);

    // init hw/leo/lso
    if segment.segment_seq == 0 {
        println!("segment_seqsegment:{:?}", segment);
        let shard_offset = ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
        shard_offset.save_earliest_offset(&shard.shard_name, 0)?;
        shard_offset.save_latest_offset(&shard.shard_name, 0)?;
        shard_offset.save_high_watermark_offset(&shard.shard_name, 0)?;
        cache_manager.save_offset_state(
            shard.shard_name.clone(),
            ShardOffsetState {
                earliest_offset: 0,
                latest_offset: 0,
                high_watermark_offset: 0,
            },
        );
    }

    // file segment init
    if shard.config.storage_type == StorageType::EngineSegment {
        let segment_file = open_segment_write(cache_manager, &segment_iden).await?;
        segment_file.try_create().await?;
    }

    // isr change
    apply_leader_and_isr(
        cache_manager,
        rocksdb_engine_handler,
        fetcher_manager,
        &segment,
    )
    .await?;

    Ok(())
}

fn is_outdated_segment_notify(
    cache_manager: &Arc<StorageCacheManager>,
    segment_iden: &SegmentIdentity,
    incoming: &EngineSegment,
) -> bool {
    if let Some(local) = cache_manager.get_segment(segment_iden) {
        let stale = local.segment_epoch > incoming.segment_epoch
            || (local.segment_epoch == incoming.segment_epoch
                && local.leader_epoch > incoming.leader_epoch);
        if stale {
            warn!(
                "Dropping stale segment notification for {}: local (segment_epoch {}, leader_epoch {}) > incoming (segment_epoch {}, leader_epoch {})",
                segment_iden.name(),
                local.segment_epoch,
                local.leader_epoch,
                incoming.segment_epoch,
                incoming.leader_epoch
            );
            return true;
        }
    }
    false
}

async fn parse_segment_meta(
    cache_manager: &Arc<StorageCacheManager>,
    action_type: BrokerUpdateCacheActionType,
    data: &[u8],
) -> Result<(), StorageEngineError> {
    match action_type {
        BrokerUpdateCacheActionType::Create | BrokerUpdateCacheActionType::Update => {
            let meta = EngineSegmentMetadata::decode(data)?;
            let shard = if let Some(shard) = cache_manager.shards.get(&meta.shard_name) {
                shard.clone()
            } else {
                warn!(
                    "Skipping segment metadata update for segment {} in shard '{}': shard not found in cache",
                    meta.segment_seq, meta.shard_name
                );
                return Ok(());
            };

            if shard.config.storage_type != StorageType::EngineSegment {
                warn!(
                    "Skipping segment metadata update for segment {} in shard '{}': storage type {:?} is not EngineSegment",
                    meta.segment_seq, meta.shard_name, shard.config.storage_type
                );
                return Ok(());
            }

            let segment_iden = SegmentIdentity::new(&meta.shard_name, meta.segment_seq);

            cache_manager.set_segment_meta(meta);
            cache_manager.sort_offset_index(&segment_iden.shard_name);
        }

        BrokerUpdateCacheActionType::Delete => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::is_outdated_segment_notify;
    use crate::core::cache::StorageCacheManager;
    use crate::core::test_tool::test_init_conf;
    use crate::filesegment::SegmentIdentity;
    use broker_core::cache::NodeCacheManager;
    use common_config::config::BrokerConfig;
    use metadata_struct::storage::segment::EngineSegment;
    use std::sync::Arc;

    fn make_cache() -> Arc<StorageCacheManager> {
        test_init_conf();
        Arc::new(StorageCacheManager::new(Arc::new(NodeCacheManager::new(
            BrokerConfig::default(),
        ))))
    }

    fn segment(epoch: u32) -> EngineSegment {
        EngineSegment {
            shard_name: "s1".to_string(),
            segment_seq: 0,
            segment_epoch: epoch,
            ..Default::default()
        }
    }

    fn segment_le(segment_epoch: u32, leader_epoch: u32) -> EngineSegment {
        EngineSegment {
            shard_name: "s1".to_string(),
            segment_seq: 0,
            segment_epoch,
            leader_epoch,
            ..Default::default()
        }
    }

    #[test]
    fn segment_epoch_monotonic_filter() {
        let cache = make_cache();
        let iden = SegmentIdentity::new("s1", 0);

        assert!(!is_outdated_segment_notify(&cache, &iden, &segment(0)));
        cache.set_segment(&segment(2));

        assert!(is_outdated_segment_notify(&cache, &iden, &segment(1)));
        assert!(!is_outdated_segment_notify(&cache, &iden, &segment(2)));
        assert!(!is_outdated_segment_notify(&cache, &iden, &segment(3)));
    }

    #[test]
    fn leader_epoch_filter_within_same_segment_epoch() {
        let cache = make_cache();
        let iden = SegmentIdentity::new("s1", 0);
        cache.set_segment(&segment_le(5, 10));

        assert!(is_outdated_segment_notify(&cache, &iden, &segment_le(5, 9)));
        assert!(!is_outdated_segment_notify(
            &cache,
            &iden,
            &segment_le(6, 9)
        ));
    }
}
