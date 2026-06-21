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
use crate::filesegment::index::build::delete_segment_index;
use common_config::broker::broker_config;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::{fs::remove_dir_all, path::Path, sync::Arc};
use tracing::{error, info};

pub async fn delete_by_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    seg_iden: &SegmentIdentity,
) -> Result<(), StorageEngineError> {
    if let Err(e) = delete_segment_index(rocksdb_engine_handler, seg_iden) {
        info!("delete index for {}: {}", seg_iden.name(), e);
    }

    match open_segment_write(cache_manager, seg_iden).await {
        Ok(segment_file) => segment_file.delete().await?,
        Err(e) => info!("delete segment file {}, hint: {}", seg_iden.name(), e),
    }

    if let Some(shard) = cache_manager.shards.get(&seg_iden.shard_name) {
        let next_iden = SegmentIdentity::new(&seg_iden.shard_name, shard.start_segment_seq);
        if let Some(meta) = cache_manager.get_segment_meta(&next_iden) {
            ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone())
                .save_earliest_offset(&seg_iden.shard_name, meta.start_offset.max(0) as u64)?;
        }
    }

    Ok(())
}

pub async fn delete_by_shard(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
) {
    for segment in cache_manager.get_segments_list_by_shard(shard_name) {
        let seg_iden = SegmentIdentity::new(shard_name, segment.segment_seq);
        if let Err(e) = delete_segment_index(rocksdb_engine_handler, &seg_iden) {
            info!("delete index for {}: {}", seg_iden.name(), e);
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
