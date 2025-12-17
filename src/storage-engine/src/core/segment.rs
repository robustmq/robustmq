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
use super::error::JournalServerError;
use crate::index::build::delete_segment_index;
use crate::segment::file::open_segment_write;
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;
use protocol::broker::broker_storage::GetSegmentDeleteStatusRequest;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;
use tracing::{error, info};

pub async fn delete_local_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_file_manager: &Arc<SegmentFileManager>,
    segment_iden: &SegmentIdentity,
) -> Result<(), JournalServerError> {
    if cache_manager.get_segment(segment_iden).is_none() {
        return Ok(());
    };

    // delete segment by cache
    cache_manager.delete_segment(segment_iden);

    // delete segment file manager  by cache
    segment_file_manager.remove_segment_file(segment_iden);

    // delete build index thread
    cache_manager.remove_build_index_thread(segment_iden);

    // delete index
    if let Err(e) = delete_segment_index(rocksdb_engine_handler, segment_iden) {
        info!("Delete Segment {:?} index, hint:{:?}", segment_iden, e);
    }

    // delete local file
    match open_segment_write(cache_manager, segment_iden).await {
        Ok((segment_file, _)) => {
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
    req: &GetSegmentDeleteStatusRequest,
) -> Result<bool, JournalServerError> {
    let segment_iden = SegmentIdentity {
        shard_name: req.shard_name.clone(),
        segment_seq: req.segment,
    };

    let (segment_file, _) = open_segment_write(cache_manager, &segment_iden).await?;

    Ok(!segment_file.exists())
}
