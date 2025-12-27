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

use crate::{
    clients::manager::ClientConnectionManager,
    core::{cache::StorageCacheManager, error::StorageEngineError},
    memory::engine::MemoryStorageEngine,
    rocksdb::engine::RocksDBStorageEngine,
    segment::{
        write::{WriteChannelDataRecord, WriteManager},
        SegmentIdentity,
    },
};
use common_config::broker::broker_config;
use metadata_struct::{adapter::record::Record, storage::shard::EngineType};
use std::sync::Arc;

pub async fn batch_write(
    write_manager: &Arc<WriteManager>,
    cache_manager: &Arc<StorageCacheManager>,
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    client_connection_manager: &Arc<ClientConnectionManager>,
    shard_name: &str,
    records: &[Record],
) -> Result<Vec<u64>, StorageEngineError> {
    let Some(shard) = cache_manager.shards.get(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
    };
    let Some(active_segment) = cache_manager.get_active_segment(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
    };
    let conf = broker_config();

    let offsets = if conf.broker_id == active_segment.leader {
        write_data_to_remote(client_connection_manager, shard_name, records).await?
    } else {
        match shard.engine_type {
            EngineType::Memory => {
                write_memory_to_local(memory_storage_engine, shard_name, records).await?
            }
            EngineType::RocksDB => {
                write_rocksdb_to_local(rocksdb_storage_engine, shard_name, records).await?
            }
            EngineType::Segment => {
                write_segment_to_local(
                    write_manager,
                    shard_name,
                    active_segment.segment_seq,
                    records,
                )
                .await?
            }
        }
    };
    Ok(offsets)
}

async fn write_data_to_remote(
    _client_connection_manager: &Arc<ClientConnectionManager>,
    _shard_name: &str,
    _records: &[Record],
) -> Result<Vec<u64>, StorageEngineError> {
    Ok(Vec::new())
}

async fn write_memory_to_local(
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    shard_name: &str,
    records: &[Record],
) -> Result<Vec<u64>, StorageEngineError> {
    let offsets = memory_storage_engine
        .batch_write(shard_name, records)
        .await?;
    Ok(offsets)
}

async fn write_rocksdb_to_local(
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    shard_name: &str,
    records: &[Record],
) -> Result<Vec<u64>, StorageEngineError> {
    let offsets = rocksdb_storage_engine
        .batch_write(shard_name, records)
        .await?;
    Ok(offsets)
}

async fn write_segment_to_local(
    write_manager: &Arc<WriteManager>,
    shard_name: &str,
    segment: u32,
    records: &[Record],
) -> Result<Vec<u64>, StorageEngineError> {
    let segment_iden = SegmentIdentity::new(shard_name, segment);
    let data_list = records
        .iter()
        .map(|record| WriteChannelDataRecord {
            pkid: 0,
            key: record.key.clone(),
            tags: record.tags.clone(),
            value: record.data.clone(),
        })
        .collect();
    let resp = write_manager.write(&segment_iden, data_list).await?;
    Ok(resp.offsets.iter().map(|raw| *raw.1).collect())
}
