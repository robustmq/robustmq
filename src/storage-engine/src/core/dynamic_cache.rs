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
    commitlog::offset::CommitLogOffset,
    core::{error::StorageEngineError, segment::delete_local_segment, shard::delete_local_shard},
    filesegment::{
        segment_file::open_segment_write, segment_offset::SegmentOffset, SegmentIdentity,
    },
};
use common_config::{broker::broker_config, storage::StorageType};
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;
use protocol::broker::broker_common::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRecord,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tracing::warn;

pub async fn update_storage_cache_metadata(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    record: &UpdateCacheRecord,
) -> Result<(), StorageEngineError> {
    match record.resource_type() {
        BrokerUpdateCacheResourceType::Shard => {
            parse_shard(
                cache_manager,
                rocksdb_engine_handler,
                record.action_type(),
                &record.data,
            )
            .await?;
        }

        BrokerUpdateCacheResourceType::Segment => {
            parse_segment(
                cache_manager,
                rocksdb_engine_handler,
                record.action_type(),
                &record.data,
            )
            .await?;
        }

        BrokerUpdateCacheResourceType::SegmentMeta => {
            parse_segment_meta(
                cache_manager,
                rocksdb_engine_handler,
                record.action_type(),
                &record.data,
            )
            .await?;
        }

        _ => {}
    }

    Ok(())
}

async fn parse_shard(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    action_type: BrokerUpdateCacheActionType,
    data: &[u8],
) -> Result<(), StorageEngineError> {
    match action_type {
        BrokerUpdateCacheActionType::Create => {
            let shard = EngineShard::decode(data)?;
            if shard.config.storage_type == StorageType::EngineMemory
                || shard.config.storage_type == StorageType::EngineRocksDB
            {
                let commit_offset =
                    CommitLogOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
                commit_offset.save_earliest_offset(&shard.shard_name, 0)?;
                commit_offset.save_latest_offset(&shard.shard_name, 0)?;
            }
            cache_manager.set_shard(shard);
        }
        BrokerUpdateCacheActionType::Update => {}
        BrokerUpdateCacheActionType::Delete => {
            let shard = EngineShard::decode(data)?;
            delete_local_shard(
                cache_manager.clone(),
                rocksdb_engine_handler.clone(),
                shard.shard_name,
            );
        }
    }
    Ok(())
}

async fn parse_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    action_type: BrokerUpdateCacheActionType,
    data: &[u8],
) -> Result<(), StorageEngineError> {
    match action_type {
        BrokerUpdateCacheActionType::Create => {
            let segment = EngineSegment::decode(data)?;
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
            cache_manager.set_segment(&segment);

            let conf = broker_config();
            if conf.broker_id == segment.leader {
                cache_manager.add_leader_segment(&segment_iden);
            }

            if shard.config.storage_type == StorageType::EngineSegment {
                let segment_file = open_segment_write(cache_manager, &segment_iden).await?;
                segment_file.try_create().await?;
            }
            if shard.config.storage_type == StorageType::EngineMemory
                && shard.config.storage_type == StorageType::EngineRocksDB
            {
                let commit_log_offset =
                    CommitLogOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
                commit_log_offset.save_earliest_offset(&shard.shard_name, 0)?;
                commit_log_offset.save_latest_offset(&shard.shard_name, 0)?;
            }
        }

        BrokerUpdateCacheActionType::Update => {
            let segment = EngineSegment::decode(data)?;
            cache_manager.set_segment(&segment);
            let conf = broker_config();
            let segment_iden = SegmentIdentity::new(&segment.shard_name, segment.segment_seq);
            if conf.broker_id == segment.leader {
                cache_manager.add_leader_segment(&segment_iden);
            } else {
                cache_manager.remove_leader_segment(&segment_iden);
            }
        }
        BrokerUpdateCacheActionType::Delete => {
            let segment = EngineSegment::decode(data)?;
            let segment_iden = SegmentIdentity::new(&segment.shard_name, segment.segment_seq);
            delete_local_segment(cache_manager, rocksdb_engine_handler, &segment_iden).await?;
        }
    }
    Ok(())
}

async fn parse_segment_meta(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
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

            let segment_index_manager = SegmentOffset::new(rocksdb_engine_handler.clone());
            segment_index_manager.batch_save_segment_metadata(
                &segment_iden,
                meta.start_offset,
                meta.end_offset,
                meta.start_timestamp,
                meta.end_timestamp,
            )?;
            cache_manager.set_segment_meta(meta);

            cache_manager.sort_offset_index(&segment_iden.shard_name);
        }

        BrokerUpdateCacheActionType::Delete => {}
    }
    Ok(())
}
