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
use super::error::StorageEngineError;
use crate::filesegment::file::{open_segment_write, SegmentFile};
use crate::filesegment::index::build::delete_segment_index;
use crate::filesegment::SegmentIdentity;
use common_config::broker::broker_config;
use metadata_struct::storage::segment::EngineSegment;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tracing::{error, info};

pub fn segment_validator(
    cache_manager: &Arc<StorageCacheManager>,
    shard_name: &str,
    segment: u32,
) -> Result<(), StorageEngineError> {
    let segment_iden = SegmentIdentity::new(shard_name, segment);
    let segment = if let Some(segment) = cache_manager.get_segment(&segment_iden) {
        segment
    } else {
        return Err(StorageEngineError::SegmentNotExist(segment_iden.name()));
    };

    if !segment.allow_read() {
        return Err(StorageEngineError::SegmentStatusError(
            segment_iden.name(),
            segment.status.to_string(),
        ));
    }

    if cache_manager.get_segment_meta(&segment_iden).is_none() {
        return Err(StorageEngineError::SegmentFileMetaNotExists(
            segment_iden.name(),
        ));
    }
    Ok(())
}
pub async fn delete_local_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    if cache_manager.get_segment(segment_iden).is_none() {
        return Ok(());
    };

    // delete segment by cache
    cache_manager.delete_segment(segment_iden);

    // delete index
    if let Err(e) = delete_segment_index(rocksdb_engine_handler, segment_iden) {
        info!("Delete Segment {:?} index, hint:{:?}", segment_iden, e);
    }

    // delete local file
    match open_segment_write(cache_manager, segment_iden).await {
        Ok(segment_file) => {
            if let Err(e) = segment_file.delete().await {
                error!("{}", e);
            }
        }
        Err(e) => {
            info!("Delete Segment {:?}, hint: {:?}", segment_iden, e);
        }
    }

    info!("Segment {} deleted successfully", segment_iden.name());
    Ok(())
}

pub async fn segment_already_delete(
    cache_manager: &Arc<StorageCacheManager>,
    shard_name: &str,
    segment: u32,
) -> Result<bool, StorageEngineError> {
    let segment_iden = SegmentIdentity {
        shard_name: shard_name.to_string(),
        segment,
    };

    let segment_file = open_segment_write(cache_manager, &segment_iden).await?;

    Ok(!segment_file.exists())
}

/// Create a new local segment file from `JournalSegment`.
pub async fn create_local_segment(
    cache_manager: &Arc<StorageCacheManager>,
    segment: &EngineSegment,
) -> Result<(), StorageEngineError> {
    let segment_iden = SegmentIdentity {
        shard_name: segment.shard_name.clone(),
        segment: segment.segment_seq,
    };

    if cache_manager.get_segment(&segment_iden).is_some() {
        return Ok(());
    }

    let conf = broker_config();
    let fold = if let Some(fold) = segment.get_fold(conf.broker_id) {
        fold
    } else {
        return Err(StorageEngineError::SegmentDataDirectoryNotFound(
            segment_iden.name(),
            conf.broker_id,
        ));
    };

    // create segment file
    let segment_file =
        SegmentFile::new(segment.shard_name.clone(), segment.segment_seq, fold).await?;
    segment_file.try_create().await?;

    // add cache
    cache_manager.set_segment(segment);

    info!("Segment {} created successfully", segment_iden.name());
    Ok(())
}
