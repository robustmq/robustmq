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
use super::segment::delete_local_segment;
use crate::segment::file::data_fold_shard;
use crate::segment::SegmentIdentity;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::adapter_offset::AdapterShardInfo;
use protocol::meta::meta_service_journal::{CreateShardRequest, DeleteShardRequest};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::fs::remove_dir_all;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info};

#[derive(Clone, Debug, Default)]
pub struct ShardOffsetState {
    pub earliest_offset: u64,
    pub high_watermark_offset: u64,
    pub latest_offset: u64,
}

pub fn delete_local_shard(
    cache_manager: Arc<StorageCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    shard_name: String,
) {
    if !cache_manager.shards.contains_key(&shard_name) {
        return;
    }

    tokio::spawn(async move {
        // delete segment
        for segment in cache_manager.get_segments_list_by_shard(&shard_name) {
            let segment_iden = SegmentIdentity::new(&shard_name, segment.segment_seq);
            if let Err(e) =
                delete_local_segment(&cache_manager, &rocksdb_engine_handler, &segment_iden).await
            {
                error!("{}", e);
                return;
            }
        }

        // delete file
        let conf = broker_config();
        for data_fold in conf.storage_runtime.data_path.iter() {
            let shard_fold_name = data_fold_shard(&shard_name, data_fold);
            if Path::new(&shard_fold_name).exists() {
                match remove_dir_all(shard_fold_name) {
                    Ok(()) => {}
                    Err(e) => {
                        info!("{}", e);
                    }
                }
            }
        }

        // delete shard
        cache_manager.delete_shard(&shard_name);

        info!("Shard {} deleted successfully", shard_name);
    });
}

pub fn is_delete_by_shard(shard_name: &str) -> Result<bool, StorageEngineError> {
    let conf = broker_config();
    for data_fold in conf.storage_runtime.data_path.iter() {
        let shard_fold_name = data_fold_shard(shard_name, data_fold);
        if Path::new(&shard_fold_name).exists() {
            return Ok(false);
        }
    }

    Ok(true)
}

pub async fn create_shard_to_place(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    shard: &AdapterShardInfo,
) -> Result<(), StorageEngineError> {
    let shard_name = &shard.shard_name;

    let conf: &common_config::config::BrokerConfig = broker_config();
    let request = CreateShardRequest {
        shard_name: shard_name.to_string(),
        shard_config: shard.config.encode()?,
    };
    grpc_clients::meta::storage::call::create_shard(
        client_pool,
        &conf.get_meta_service_addr(),
        request,
    )
    .await?;

    let start = Instant::now();
    loop {
        let segment_iden = SegmentIdentity::new(shard_name, 0);
        if cache_manager.shards.contains_key(shard_name)
            && cache_manager.get_segment(&segment_iden).is_some()
            && cache_manager.get_segment_meta(&segment_iden).is_some()
        {
            info!("Shard {} created successfully", shard_name);
            return Ok(());
        }
        if start.elapsed().as_millis() >= 3000 {
            break;
        }

        sleep(Duration::from_millis(100)).await
    }
    Ok(())
}

pub async fn delete_shard_to_place(
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
) -> Result<(), StorageEngineError> {
    let conf = broker_config();
    let request = DeleteShardRequest {
        shard_name: shard_name.to_string(),
    };

    grpc_clients::meta::storage::call::delete_shard(
        client_pool,
        &conf.get_meta_service_addr(),
        request,
    )
    .await?;
    Ok(())
}
