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
use super::segment::delete_local_segment;
use crate::segment::file::data_fold_shard;
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::journal::shard::JournalShardConfig;
use protocol::broker::broker_storage::{DeleteShardFileRequest, GetShardDeleteStatusRequest};
use protocol::meta::meta_service_journal::{CreateShardRequest, DeleteShardRequest};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::fs::remove_dir_all;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::{error, info};

pub fn delete_local_shard(
    cache_manager: Arc<StorageCacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    segment_file_manager: Arc<SegmentFileManager>,
    req: DeleteShardFileRequest,
) {
    if cache_manager.get_shard(&req.shard_name).is_none() {
        return;
    }

    tokio::spawn(async move {
        // delete segment
        for segment in cache_manager.get_segments_list_by_shard(&req.shard_name) {
            let segment_iden = SegmentIdentity::new(&req.shard_name, segment.segment_seq);
            if let Err(e) = delete_local_segment(
                &cache_manager,
                &rocksdb_engine_handler,
                &segment_file_manager,
                &segment_iden,
            )
            .await
            {
                error!("{}", e);
                return;
            }
        }

        // delete shard
        cache_manager.delete_shard(&req.shard_name);

        // delete file
        let conf = broker_config();
        for data_fold in conf.journal_storage.data_path.iter() {
            let shard_fold_name = data_fold_shard(&req.shard_name, data_fold);
            if Path::new(&shard_fold_name).exists() {
                match remove_dir_all(shard_fold_name) {
                    Ok(()) => {}
                    Err(e) => {
                        info!("{}", e);
                    }
                }
            }
        }
        info!("Shard {} deleted successfully", req.shard_name);
    });
}

pub fn is_delete_by_shard(req: &GetShardDeleteStatusRequest) -> Result<bool, JournalServerError> {
    let conf = broker_config();
    for data_fold in conf.journal_storage.data_path.iter() {
        let shard_fold_name = data_fold_shard(&req.shard_name, data_fold);
        if Path::new(&shard_fold_name).exists() {
            return Ok(false);
        }
    }

    Ok(true)
}

/// invoke `create_shard` in meta service
///
/// After meta service receives the request and creates the shard, it will invoke a `update_cache` call back to the journal server. Journal server will update its cache
///
/// Will wait for 3s for the cache update to take effect
pub async fn create_shard_to_place(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
) -> Result<(), JournalServerError> {
    let config = JournalShardConfig {
        replica_num: 1,
        max_segment_size: 1073741824,
    };
    let conf = broker_config();
    let request = CreateShardRequest {
        shard_name: shard_name.to_string(),
        shard_config: config.encode()?,
    };
    grpc_clients::meta::journal::call::create_shard(
        client_pool,
        &conf.get_meta_service_addr(),
        request,
    )
    .await?;

    let start = Instant::now();
    loop {
        if cache_manager.get_shard(shard_name).is_some() {
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

/// invoke `delete_shard` in meta service
///
/// After meta service receives the request and deletes the shard, it will invoke a `delete_shard_file` and `delete_segment` call back to the journal server. Journal server will mark the shard as being deleted
///
/// A background thread will delete the shard and its segments. No need to wait for the deletion to complete
pub async fn delete_shard_to_place(
    client_pool: Arc<ClientPool>,
    shard_name: &str,
) -> Result<(), JournalServerError> {
    let conf = broker_config();
    let request = DeleteShardRequest {
        shard_name: shard_name.to_string(),
    };

    grpc_clients::meta::journal::call::delete_shard(
        &client_pool,
        &conf.get_meta_service_addr(),
        request,
    )
    .await?;
    Ok(())
}

pub async fn try_auto_create_shard(
    cache_manager: &Arc<StorageCacheManager>,
    client_pool: &Arc<ClientPool>,
    shard_name: &str,
) -> Result<(), JournalServerError> {
    if cache_manager.get_shard(shard_name).is_some() {
        return Ok(());
    }

    create_shard_to_place(cache_manager, client_pool, shard_name).await?;
    let mut i = 0;
    loop {
        if i >= 30 {
            break;
        }
        if cache_manager.get_shard(shard_name).is_some() {
            return Ok(());
        }
        i += 1;
        sleep(Duration::from_secs(1)).await;
    }

    Err(JournalServerError::ShardNotExist(shard_name.to_string()))
}
