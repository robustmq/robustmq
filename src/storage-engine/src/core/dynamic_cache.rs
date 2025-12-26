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
    core::{error::StorageEngineError, segment::delete_local_segment, shard::delete_local_shard},
    segment::{file::open_segment_write, index::segment::SegmentIndexManager, SegmentIdentity},
};
use common_config::broker::broker_config;
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;
use protocol::broker::broker_common::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRequest,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn update_storage_cache_metadata(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    request: &UpdateCacheRequest,
) -> Result<(), StorageEngineError> {
    match request.resource_type() {
        BrokerUpdateCacheResourceType::Shard => {
            parse_shard(
                cache_manager,
                rocksdb_engine_handler,
                request.action_type(),
                &request.data,
            )
            .await?;
        }

        BrokerUpdateCacheResourceType::Segment => {
            parse_segment(
                cache_manager,
                rocksdb_engine_handler,
                request.action_type(),
                &request.data,
            )
            .await?;
        }

        BrokerUpdateCacheResourceType::SegmentMeta => {
            parse_segment_meta(
                cache_manager,
                rocksdb_engine_handler,
                request.action_type(),
                &request.data,
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
        BrokerUpdateCacheActionType::Set => {
            let shard = EngineShard::decode(data)?;
            cache_manager.set_shard(shard);
        }

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
        BrokerUpdateCacheActionType::Set => {
            let segment = EngineSegment::decode(data)?;
            cache_manager.set_segment(&segment);
            let segment_iden = SegmentIdentity::new(&segment.shard_name, segment.segment_seq);
            let segment_file = open_segment_write(cache_manager, &segment_iden).await?;
            segment_file.try_create().await?;

            let conf = broker_config();
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
        BrokerUpdateCacheActionType::Set => {
            let meta = EngineSegmentMetadata::decode(data)?;
            cache_manager.set_segment_meta(meta.clone());
            let segment_iden = SegmentIdentity::new(&meta.shard_name, meta.segment_seq);

            // persist segment meta index
            let segment_index_manager = SegmentIndexManager::new(rocksdb_engine_handler.clone());
            segment_index_manager.save_start_offset(&segment_iden, meta.start_offset)?;
            segment_index_manager.save_end_offset(&segment_iden, meta.end_offset)?;
            segment_index_manager.save_start_timestamp(&segment_iden, meta.start_timestamp)?;
            segment_index_manager.save_end_timestamp(&segment_iden, meta.end_timestamp)?;
        }

        BrokerUpdateCacheActionType::Delete => {
            // There is no need to implement it. The metadata will be deleted along with the segment, and it will not be deleted separately.
        }
    }
    Ok(())
}
