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
use crate::core::offset::ShardOffset;
use crate::core::segment::delete_local_segment;
use crate::filesegment::file::data_fold_shard;
use crate::filesegment::SegmentIdentity;
use common_base::tools::loop_select_ticket;
use common_config::{broker::broker_config, storage::StorageType};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::{fs::remove_dir_all, path::Path, sync::Arc};
use tokio::sync::broadcast;
use tracing::{error, info};

const DELETE_WORKER_INTERVAL_MS: u64 = 1000;

pub async fn start_delete_worker(
    cache_manager: Arc<StorageCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    stop_sx: &broadcast::Sender<bool>,
) {
    let ac_fn = || {
        let cache_manager = cache_manager.clone();
        let rocksdb_engine_handler = rocksdb_engine_handler.clone();
        async move {
            run_once(&cache_manager, &rocksdb_engine_handler).await;
            Ok(())
        }
    };
    loop_select_ticket(ac_fn, DELETE_WORKER_INTERVAL_MS, stop_sx).await;
}

async fn run_once(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
) {
    let (shards, segments) = cache_manager.take_pending_deletes();

    for seg_iden in segments {
        delete_segment(cache_manager, rocksdb_engine_handler, &seg_iden).await;
    }

    for shard_name in shards {
        delete_shard(cache_manager, rocksdb_engine_handler, &shard_name).await;
    }
}

async fn delete_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    seg_iden: &SegmentIdentity,
) {
    if let Err(e) = delete_local_segment(cache_manager, rocksdb_engine_handler, seg_iden).await {
        error!("Failed to delete segment {}: {}", seg_iden.name(), e);
        return;
    }

    // For EngineSegment shards, advance earliest_offset to the new start segment.
    if let Some(shard) = cache_manager.shards.get(&seg_iden.shard_name) {
        if shard.config.storage_type == StorageType::EngineSegment {
            let next_iden = SegmentIdentity::new(&seg_iden.shard_name, shard.start_segment_seq);
            if let Some(meta) = cache_manager.get_segment_meta(&next_iden) {
                let shard_offset =
                    ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
                let _ = shard_offset
                    .save_earliest_offset(&seg_iden.shard_name, meta.start_offset.max(0) as u64);
            }
        }
    }
}

async fn delete_shard(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
) {
    if !cache_manager.shards.contains_key(shard_name) {
        return;
    }

    for segment in cache_manager.get_segments_list_by_shard(shard_name) {
        let seg_iden = SegmentIdentity::new(shard_name, segment.segment_seq);
        if let Err(e) = delete_local_segment(cache_manager, rocksdb_engine_handler, &seg_iden).await
        {
            error!(
                "Failed to delete segment {} during shard delete: {}",
                seg_iden.name(),
                e
            );
            return;
        }
    }

    let conf = broker_config();
    for data_fold in conf.storage_runtime.data_path.iter() {
        let shard_fold = data_fold_shard(shard_name, data_fold);
        if Path::new(&shard_fold).exists() {
            if let Err(e) = remove_dir_all(&shard_fold) {
                info!("Remove shard dir {}: {}", shard_fold, e);
            }
        }
    }

    cache_manager.delete_shard(shard_name);
    info!("Shard {} deleted successfully", shard_name);
}
