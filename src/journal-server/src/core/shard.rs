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

use std::fs::remove_dir_all;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use common_base::config::journal_server::journal_server_conf;
use grpc_clients::pool::ClientPool;
use log::error;
use metadata_struct::journal::namespace;
use metadata_struct::journal::shard::shard_name_iden;
use protocol::journal_server::journal_inner::{
    DeleteShardFileRequest, GetShardDeleteStatusRequest,
};
use protocol::placement_center::placement_center_journal::{
    CreateShardRequest, DeleteShardRequest,
};
use rocksdb_engine::RocksDBEngine;
use tokio::time::sleep;

use super::cache::CacheManager;
use super::error::JournalServerError;
use super::segment::delete_local_segment;
use crate::segment::file::data_fold_shard;
use crate::segment::manager::SegmentFileManager;
use crate::segment::SegmentIdentity;

pub fn delete_local_shard(
    cache_manager: Arc<CacheManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    segment_file_manager: Arc<SegmentFileManager>,
    req: DeleteShardFileRequest,
) {
    if cache_manager
        .get_shard(&req.namespace, &req.shard_name)
        .is_none()
    {
        return;
    }

    tokio::spawn(async move {
        // delete segment
        for segment in cache_manager.get_segments_list_by_shard(&req.namespace, &req.shard_name) {
            let segment_iden =
                SegmentIdentity::new(&req.namespace, &req.shard_name, segment.segment_seq);
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
        cache_manager.delete_shard(&req.namespace, &req.shard_name);

        // delete file
        let conf = journal_server_conf();
        for data_fold in conf.storage.data_path.iter() {
            let shard_fold_name = data_fold_shard(&req.namespace, &req.shard_name, data_fold);
            if Path::new(data_fold).exists() {
                match remove_dir_all(shard_fold_name) {
                    Ok(()) => {}
                    Err(e) => {
                        error!("{}", e);
                    }
                }
            }
        }
    });
}

pub fn is_delete_by_shard(req: &GetShardDeleteStatusRequest) -> Result<bool, JournalServerError> {
    let conf = journal_server_conf();
    for data_fold in conf.storage.data_path.iter() {
        let shard_fold_name = data_fold_shard(&req.namespace, &req.shard_name, data_fold);
        if Path::new(&shard_fold_name).exists() {
            return Ok(false);
        }
    }

    return Ok(true);
}

pub async fn create_shard_to_place(
    client_pool: Arc<ClientPool>,
    namespace: &str,
    shard_name: &str,
    replica_num: u32,
) -> Result<(), JournalServerError> {
    let conf = journal_server_conf();
    let request = CreateShardRequest {
        cluster_name: conf.cluster_name.to_string(),
        namespace: namespace.to_string(),
        shard_name: shard_name.to_string(),
        replica: replica_num,
    };
    grpc_clients::placement::journal::call::create_shard(
        &client_pool,
        &conf.placement_center,
        request,
    )
    .await?;
    Ok(())
}

pub async fn delete_shard_to_place(
    client_pool: Arc<ClientPool>,
    namespace: &str,
    shard_name: &str,
) -> Result<(), JournalServerError> {
    let conf = journal_server_conf();
    let request = DeleteShardRequest {
        cluster_name: conf.cluster_name.clone(),
        namespace: namespace.to_string(),
        shard_name: shard_name.to_string(),
    };

    grpc_clients::placement::journal::call::delete_shard(
        &client_pool,
        &conf.placement_center,
        request,
    )
    .await?;
    Ok(())
}

pub async fn try_auto_create_shard(
    cache_manager: &Arc<CacheManager>,
    client_pool: &Arc<ClientPool>,
    namespace: &str,
    shard_name: &str,
) -> Result<(), JournalServerError> {
    if cache_manager.get_shard(namespace, shard_name).is_some() {
        return Ok(());
    }

    if !cache_manager.get_cluster().enable_auto_create_shard {
        return Err(JournalServerError::ShardNotExist(shard_name_iden(
            namespace, shard_name,
        )));
    }

    create_shard_to_place(
        client_pool.clone(),
        namespace,
        shard_name,
        cache_manager.get_cluster().default_shard_replica_num,
    )
    .await?;
    let mut i = 0;
    loop {
        if i >= 30 {
            break;
        }
        if cache_manager.get_shard(namespace, shard_name).is_some() {
            return Ok(());
        }
        i += 1;
        sleep(Duration::from_secs(1)).await;
    }

    return Err(JournalServerError::ShardNotExist(shard_name_iden(
        namespace, shard_name,
    )));
}
