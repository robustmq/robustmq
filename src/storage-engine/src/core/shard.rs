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
use crate::segment::index::read::{get_in_segment_by_timestamp, get_index_data_by_timestamp};
use crate::segment::offset::get_shard_cursor_offset;
use crate::segment::SegmentIdentity;
use common_config::broker::broker_config;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::adapter_offset::{
    AdapterConsumerGroupOffset, AdapterOffsetStrategy, AdapterShardInfo,
};
use metadata_struct::storage::shard::EngineShardConfig;
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
    let config = EngineShardConfig {
        replica_num: shard.replica_num,
        max_segment_size: 1073741824,
    };

    let conf: &common_config::config::BrokerConfig = broker_config();
    let request = CreateShardRequest {
        shard_name: shard_name.to_string(),
        shard_config: config.encode()?,
    };
    grpc_clients::meta::storage::call::create_shard(
        client_pool,
        &conf.get_meta_service_addr(),
        request,
    )
    .await?;

    let start = Instant::now();
    loop {
        if cache_manager.shards.contains_key(shard_name) {
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

pub fn get_shard_earliest_offset(
    cache_manager: &Arc<StorageCacheManager>,
    shard_name: &str,
) -> Result<AdapterConsumerGroupOffset, StorageEngineError> {
    let shard = cache_manager
        .shards
        .get(shard_name)
        .ok_or_else(|| StorageEngineError::ShardNotExist(shard_name.to_string()))?;

    let segment_iden = SegmentIdentity::new(shard_name, shard.start_segment_seq);
    let segment_meta = cache_manager
        .get_segment_meta(&segment_iden)
        .ok_or_else(|| StorageEngineError::SegmentNotExist(segment_iden.name()))?;

    if segment_meta.start_offset < 0 {
        return Err(StorageEngineError::CommonErrorStr(format!(
            "Invalid start offset {} for shard {} segment {}",
            segment_meta.start_offset, shard_name, shard.start_segment_seq
        )));
    }

    Ok(AdapterConsumerGroupOffset {
        shard_name: shard_name.to_string(),
        segment_no: shard.start_segment_seq,
        offset: segment_meta.start_offset as u64,
        ..Default::default()
    })
}

pub fn get_shard_latest_offset(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
) -> Result<AdapterConsumerGroupOffset, StorageEngineError> {
    let shard = cache_manager
        .shards
        .get(shard_name)
        .ok_or_else(|| StorageEngineError::ShardNotExist(shard_name.to_string()))?;

    let offset =
        get_shard_cursor_offset(rocksdb_engine_handler, shard_name, shard.active_segment_seq)?;

    Ok(AdapterConsumerGroupOffset {
        shard_name: shard_name.to_string(),
        segment_no: shard.active_segment_seq,
        offset,
        ..Default::default()
    })
}

pub fn get_shard_offset_by_timestamp(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    timestamp: u64,
    strategy: AdapterOffsetStrategy,
) -> Result<AdapterConsumerGroupOffset, StorageEngineError> {
    if let Some(segment) = get_in_segment_by_timestamp(cache_manager, shard_name, timestamp as i64)?
    {
        if let Some(index_data) =
            get_index_data_by_timestamp(rocksdb_engine_handler, shard_name, timestamp)?
        {
            Ok(AdapterConsumerGroupOffset {
                shard_name: shard_name.to_string(),
                segment_no: segment,
                offset: index_data.offset,
                ..Default::default()
            })
        } else {
            Err(StorageEngineError::CommonErrorStr(format!(
                "No index data found for timestamp {} in segment {}",
                timestamp, segment
            )))
        }
    } else {
        match strategy {
            AdapterOffsetStrategy::Earliest => get_shard_earliest_offset(cache_manager, shard_name),
            AdapterOffsetStrategy::Latest => {
                get_shard_latest_offset(cache_manager, rocksdb_engine_handler, shard_name)
            }
        }
    }
}
