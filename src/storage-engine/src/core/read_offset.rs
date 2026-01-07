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
    clients::{
        manager::ClientConnectionManager,
        packet::{build_read_req, read_resp_parse},
    },
    core::{
        cache::StorageCacheManager,
        error::StorageEngineError,
        segment::segment_validator,
        shard_offset::{get_earliest_offset, get_latest_offset},
    },
    memory::engine::MemoryStorageEngine,
    rocksdb::engine::RocksDBStorageEngine,
    segment::{
        file::open_segment_write, index::read::get_in_segment_by_offset,
        read::segment_read_by_offset, SegmentIdentity,
    },
};
use common_config::{broker::broker_config, storage::StorageType};
use metadata_struct::storage::{
    adapter_read_config::AdapterReadConfig, shard::EngineShard, storage_record::StorageRecord,
};
use protocol::storage::{
    codec::StorageEnginePacket,
    protocol::{ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType},
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub struct ReadByOffsetParams {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub cache_manager: Arc<StorageCacheManager>,
    pub memory_storage_engine: Arc<MemoryStorageEngine>,
    pub rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
    pub client_connection_manager: Arc<ClientConnectionManager>,
    pub shard_name: String,
    pub offset: u64,
    pub read_config: AdapterReadConfig,
}

pub async fn read_by_offset(
    params: ReadByOffsetParams,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let rocksdb_engine_handler = &params.rocksdb_engine_handler;
    let cache_manager = &params.cache_manager;
    let memory_storage_engine = &params.memory_storage_engine;
    let rocksdb_storage_engine = &params.rocksdb_storage_engine;
    let client_connection_manager = &params.client_connection_manager;
    let shard_name = params.shard_name.as_str();
    let offset = params.offset;
    let read_config = &params.read_config;
    let Some(shard) = cache_manager.shards.get(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
    };

    let segment_no = get_segment_no_by_offset(
        cache_manager,
        rocksdb_engine_handler,
        &shard,
        shard_name,
        offset,
    )?;

    let segment_iden = SegmentIdentity::new(shard_name, segment_no);
    let Some(segment) = cache_manager.get_segment(&segment_iden) else {
        return Err(StorageEngineError::SegmentNotExist(segment_iden.name()));
    };

    segment_validator(cache_manager, shard_name, segment.segment_seq)?;

    let conf = broker_config();
    let results = if conf.broker_id == segment.leader {
        match shard.config.storage_type {
            StorageType::EngineMemory => {
                read_by_memory(memory_storage_engine, shard_name, offset, read_config).await?
            }
            StorageType::EngineRocksDB => {
                read_by_rocksdb(rocksdb_storage_engine, shard_name, offset, read_config).await?
            }
            StorageType::EngineSegment => {
                read_by_segment(
                    cache_manager,
                    rocksdb_engine_handler,
                    shard_name,
                    offset,
                    segment.segment_seq,
                    read_config,
                )
                .await?
            }
            _ => {
                return Err(StorageEngineError::CommonErrorStr(format!(
                    "Unsupported storage type {:?} for shard {}",
                    shard.config.storage_type, shard_name
                )))
            }
        }
    } else {
        read_by_remote(
            client_connection_manager,
            conf.broker_id,
            shard_name,
            offset,
            read_config,
        )
        .await?
    };
    Ok(results)
}

pub async fn read_by_remote(
    client_connection_manager: &Arc<ClientConnectionManager>,
    target_broker_id: u64,
    shard_name: &str,
    offset: u64,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let messages = vec![ReadReqMessage {
        shard_name: shard_name.to_string(),
        read_type: ReadType::Offset,
        filter: ReadReqFilter {
            offset: Some(offset),
            ..Default::default()
        },
        options: ReadReqOptions {
            max_size: read_config.max_size,
            max_record: read_config.max_record_num,
        },
    }];
    let read_req = build_read_req(messages);
    let resp = client_connection_manager
        .write_send(target_broker_id, StorageEnginePacket::ReadReq(read_req))
        .await?;

    match resp {
        StorageEnginePacket::ReadResp(resp) => Ok(read_resp_parse(&resp)?),
        packet => Err(StorageEngineError::ReceivedPacketError(
            target_broker_id,
            format!("Expected ReadResp, got {:?}", packet),
        )),
    }
}

async fn read_by_memory(
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    shard_name: &str,
    offset: u64,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    memory_storage_engine
        .read_by_offset(shard_name, offset, read_config)
        .await
}

async fn read_by_rocksdb(
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    shard_name: &str,
    offset: u64,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    rocksdb_storage_engine
        .read_by_offset(shard_name, offset, read_config)
        .await
}

async fn read_by_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    offset: u64,
    segment: u32,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let segment_iden = SegmentIdentity::new(shard_name, segment);
    let segment_file = open_segment_write(cache_manager, &segment_iden).await?;
    let data_list = segment_read_by_offset(
        rocksdb_engine_handler,
        &segment_file,
        &segment_iden,
        offset,
        read_config.max_size,
        read_config.max_record_num,
    )
    .await?;
    Ok(data_list.into_iter().map(|raw| raw.record).collect())
}

fn get_segment_no_by_offset(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard: &EngineShard,
    shard_name: &str,
    offset: u64,
) -> Result<u32, StorageEngineError> {
    match shard.config.storage_type {
        StorageType::EngineMemory | StorageType::EngineRocksDB => Ok(shard.active_segment_seq),
        StorageType::EngineSegment => {
            if let Some(segment_no) = get_in_segment_by_offset(cache_manager, shard_name, offset)? {
                Ok(segment_no)
            } else {
                let earliest_offset =
                    get_earliest_offset(rocksdb_engine_handler, cache_manager, shard_name)?;
                let latest_offset =
                    get_latest_offset(rocksdb_engine_handler, cache_manager, shard_name)?;
                if offset <= earliest_offset {
                    Ok(shard.start_segment_seq)
                } else if offset >= latest_offset {
                    Ok(shard.active_segment_seq)
                } else {
                    Err(StorageEngineError::CommonErrorStr(format!(
                        "Offset {} is within range [{}, {}] but no segment found",
                        offset, earliest_offset, latest_offset
                    )))
                }
            }
        }
        _ => Err(StorageEngineError::CommonErrorStr(format!(
            "Unsupported storage type {:?} for shard {}",
            shard.config.storage_type, shard_name
        ))),
    }
}
