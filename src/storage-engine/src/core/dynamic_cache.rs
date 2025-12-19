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

use std::sync::Arc;

use metadata_struct::storage::segment::JournalSegment;
use metadata_struct::storage::segment_meta::JournalSegmentMetadata;
use metadata_struct::storage::shard::EngineShard;
use protocol::broker::broker_common::{
    BrokerUpdateCacheActionType, BrokerUpdateCacheResourceType, UpdateCacheRequest,
};
use tracing::{error, info};

use super::cache::StorageCacheManager;
use crate::{
    core::error::StorageEngineError,
    segment::manager::{create_local_segment, SegmentFileManager},
};

pub async fn update_storage_cache_metadata(
    cache_manager: &Arc<StorageCacheManager>,
    segment_file_manager: &Arc<SegmentFileManager>,
    request: &UpdateCacheRequest,
) -> Result<(), StorageEngineError> {
    match request.resource_type() {
        BrokerUpdateCacheResourceType::Shard => {
            parse_shard(cache_manager, request.action_type(), &request.data);
        }

        BrokerUpdateCacheResourceType::Segment => {
            parse_segment(
                cache_manager,
                segment_file_manager,
                request.action_type(),
                &request.data,
            )
            .await;
        }

        BrokerUpdateCacheResourceType::SegmentMeta => {
            parse_segment_meta(cache_manager, request.action_type(), &request.data).await;
        }

        _ => {}
    }
    Ok(())
}

fn parse_shard(
    cache_manager: &Arc<StorageCacheManager>,
    action_type: BrokerUpdateCacheActionType,
    data: &[u8],
) {
    match action_type {
        BrokerUpdateCacheActionType::Set => match EngineShard::decode(data) {
            Ok(shard) => {
                info!("Update the cache, set shard, shard name: {:?}", shard);
                cache_manager.set_shard(shard);
            }
            Err(e) => {
                error!(
                    "set shard information failed to parse with error message :{}",
                    e
                );
            }
        },

        BrokerUpdateCacheActionType::Delete => {}
    }
}

async fn parse_segment(
    cache_manager: &Arc<StorageCacheManager>,
    segment_file_manager: &Arc<SegmentFileManager>,
    action_type: BrokerUpdateCacheActionType,
    data: &[u8],
) {
    match action_type {
        BrokerUpdateCacheActionType::Set => match JournalSegment::decode(data) {
            Ok(segment) => {
                info!("Segment cache update, action: set, segment:{:?}", segment);

                if let Err(e) =
                    create_local_segment(cache_manager, segment_file_manager, &segment).await
                {
                    error!("Error creating local Segment file, error message: {}", e);
                }
            }
            Err(e) => {
                error!(
                    "Set segment information failed to parse with error message :{}",
                    e
                );
            }
        },
        BrokerUpdateCacheActionType::Delete => {}
    }
}

async fn parse_segment_meta(
    cache_manager: &Arc<StorageCacheManager>,
    action_type: BrokerUpdateCacheActionType,
    data: &[u8],
) {
    match action_type {
        BrokerUpdateCacheActionType::Set => match JournalSegmentMetadata::decode(data) {
            Ok(segment_meta) => {
                info!(
                    "Update the cache, set segment meta, segment meta:{:?}",
                    segment_meta
                );

                cache_manager.set_segment_meta(segment_meta);
            }
            Err(e) => {
                error!(
                    "Set segment meta information failed to parse with error message :{}",
                    e
                );
            }
        },

        BrokerUpdateCacheActionType::Delete => {}
    }
}
