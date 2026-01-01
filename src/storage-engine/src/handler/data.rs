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

use crate::clients::manager::ClientConnectionManager;
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::core::read_key::{read_by_key, ReadByKeyParams};
use crate::core::read_offset::{read_by_offset, ReadByOffsetParams};
use crate::core::read_tag::{read_by_tag, ReadByTagParams};
use crate::core::wirte::batch_write;
use crate::memory::engine::MemoryStorageEngine;
use crate::rocksdb::engine::RocksDBStorageEngine;
use crate::segment::write::WriteManager;
use common_base::utils::serialize::{deserialize, serialize};
use metadata_struct::storage::adapter_read_config::AdapterReadConfig;
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use protocol::storage::protocol::{
    ReadReqBody, ReadType, StorageEngineNetworkError, WriteRespMessage, WriteRespMessageStatus,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

fn params_validator(
    cache_manager: &Arc<StorageCacheManager>,
    shard_name: &str,
) -> Result<(), StorageEngineError> {
    if !cache_manager.shards.contains_key(shard_name) {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_string()));
    }

    Ok(())
}

/// the entry point for handling write requests
pub async fn write_data_req(
    cache_manager: &Arc<StorageCacheManager>,
    write_manager: &Arc<WriteManager>,
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    client_connection_manager: &Arc<ClientConnectionManager>,
    shard_name: &str,
    messages: &[Vec<u8>],
) -> Result<Vec<WriteRespMessage>, StorageEngineError> {
    if messages.is_empty() {
        return Ok(Vec::new());
    }

    params_validator(cache_manager, shard_name)?;

    let mut record_list = Vec::new();
    for message_bytes in messages {
        let adapter_record = deserialize::<AdapterWriteRecord>(message_bytes)?;
        record_list.push(adapter_record);
    }

    let response = batch_write(
        write_manager,
        cache_manager,
        memory_storage_engine,
        rocksdb_storage_engine,
        client_connection_manager,
        shard_name,
        &record_list,
    )
    .await?;

    let messages: Vec<WriteRespMessageStatus> = response
        .iter()
        .map(|row| {
            if row.is_error() {
                WriteRespMessageStatus {
                    pkid: row.pkid,
                    error: Some(StorageEngineNetworkError::new(
                        "InternalError".to_string(),
                        row.error_info(),
                    )),
                    ..Default::default()
                }
            } else {
                WriteRespMessageStatus {
                    pkid: row.pkid,
                    offset: row.offset,
                    ..Default::default()
                }
            }
        })
        .collect();

    let resp_message = WriteRespMessage {
        shard_name: shard_name.to_string(),
        messages,
    };

    Ok(vec![resp_message])
}

/// handle all read requests from Journal Client
///
/// Redirect read requests to the corresponding handler according to the read type
pub async fn read_data_req(
    cache_manager: &Arc<StorageCacheManager>,
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    client_connection_manager: &Arc<ClientConnectionManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    req_body: &ReadReqBody,
) -> Result<Vec<Vec<u8>>, StorageEngineError> {
    let mut results = Vec::new();
    for raw in req_body.messages.iter() {
        // Create AdapterReadConfig from options
        let read_config = AdapterReadConfig {
            max_record_num: raw.options.max_record,
            max_size: raw.options.max_size,
        };

        let read_data_list = match raw.read_type {
            ReadType::Offset => {
                let offset = raw.filter.offset.ok_or(StorageEngineError::CommonErrorStr(
                    "Offset is required for Offset read type".to_string(),
                ))?;

                read_by_offset(ReadByOffsetParams {
                    rocksdb_engine_handler: rocksdb_engine_handler.clone(),
                    cache_manager: cache_manager.clone(),
                    memory_storage_engine: memory_storage_engine.clone(),
                    rocksdb_storage_engine: rocksdb_storage_engine.clone(),
                    client_connection_manager: client_connection_manager.clone(),
                    shard_name: raw.shard_name.clone(),
                    offset,
                    read_config,
                })
                .await?
            }

            ReadType::Key => {
                let key = raw
                    .filter
                    .key
                    .clone()
                    .ok_or(StorageEngineError::CommonErrorStr(
                        "Key is required for Key read type".to_string(),
                    ))?;

                read_by_key(ReadByKeyParams {
                    rocksdb_engine_handler: rocksdb_engine_handler.clone(),
                    cache_manager: cache_manager.clone(),
                    memory_storage_engine: memory_storage_engine.clone(),
                    rocksdb_storage_engine: rocksdb_storage_engine.clone(),
                    client_connection_manager: client_connection_manager.clone(),
                    shard_name: raw.shard_name.clone(),
                    key,
                })
                .await?
            }

            ReadType::Tag => {
                let tag = raw
                    .filter
                    .tag
                    .clone()
                    .ok_or(StorageEngineError::CommonErrorStr(
                        "Tag is required for Tag read type".to_string(),
                    ))?;

                read_by_tag(ReadByTagParams {
                    rocksdb_engine_handler: rocksdb_engine_handler.clone(),
                    cache_manager: cache_manager.clone(),
                    memory_storage_engine: memory_storage_engine.clone(),
                    rocksdb_storage_engine: rocksdb_storage_engine.clone(),
                    client_connection_manager: client_connection_manager.clone(),
                    shard_name: raw.shard_name.clone(),
                    tag,
                    start_offset: raw.filter.offset,
                    read_config,
                })
                .await?
            }
        };

        for read_data in read_data_list {
            results.push(serialize(&read_data)?);
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use crate::clients::manager::ClientConnectionManager;
    use crate::group::OffsetManager;
    use crate::memory::engine::MemoryStorageEngine;
    use crate::rocksdb::engine::RocksDBStorageEngine;
    use crate::{core::test_tool::test_base_write_data, handler::data::read_data_req};
    use common_base::utils::serialize::deserialize;
    use common_config::storage::memory::StorageDriverMemoryConfig;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::storage::storage_record::StorageRecord;
    use protocol::storage::protocol::{
        ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType,
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn read_data_req_test() {
        let (segment_iden, cache_manager, _, rocksdb_engine_handler) =
            test_base_write_data(30).await;
        let client_pool = Arc::new(ClientPool::new(8));
        let offset_manager = Arc::new(OffsetManager::new(
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
            true,
        ));
        let memory_storage_engine = Arc::new(MemoryStorageEngine::create_storage(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
            offset_manager.clone(),
            StorageDriverMemoryConfig::default(),
        ));
        let rocksdb_storage_engine = Arc::new(RocksDBStorageEngine::create_storage(
            cache_manager.clone(),
            rocksdb_engine_handler.clone(),
            offset_manager.clone(),
        ));
        let client_connection_manager =
            Arc::new(ClientConnectionManager::new(cache_manager.clone(), 8));

        // offset
        let req_body = ReadReqBody {
            messages: vec![ReadReqMessage {
                shard_name: segment_iden.shard_name.clone(),
                read_type: ReadType::Offset,
                filter: ReadReqFilter {
                    offset: Some(5),
                    ..Default::default()
                },
                options: ReadReqOptions {
                    max_size: 1024 * 1024 * 1024,
                    max_record: 2,
                },
            }],
        };
        let res = read_data_req(
            &cache_manager,
            &memory_storage_engine,
            &rocksdb_storage_engine,
            &client_connection_manager,
            &rocksdb_engine_handler,
            &req_body,
        )
        .await;
        let resp = res.unwrap();

        assert_eq!(resp.len(), 2);

        let mut i = 5;
        for record_bytes in resp.iter() {
            let record: StorageRecord = deserialize(record_bytes).unwrap();
            assert_eq!(record.metadata.offset, i);
            i += 1;
        }

        // key
        let key = format!("key-{}", 1);
        let req_body = ReadReqBody {
            messages: vec![ReadReqMessage {
                shard_name: segment_iden.shard_name.clone(),
                read_type: ReadType::Key,
                filter: ReadReqFilter {
                    offset: Some(0),
                    key: Some(key.clone()),
                    ..Default::default()
                },
                options: ReadReqOptions {
                    max_size: 1024 * 1024 * 1024,
                    max_record: 2,
                },
            }],
        };

        let res = read_data_req(
            &cache_manager,
            &memory_storage_engine,
            &rocksdb_storage_engine,
            &client_connection_manager,
            &rocksdb_engine_handler,
            &req_body,
        )
        .await;
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert_eq!(resp.len(), 1);
        let record_bytes = resp.first().unwrap();
        let record: StorageRecord = deserialize(record_bytes).unwrap();
        assert_eq!(record.metadata.key.unwrap(), key);

        // tag
        let tag = format!("tag-{}", 1);
        let req_body = ReadReqBody {
            messages: vec![ReadReqMessage {
                shard_name: segment_iden.shard_name.clone(),
                read_type: ReadType::Tag,
                filter: ReadReqFilter {
                    offset: Some(0),
                    tag: Some(tag.clone()),
                    ..Default::default()
                },
                options: ReadReqOptions {
                    max_size: 1024 * 1024 * 1024,
                    max_record: 2,
                },
            }],
        };

        let res = read_data_req(
            &cache_manager,
            &memory_storage_engine,
            &rocksdb_storage_engine,
            &client_connection_manager,
            &rocksdb_engine_handler,
            &req_body,
        )
        .await;
        assert!(res.is_ok());
        let resp = res.unwrap();
        assert_eq!(resp.len(), 1);
        let record_bytes = resp.first().unwrap();
        let record: StorageRecord = deserialize(record_bytes).unwrap();
        assert!(record.metadata.tags.unwrap().contains(&tag));
    }
}
