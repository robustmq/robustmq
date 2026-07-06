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
use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::core::offset::ShardOffset;
use crate::core::read_key::{read_by_key, ReadByKeyParams};
use crate::core::read_offset::{read_by_offset, ReadByOffsetParams};
use crate::core::read_tag::{read_by_tag, ReadByTagParams};
use crate::core::write::batch_write;
use crate::filesegment::write_manager::WriteManager;
use common_base::utils::serialize::{deserialize, serialize};
use common_config::storage::StorageType;
use metadata_struct::adapter::adapter_offset::AdapterOffsetStrategy;
use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use protocol::storage::protocol::{
    DeleteReqBody, ReadReqBody, ReadType, ShardOffsetReqBody, ShardOffsetRespBody,
    StorageEngineNetworkError, WriteRespMessage, WriteRespMessageStatus,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub async fn shard_offset_req(
    cache_manager: &Arc<StorageCacheManager>,
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    req_body: &ShardOffsetReqBody,
) -> Result<ShardOffsetRespBody, StorageEngineError> {
    let shard_name = req_body.shard_name.as_str();
    let Some(shard) = cache_manager.shards.get(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_string()));
    };

    // For EngineSegment, the caller routes to the specific segment's leader, which may
    // differ from the active-segment leader. Skip the active-segment leader check in
    // that case; for Memory/RocksDB the active segment IS the only segment, so check.
    if !matches!(shard.config.storage_type, StorageType::EngineSegment) {
        let active_segment = cache_manager
            .get_active_segment(shard_name)
            .ok_or_else(|| StorageEngineError::NotAvailableSegments(shard_name.to_string()))?;
        if !active_segment.is_leader() {
            return Err(StorageEngineError::NotLeader(active_segment.name()));
        }
    }

    let shard_offsets = ShardOffset::new(
        cache_manager.clone(),
        rocksdb_storage_engine.rocksdb_engine_handler.clone(),
    )
    .get_shard_offsets(shard_name)?;

    let start_offset = shard_offsets.earliest_offset;
    let end_offset = shard_offsets.latest_offset.saturating_sub(1);
    let high_watermark = shard_offsets.high_watermark_offset;

    let strategy = if req_body.strategy == 1 {
        AdapterOffsetStrategy::Latest
    } else {
        AdapterOffsetStrategy::Earliest
    };

    let offset = if req_body.by_timestamp {
        match shard.config.storage_type {
            StorageType::EngineMemory => {
                memory_storage_engine
                    .get_offset_by_timestamp(shard_name, req_body.timestamp, strategy)
                    .await?
            }
            StorageType::EngineRocksDB => {
                rocksdb_storage_engine
                    .get_offset_by_timestamp(shard_name, req_body.timestamp, strategy)
                    .await?
            }
            StorageType::EngineSegment => {
                crate::filesegment::read::get_segment_offset_by_timestamp(
                    cache_manager,
                    &rocksdb_storage_engine.rocksdb_engine_handler,
                    shard_name,
                    req_body.timestamp,
                    strategy,
                )?
            }
            t => return Err(StorageEngineError::UnsupportedStorageType(format!("{t:?}"))),
        }
    } else {
        0
    };

    Ok(ShardOffsetRespBody {
        start_offset,
        end_offset,
        high_watermark,
        offset,
        error_code: 0,
    })
}

fn params_validator(
    cache_manager: &Arc<StorageCacheManager>,
    shard_name: &str,
) -> Result<(), StorageEngineError> {
    if !cache_manager.shards.contains_key(shard_name) {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_string()));
    }

    Ok(())
}

/// Apply a delete locally on this (leader) node. Forwarded here by a non-leader
/// node via the engine protocol; never re-forwards.
pub async fn delete_data_req(
    cache_manager: &Arc<StorageCacheManager>,
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    body: &DeleteReqBody,
) -> Result<(), StorageEngineError> {
    let shard_name = body.shard_name.as_str();
    let Some(shard) = cache_manager.shards.get(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_string()));
    };

    match shard.config.storage_type {
        StorageType::EngineMemory => {
            for key in &body.keys {
                memory_storage_engine.delete_by_key(shard_name, key).await?;
            }
            for &offset in &body.offsets {
                memory_storage_engine
                    .delete_by_offset(shard_name, offset)
                    .await?;
            }
        }
        StorageType::EngineRocksDB => {
            if !body.keys.is_empty() {
                let key_refs: Vec<&[u8]> = body.keys.iter().map(|k| k.as_ref()).collect();
                rocksdb_storage_engine
                    .delete_by_keys(shard_name, &key_refs)
                    .await?;
            }
            if !body.offsets.is_empty() {
                rocksdb_storage_engine
                    .delete_by_offsets(shard_name, &body.offsets)
                    .await?;
            }
        }
        _ => {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "delete is not supported for storage type {:?}",
                shard.config.storage_type
            )))
        }
    }
    Ok(())
}

/// Delete all records with offset < `target_offset` on this (leader) node.
/// Forwarded here by a non-leader node via the engine protocol; never re-forwards.
/// Returns the achieved low_watermark (may be less than `target_offset` if it
/// exceeds the shard's latest offset).
pub async fn delete_records_before_req(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    shard_name: &str,
    target_offset: u64,
) -> Result<u64, StorageEngineError> {
    let Some(shard) = cache_manager.shards.get(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_string()));
    };
    let storage_type = shard.config.storage_type;
    drop(shard);

    match storage_type {
        StorageType::EngineMemory | StorageType::EngineRocksDB => {
            let shard_offset =
                ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
            let earliest = shard_offset.get_earliest_offset(shard_name)?;
            let latest = shard_offset.get_latest_offset(shard_name)?;
            let target = target_offset.min(latest);
            if target <= earliest {
                return Ok(earliest);
            }

            let offsets: Vec<u64> = (earliest..target).collect();
            match storage_type {
                StorageType::EngineMemory => {
                    for &offset in &offsets {
                        memory_storage_engine
                            .delete_by_offset(shard_name, offset)
                            .await?;
                    }
                }
                StorageType::EngineRocksDB => {
                    rocksdb_storage_engine
                        .delete_by_offsets(shard_name, &offsets)
                        .await?;
                }
                _ => unreachable!(),
            }
            shard_offset.save_earliest_offset(shard_name, target)?;
            Ok(target)
        }
        StorageType::EngineSegment => {
            crate::filesegment::delete::delete_segments_before_offset(
                cache_manager,
                rocksdb_engine_handler,
                shard_name,
                target_offset,
            )
            .await
        }
        _ => Err(StorageEngineError::CommonErrorStr(format!(
            "delete_records_before is not supported for storage type {:?}",
            storage_type
        ))),
    }
}

/// the entry point for handling write requests
#[allow(clippy::too_many_arguments)]
pub async fn write_data_req(
    cache_manager: &Arc<StorageCacheManager>,
    write_manager: &Arc<WriteManager>,
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    client_connection_manager: &Arc<ClientConnectionManager>,
    shard_name: &str,
    messages: &[Vec<u8>],
    acks: i8,
    timeout_ms: u64,
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
        acks,
        timeout_ms,
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
                    single_segment: raw.batch_call_source,
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
                    batch_call_source: raw.batch_call_source,
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
                    batch_call_source: raw.batch_call_source,
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
    use crate::commitlog::memory::engine::MemoryStorageEngine;
    use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
    use crate::core::offset::ShardOffset;
    use crate::core::test_tool::test_init_segment;
    use crate::filesegment::write_manager::WriteManager;
    use crate::handler::data::read_data_req;
    use crate::{clients::manager::ClientConnectionManager, handler::data::write_data_req};
    use bytes::Bytes;
    use common_base::utils::serialize::{self, deserialize};
    use common_config::storage::memory::StorageDriverMemoryConfig;
    use common_config::storage::StorageType;
    use grpc_clients::pool::ClientPool;
    use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
    use metadata_struct::storage::record::StorageRecord;
    use protocol::storage::protocol::{
        ReadReqBody, ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType,
    };
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::sync::broadcast;
    use tokio::time::sleep;

    #[tokio::test]
    async fn delete_records_before_req_test_by_memory() {
        delete_records_before_req_test(StorageType::EngineMemory).await;
    }

    #[tokio::test]
    async fn delete_records_before_req_test_by_rocksdb() {
        delete_records_before_req_test(StorageType::EngineRocksDB).await;
    }

    async fn delete_records_before_req_test(engine_storage_type: StorageType) {
        use crate::core::cache::StorageCacheManager;
        use crate::handler::data::delete_records_before_req;
        use broker_core::cache::NodeCacheManager;
        use common_base::uuid::unique_id;
        use common_config::config::BrokerConfig;
        use metadata_struct::storage::shard::{EngineShard, EngineShardConfig, EngineShardStatus};
        use rocksdb_engine::test::test_rocksdb_instance;

        let rocksdb_engine_handler = test_rocksdb_instance();
        let broker_cache = Arc::new(NodeCacheManager::new(BrokerConfig::default()));
        let cache_manager = Arc::new(StorageCacheManager::new(broker_cache));

        let shard_name = unique_id();
        cache_manager.set_shard(EngineShard {
            shard_uid: unique_id(),
            shard_name: shard_name.clone(),
            topic_name: "".to_string(),
            start_segment_seq: 0,
            active_segment_seq: 0,
            last_segment_seq: 0,
            status: EngineShardStatus::Run,
            config: EngineShardConfig {
                storage_type: engine_storage_type,
                ..Default::default()
            },
            desc: "".to_string(),
            create_time: 0,
        });

        let memory_storage_engine = Arc::new(MemoryStorageEngine::new(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
            StorageDriverMemoryConfig::default(),
        ));
        let rocksdb_storage_engine = Arc::new(RocksDBStorageEngine::new(
            cache_manager.clone(),
            rocksdb_engine_handler.clone(),
        ));

        let shard_offset = ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
        shard_offset.save_earliest_offset(&shard_name, 0).unwrap();
        shard_offset.save_latest_offset(&shard_name, 0).unwrap();

        let messages: Vec<AdapterWriteRecord> = (0..10)
            .map(|i| {
                AdapterWriteRecord::new(shard_name.clone(), Bytes::from(format!("data{i}")))
                    .with_key(format!("key-{i}"))
            })
            .collect();

        match engine_storage_type {
            StorageType::EngineMemory => {
                memory_storage_engine
                    .batch_write(&shard_name, &messages)
                    .await
                    .unwrap();
            }
            StorageType::EngineRocksDB => {
                rocksdb_storage_engine
                    .batch_write(&shard_name, &messages)
                    .await
                    .unwrap();
            }
            _ => unreachable!(),
        }
        shard_offset.save_latest_offset(&shard_name, 10).unwrap();

        // target falls mid-range: offsets [0,4) deleted, low_watermark advances to 4
        let achieved = delete_records_before_req(
            &cache_manager,
            &rocksdb_engine_handler,
            &memory_storage_engine,
            &rocksdb_storage_engine,
            &shard_name,
            4,
        )
        .await
        .unwrap();
        assert_eq!(achieved, 4);
        assert_eq!(shard_offset.get_earliest_offset(&shard_name).unwrap(), 4);

        // target beyond latest offset is clamped to latest, not overshot
        let achieved = delete_records_before_req(
            &cache_manager,
            &rocksdb_engine_handler,
            &memory_storage_engine,
            &rocksdb_storage_engine,
            &shard_name,
            1000,
        )
        .await
        .unwrap();
        assert_eq!(achieved, 10);
        assert_eq!(shard_offset.get_earliest_offset(&shard_name).unwrap(), 10);
    }

    #[tokio::test]
    async fn read_data_req_test_by_segment() {
        read_data_req_test(StorageType::EngineSegment).await;
    }

    #[tokio::test]
    async fn read_data_req_test_by_memory() {
        read_data_req_test(StorageType::EngineMemory).await;
    }

    #[tokio::test]
    async fn read_data_req_test_by_rocksdb() {
        read_data_req_test(StorageType::EngineRocksDB).await;
    }

    async fn read_data_req_test(engine_storage_type: StorageType) {
        let (segment_iden, cache_manager, _, rocksdb_engine_handler) =
            test_init_segment(engine_storage_type).await;

        let commit_offset = ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
        commit_offset
            .save_earliest_offset(&segment_iden.shard_name, 0)
            .unwrap();
        commit_offset
            .save_latest_offset(&segment_iden.shard_name, 0)
            .unwrap();
        cache_manager.save_offset_state(
            segment_iden.shard_name.clone(),
            crate::core::offset::ShardOffsetState::default(),
        );

        let shard_info = cache_manager.shards.get(&segment_iden.shard_name).unwrap();
        assert_eq!(shard_info.config.storage_type, engine_storage_type);

        let client_poll = Arc::new(ClientPool::new(100));

        let write_manager = Arc::new(WriteManager::new(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
            client_poll.clone(),
            3,
        ));

        let (stop_send, _) = broadcast::channel(2);
        write_manager.start(stop_send.clone());

        sleep(Duration::from_millis(100)).await;

        let memory_storage_engine = Arc::new(MemoryStorageEngine::new(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
            StorageDriverMemoryConfig::default(),
        ));
        let rocksdb_storage_engine = Arc::new(RocksDBStorageEngine::new(
            cache_manager.clone(),
            rocksdb_engine_handler.clone(),
        ));
        let client_connection_manager =
            Arc::new(ClientConnectionManager::new(cache_manager.clone(), 8));
        let mut messages = Vec::new();
        for i in 0..10 {
            let record = AdapterWriteRecord::new(
                segment_iden.shard_name.to_string(),
                Bytes::from("dsfsfsdfsf"),
            )
            .with_key(format!("key-{}", i))
            .with_tags(vec![format!("tag-{}", i)]);
            messages.push(serialize::serialize(&record).unwrap());
        }
        write_data_req(
            &cache_manager,
            &write_manager,
            &memory_storage_engine,
            &rocksdb_storage_engine,
            &client_connection_manager,
            &segment_iden.shard_name,
            &messages,
            1,
            0,
        )
        .await
        .unwrap();

        // offset
        let req_body = ReadReqBody {
            messages: vec![ReadReqMessage {
                shard_name: segment_iden.shard_name.clone(),
                read_type: ReadType::Offset,
                batch_call_source: true,
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

        for (i, record_bytes) in (5..).zip(resp.iter()) {
            let record: StorageRecord = deserialize(record_bytes).unwrap();
            assert_eq!(record.metadata.offset, i);
        }

        // key
        let key = format!("key-{}", 1);
        let req_body = ReadReqBody {
            messages: vec![ReadReqMessage {
                shard_name: segment_iden.shard_name.clone(),
                read_type: ReadType::Key,
                batch_call_source: true,
                filter: ReadReqFilter {
                    offset: Some(0),
                    key: Some(key.clone().into()),
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
                batch_call_source: true,
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
