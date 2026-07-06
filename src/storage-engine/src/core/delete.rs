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
use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
use crate::filesegment::file::{data_file_segment, data_fold_shard};
use crate::filesegment::SegmentIdentity;
use crate::isr::fetcher_manager::ReplicaFetcherManager;
use common_base::tools::loop_select_ticket;
use common_config::broker::broker_config;
use common_config::storage::StorageType;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{error, info};

const DELETE_WORKER_INTERVAL_MS: u64 = 5000;

pub async fn start_delete_worker(
    cache_manager: Arc<StorageCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    memory_engine: Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
    fetcher_manager: Arc<ReplicaFetcherManager>,
    stop_sx: &broadcast::Sender<bool>,
) {
    let ac_fn = || {
        let cache_manager = cache_manager.clone();
        let rocksdb_engine_handler = rocksdb_engine_handler.clone();
        let memory_engine = memory_engine.clone();
        let rocksdb_storage_engine = rocksdb_storage_engine.clone();
        let fetcher_manager = fetcher_manager.clone();
        async move {
            run_once(
                &cache_manager,
                &rocksdb_engine_handler,
                &memory_engine,
                &rocksdb_storage_engine,
                &fetcher_manager,
            )
            .await;
            Ok(())
        }
    };
    loop_select_ticket(ac_fn, DELETE_WORKER_INTERVAL_MS, stop_sx).await;
}

async fn run_once(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    memory_engine: &Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
) {
    let (shards, segments) = cache_manager.take_pending_deletes();

    for seg_iden in segments {
        delete_segment(
            cache_manager,
            rocksdb_engine_handler,
            memory_engine,
            rocksdb_storage_engine,
            fetcher_manager,
            &seg_iden,
        )
        .await;
    }

    for shard_name in shards {
        delete_shard(
            cache_manager,
            rocksdb_engine_handler,
            memory_engine,
            rocksdb_storage_engine,
            fetcher_manager,
            &shard_name,
        )
        .await;
    }
}

async fn delete_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    memory_engine: &Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    seg_iden: &SegmentIdentity,
) {
    // clean isr fetch
    fetcher_manager.remove_segment(&seg_iden.shard_name, seg_iden.segment);

    // clean data
    let storage_type = cache_manager
        .shards
        .get(&seg_iden.shard_name)
        .map(|s| s.config.storage_type)
        .unwrap_or_default();

    match storage_type {
        StorageType::EngineMemory => {
            memory_engine.delete_by_segment(&seg_iden.shard_name, seg_iden.segment);
        }
        StorageType::EngineRocksDB => {
            if let Err(e) =
                rocksdb_storage_engine.delete_by_segment(&seg_iden.shard_name, seg_iden.segment)
            {
                error!("delete rocksdb segment {}: {}", seg_iden.name(), e);
                return;
            }
        }
        StorageType::EngineSegment => {
            if let Err(e) = crate::filesegment::delete::delete_by_segment(
                cache_manager,
                rocksdb_engine_handler,
                seg_iden,
            )
            .await
            {
                error!("delete file segment {}: {}", seg_iden.name(), e);
                return;
            }
        }
        _ => {}
    }

    // clean cache
    cache_manager.delete_segment(seg_iden);
    info!("segment {} deleted", seg_iden.name());
}

async fn delete_shard(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    memory_engine: &Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    fetcher_manager: &Arc<ReplicaFetcherManager>,
    shard_name: &str,
) {
    let Some(shard) = cache_manager.shards.get(shard_name).map(|s| s.clone()) else {
        return;
    };

    // clean isr fetch
    fetcher_manager.remove_shard(shard_name);

    // clean data
    match shard.config.storage_type {
        StorageType::EngineMemory => {
            memory_engine.delete_by_shard(shard_name);
        }
        StorageType::EngineRocksDB => {
            if let Err(e) = rocksdb_storage_engine.delete_by_shard(shard_name) {
                error!("delete rocksdb shard {}: {}", shard_name, e);
                return;
            }
        }
        StorageType::EngineSegment => {
            crate::filesegment::delete::delete_by_shard(
                cache_manager,
                rocksdb_engine_handler,
                shard_name,
            )
            .await;
        }
        _ => {}
    }

    // clean cache
    cache_manager.delete_shard(shard_name);
    info!("shard {} deleted", shard_name);
}

pub fn shard_already_delete(cache_manager: &Arc<StorageCacheManager>, shard_name: &str) -> bool {
    if cache_manager.shards.contains_key(shard_name) {
        return false;
    }
    let conf = broker_config();
    for data_fold in &conf.storage_runtime.data_path {
        if Path::new(&data_fold_shard(shard_name, data_fold)).exists() {
            return false;
        }
    }
    true
}

pub fn segment_already_delete(
    cache_manager: &Arc<StorageCacheManager>,
    shard_name: &str,
    segment_seq: u32,
) -> bool {
    let segment_iden = SegmentIdentity::new(shard_name, segment_seq);
    if cache_manager.get_segment(&segment_iden).is_some() {
        return false;
    }
    let conf = broker_config();
    for data_fold in &conf.storage_runtime.data_path {
        let shard_fold = data_fold_shard(shard_name, data_fold);
        if Path::new(&data_file_segment(&shard_fold, segment_seq)).exists() {
            return false;
        }
    }
    true
}
