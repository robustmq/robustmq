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
        batch_call::{call_read_data_by_all_node, merge_records},
        cache::StorageCacheManager,
        error::StorageEngineError,
        segment::segment_validator,
    },
    memory::engine::MemoryStorageEngine,
    rocksdb::engine::RocksDBStorageEngine,
    segment::read::segment_read_by_key,
};
use common_config::{broker::broker_config, storage::StorageType};
use metadata_struct::storage::storage_record::StorageRecord;
use protocol::storage::{
    codec::StorageEnginePacket,
    protocol::{ReadReq, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType},
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub struct ReadByKeyParams {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub cache_manager: Arc<StorageCacheManager>,
    pub memory_storage_engine: Arc<MemoryStorageEngine>,
    pub rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
    pub client_connection_manager: Arc<ClientConnectionManager>,
    pub shard_name: String,
    pub key: String,
}

pub struct ReadByRemoteKeyParams {
    pub cache_manager: Arc<StorageCacheManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub client_connection_manager: Arc<ClientConnectionManager>,
    pub shard_name: String,
    pub segment: u32,
    pub key: String,
}

pub async fn read_by_key(
    params: ReadByKeyParams,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let rocksdb_engine_handler = &params.rocksdb_engine_handler;
    let cache_manager = &params.cache_manager;
    let memory_storage_engine = &params.memory_storage_engine;
    let rocksdb_storage_engine = &params.rocksdb_storage_engine;
    let client_connection_manager = &params.client_connection_manager;
    let shard_name = params.shard_name.as_str();
    let key = params.key.as_str();
    let Some(shard) = cache_manager.shards.get(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
    };

    let engine_type = shard.config.storage_type;
    if engine_type == StorageType::EngineMemory || engine_type == StorageType::EngineRocksDB {
        let Some(active_segment) = cache_manager.get_active_segment(shard_name) else {
            return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
        };

        segment_validator(cache_manager, shard_name, active_segment.segment_seq)?;
        let conf = broker_config();
        let results = if conf.broker_id == active_segment.leader {
            match engine_type {
                StorageType::EngineMemory => {
                    read_by_memory(memory_storage_engine, shard_name, key).await?
                }
                StorageType::EngineRocksDB => {
                    read_by_rocksdb(rocksdb_storage_engine, shard_name, key).await?
                }
                _ => Vec::new(),
            }
        } else {
            read_by_remote(ReadByRemoteKeyParams {
                cache_manager: cache_manager.clone(),
                rocksdb_engine_handler: rocksdb_engine_handler.clone(),
                client_connection_manager: client_connection_manager.clone(),
                shard_name: shard_name.to_string(),
                segment: active_segment.segment_seq,
                key: key.to_string(),
            })
            .await?
        };
        return Ok(results);
    }

    if engine_type == StorageType::EngineSegment {
        let local_records =
            read_by_segment(cache_manager, rocksdb_engine_handler, shard_name, key).await?;

        let conf = broker_config();
        let read_req = build_req(&params.shard_name, &params.key);
        let remote_records = call_read_data_by_all_node(
            cache_manager,
            client_connection_manager,
            conf.broker_id,
            read_req,
        )
        .await?;

        return Ok(merge_records(local_records, remote_records));
    }

    Ok(Vec::new())
}

pub async fn read_by_remote(
    params: ReadByRemoteKeyParams,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let client_connection_manager = &params.client_connection_manager;
    let read_req = build_req(&params.shard_name, &params.key);
    let conf = broker_config();
    let resp = client_connection_manager
        .write_send(conf.broker_id, StorageEnginePacket::ReadReq(read_req))
        .await?;

    match resp {
        StorageEnginePacket::ReadResp(resp) => Ok(read_resp_parse(&resp)?),
        packet => Err(StorageEngineError::ReceivedPacketError(
            conf.broker_id,
            format!("Expected ReadResp, got {:?}", packet),
        )),
    }
}

fn build_req(shard_name: &str, key: &str) -> ReadReq {
    let messages = vec![ReadReqMessage {
        shard_name: shard_name.to_string(),
        read_type: ReadType::Key,
        filter: ReadReqFilter {
            key: Some(key.to_string()),
            ..Default::default()
        },
        options: ReadReqOptions::default(),
    }];
    build_read_req(messages)
}

async fn read_by_memory(
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    shard_name: &str,
    key: &str,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    memory_storage_engine.read_by_key(shard_name, key).await
}

async fn read_by_rocksdb(
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    shard_name: &str,
    key: &str,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    rocksdb_storage_engine.read_by_key(shard_name, key).await
}

async fn read_by_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    key: &str,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let data_list =
        segment_read_by_key(cache_manager, rocksdb_engine_handler, shard_name, key).await?;
    Ok(data_list.iter().map(|raw| raw.record.clone()).collect())
}
