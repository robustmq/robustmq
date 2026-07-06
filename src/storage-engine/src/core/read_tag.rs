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
    clients::{manager::ClientConnectionManager, packet::build_read_req},
    commitlog::{memory::engine::MemoryStorageEngine, rocksdb::engine::RocksDBStorageEngine},
    core::{
        batch_call::{call_read_data_by_all_node, merge_records},
        cache::StorageCacheManager,
        error::StorageEngineError,
        message_ttl::is_record_expired,
        remote_read::remote_read_by_tag,
        segment::segment_validator,
    },
    filesegment::{file::open_segment_write, index::read::get_index_data_by_tag, SegmentIdentity},
};
use common_config::{broker::broker_config, storage::StorageType};
use metadata_struct::storage::{adapter_read_config::AdapterReadConfig, record::StorageRecord};
use protocol::storage::protocol::{
    ReadReq, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::collections::HashMap;
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
    pub batch_call_source: bool,
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

    let engine_type = shard.config.storage_type;
    if engine_type == StorageType::EngineMemory || engine_type == StorageType::EngineRocksDB {
        let Some(active_segment) = cache_manager.get_active_segment(shard_name) else {
            return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
        };

        let segment_iden = SegmentIdentity::new(shard_name, active_segment.segment_seq);
        segment_validator(cache_manager, &shard, &active_segment, &segment_iden)?;

        let conf = broker_config();
        let results = if conf.broker_id == active_segment.leader {
            match engine_type {
                StorageType::EngineMemory => {
                    read_by_memory(
                        memory_storage_engine,
                        shard_name,
                        tag,
                        start_offset,
                        read_config,
                    )
                    .await?
                }
                StorageType::EngineRocksDB => {
                    read_by_rocksdb(
                        rocksdb_storage_engine,
                        shard_name,
                        tag,
                        start_offset,
                        read_config,
                    )
                    .await?
                }
                _ => Vec::new(),
            }
        } else {
            remote_read_by_tag(
                client_connection_manager,
                cache_manager,
                &segment_iden,
                active_segment.leader,
                shard_name,
                tag,
                start_offset,
                read_config,
            )
            .await?
        };
        return Ok(results);
    }

    if engine_type == StorageType::EngineSegment {
        let local_records = read_by_segment(
            cache_manager,
            rocksdb_engine_handler,
            shard_name,
            tag,
            start_offset,
            read_config,
        )
        .await?;

        if params.batch_call_source {
            return Ok(local_records);
        }

        let read_req = build_req(
            &params.shard_name,
            &params.tag,
            params.start_offset,
            &params.read_config,
            true,
        );
        let remote_records =
            call_read_data_by_all_node(cache_manager, client_connection_manager, read_req).await?;

        return Ok(merge_records(local_records, remote_records));
    }

    Ok(Vec::new())
}

fn build_req(
    shard_name: &str,
    tag: &str,
    start_offset: Option<u64>,
    read_config: &AdapterReadConfig,
    batch_call_source: bool,
) -> ReadReq {
    let messages = vec![ReadReqMessage {
        shard_name: shard_name.to_string(),
        read_type: ReadType::Tag,
        batch_call_source,
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

    build_read_req(messages)
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
    tag: &str,
    start_offset: Option<u64>,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    // Look up the tag index and group positions by segment, keeping only
    // segments this node leads.  call_read_data_by_all_node fans out to the
    // other leader nodes, so every segment is covered exactly once.
    let index_list = get_index_data_by_tag(
        rocksdb_engine_handler,
        shard_name,
        start_offset,
        tag,
        read_config.max_record_num as usize,
    )?;

    let mut segment_positions: HashMap<u32, Vec<u64>> = HashMap::new();
    for idx in index_list {
        let seg_iden = SegmentIdentity::new(shard_name, idx.segment);
        if cache_manager.leader_segments.contains_key(&seg_iden.name()) {
            segment_positions
                .entry(idx.segment)
                .or_default()
                .push(idx.position);
        }
    }

    let mut results = Vec::new();
    for (segment_no, positions) in segment_positions {
        let seg_iden = SegmentIdentity::new(shard_name, segment_no);
        let mut sf = open_segment_write(cache_manager, &seg_iden).await?;
        let data_list = sf.read_by_positions(positions).await?;
        results.extend(
            data_list
                .into_iter()
                .filter(|r| !is_record_expired(&r.record.metadata))
                .map(|r| r.record),
        );
    }
    Ok(results)
}
