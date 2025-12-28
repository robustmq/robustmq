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
    core::{cache::StorageCacheManager, error::StorageEngineError, segment::segment_validator},
    memory::engine::MemoryStorageEngine,
    rocksdb::engine::RocksDBStorageEngine,
    segment::{file::open_segment_write, read::segment_read_by_tag, SegmentIdentity},
};
use common_config::broker::broker_config;
use metadata_struct::storage::{
    adapter_read_config::AdapterReadConfig, shard::EngineType, storage_record::StorageRecord,
};
use protocol::storage::{
    codec::StorageEnginePacket,
    protocol::{ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType},
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub struct ReadByTagParams {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub cache_manager: Arc<StorageCacheManager>,
    pub memory_storage_engine: Arc<MemoryStorageEngine>,
    pub rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
    pub client_connection_manager: Arc<ClientConnectionManager>,
    pub shard_name: String,
    pub tag: String,
    pub start_offset: Option<u64>,
    pub read_config: AdapterReadConfig,
}

pub async fn read_by_tag(
    params: ReadByTagParams,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let rocksdb_engine_handler = &params.rocksdb_engine_handler;
    let cache_manager = &params.cache_manager;
    let memory_storage_engine = &params.memory_storage_engine;
    let rocksdb_storage_engine = &params.rocksdb_storage_engine;
    let client_connection_manager = &params.client_connection_manager;
    let shard_name = params.shard_name.as_str();
    let tag = params.tag.as_str();
    let start_offset = params.start_offset;
    let read_config = &params.read_config;
    let Some(shard) = cache_manager.shards.get(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
    };

    let Some(active_segment) = cache_manager.get_active_segment(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
    };

    segment_validator(cache_manager, shard_name, active_segment.segment_seq)?;

    let conf = broker_config();
    let results = if conf.broker_id == active_segment.leader {
        read_by_remote(
            client_connection_manager,
            conf.broker_id,
            shard_name,
            tag,
            start_offset,
            read_config,
        )
        .await?
    } else {
        match shard.engine_type {
            EngineType::Memory => {
                read_by_memory(
                    memory_storage_engine,
                    shard_name,
                    tag,
                    start_offset,
                    read_config,
                )
                .await?
            }
            EngineType::RocksDB => {
                read_by_rocksdb(
                    rocksdb_storage_engine,
                    shard_name,
                    tag,
                    start_offset,
                    read_config,
                )
                .await?
            }
            EngineType::Segment => {
                read_by_segment(
                    cache_manager,
                    rocksdb_engine_handler,
                    shard_name,
                    active_segment.segment_seq,
                    tag,
                    start_offset,
                    read_config,
                )
                .await?
            }
        }
    };
    Ok(results)
}

pub async fn read_by_remote(
    client_connection_manager: &Arc<ClientConnectionManager>,
    target_broker_id: u64,
    shard_name: &str,
    tag: &str,
    start_offset: Option<u64>,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let messages = vec![ReadReqMessage {
        shard_name: shard_name.to_string(),
        read_type: ReadType::Tag,
        filter: ReadReqFilter {
            tag: Some(tag.to_string()),
            offset: start_offset,
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
    tag: &str,
    start_offset: Option<u64>,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    memory_storage_engine
        .read_by_tag(shard_name, tag, start_offset, read_config)
        .await
}

async fn read_by_rocksdb(
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    shard_name: &str,
    tag: &str,
    start_offset: Option<u64>,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    rocksdb_storage_engine
        .read_by_tag(shard_name, tag, start_offset, read_config)
        .await
}

async fn read_by_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    segment: u32,
    tag: &str,
    start_offset: Option<u64>,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let segment_iden = SegmentIdentity::new(shard_name, segment);
    let segment_file = open_segment_write(cache_manager, &segment_iden).await?;
    let data_list = segment_read_by_tag(
        rocksdb_engine_handler,
        &segment_file,
        &segment_iden,
        tag,
        start_offset,
        read_config.max_record_num,
    )
    .await?;
    Ok(data_list.iter().map(|raw| raw.record.clone()).collect())
}
