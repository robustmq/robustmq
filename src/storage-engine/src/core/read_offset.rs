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

use crate::core::offset::ShardOffset;
use crate::{
    clients::manager::ClientConnectionManager,
    commitlog::{memory::engine::MemoryStorageEngine, rocksdb::engine::RocksDBStorageEngine},
    core::{
        cache::StorageCacheManager,
        error::StorageEngineError,
        remote_read::{pick_replica_exclude_all, remote_read_by_offset},
        segment::segment_validator,
    },
    filesegment::{
        file::open_segment_write, index::read::get_in_segment_by_offset,
        read::segment_read_by_offset, SegmentIdentity,
    },
};
use common_config::{broker::broker_config, storage::StorageType};
use metadata_struct::storage::{
    adapter_read_config::AdapterReadConfig, record::StorageRecord, shard::EngineShard,
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
    pub single_segment: bool,
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
    let single_segment = params.single_segment;
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

    segment_validator(cache_manager, &shard, &segment, &segment_iden)?;

    let conf = broker_config();
    let results = match shard.config.storage_type {
        StorageType::EngineMemory => {
            if conf.broker_id == segment.leader {
                read_by_memory(memory_storage_engine, shard_name, offset, read_config).await?
            } else {
                remote_read_by_offset(
                    client_connection_manager,
                    cache_manager,
                    &segment_iden,
                    segment.leader,
                    shard_name,
                    offset,
                    read_config,
                    false,
                )
                .await?
            }
        }
        StorageType::EngineRocksDB => {
            if conf.broker_id == segment.leader {
                read_by_rocksdb(rocksdb_storage_engine, shard_name, offset, read_config).await?
            } else {
                remote_read_by_offset(
                    client_connection_manager,
                    cache_manager,
                    &segment_iden,
                    segment.leader,
                    shard_name,
                    offset,
                    read_config,
                    false,
                )
                .await?
            }
        }
        StorageType::EngineSegment => {
            read_by_segment(
                cache_manager,
                rocksdb_engine_handler,
                client_connection_manager,
                shard_name,
                offset,
                segment.segment_seq,
                read_config,
                single_segment,
            )
            .await?
        }
        _ => {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "Unsupported storage type {:?} for shard {}",
                shard.config.storage_type, shard_name
            )))
        }
    };
    Ok(results)
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn read_by_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    client_connection_manager: &Arc<ClientConnectionManager>,
    shard_name: &str,
    offset: u64,
    segment: u32,
    read_config: &AdapterReadConfig,
    single_segment: bool,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    if single_segment {
        read_single_segment(
            cache_manager,
            rocksdb_engine_handler,
            shard_name,
            offset,
            segment,
            read_config,
        )
        .await
    } else {
        read_multi_segment(
            cache_manager,
            rocksdb_engine_handler,
            client_connection_manager,
            shard_name,
            offset,
            segment,
            read_config,
        )
        .await
    }
}

async fn read_single_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    shard_name: &str,
    offset: u64,
    segment: u32,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let segment_iden = SegmentIdentity::new(shard_name, segment);
    let cur_segment = cache_manager
        .get_segment(&segment_iden)
        .ok_or_else(|| StorageEngineError::SegmentNotExist(segment_iden.name()))?;

    let active_seq = cache_manager
        .shards
        .get(shard_name)
        .ok_or_else(|| StorageEngineError::ShardNotExist(shard_name.to_owned()))?
        .active_segment_seq;
    let is_active = segment == active_seq;

    let ok = if is_active {
        cur_segment.is_leader()
    } else {
        cur_segment.is_replica()
    };
    if !ok {
        return Err(StorageEngineError::SegmentNotOnThisBroker(
            segment_iden.name(),
        ));
    }

    local_read(
        cache_manager,
        rocksdb_engine_handler,
        &segment_iden,
        offset,
        read_config,
    )
    .await
}

async fn read_multi_segment(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    client_connection_manager: &Arc<ClientConnectionManager>,
    shard_name: &str,
    offset: u64,
    segment: u32,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let mut results: Vec<StorageRecord> = Vec::new();
    let mut current_seq = segment;
    let mut current_offset = offset;
    let active_seq = cache_manager
        .shards
        .get(shard_name)
        .ok_or_else(|| StorageEngineError::ShardNotExist(shard_name.to_owned()))?
        .active_segment_seq;

    loop {
        let remaining = read_config
            .max_record_num
            .saturating_sub(results.len() as u64);
        if remaining == 0 {
            break;
        }
        let seg_config = AdapterReadConfig {
            max_record_num: remaining,
            max_size: read_config.max_size,
        };

        let segment_iden = SegmentIdentity::new(shard_name, current_seq);
        let cur_segment = cache_manager
            .get_segment(&segment_iden)
            .ok_or_else(|| StorageEngineError::SegmentNotExist(segment_iden.name()))?;

        if current_seq == active_seq {
            if cur_segment.is_leader() {
                let batch = local_read(
                    cache_manager,
                    rocksdb_engine_handler,
                    &segment_iden,
                    current_offset,
                    &seg_config,
                )
                .await?;
                results.extend(batch);
            } else {
                let remote = remote_read_by_offset(
                    client_connection_manager,
                    cache_manager,
                    &segment_iden,
                    cur_segment.leader,
                    shard_name,
                    current_offset,
                    &seg_config,
                    true,
                )
                .await?;
                results.extend(remote);
            }
            break;
        }

        if cur_segment.is_replica() {
            let batch = local_read(
                cache_manager,
                rocksdb_engine_handler,
                &segment_iden,
                current_offset,
                &seg_config,
            )
            .await?;
            if batch.is_empty() {
                break;
            }
            results.extend(batch);
        } else {
            let req_offset = if current_offset > 0 {
                current_offset
            } else {
                cache_manager
                    .get_segment_meta(&segment_iden)
                    .map(|m| m.start_offset.max(0) as u64)
                    .unwrap_or(0)
            };
            let initial_target = pick_replica_exclude_all(&cur_segment, &[]);
            let remote = remote_read_by_offset(
                client_connection_manager,
                cache_manager,
                &segment_iden,
                initial_target,
                shard_name,
                req_offset,
                &seg_config,
                true,
            )
            .await?;
            results.extend(remote);
        }

        let next_seq = current_seq + 1;
        if cache_manager
            .get_segment(&SegmentIdentity::new(shard_name, next_seq))
            .is_none()
        {
            break;
        }
        current_seq = next_seq;
        current_offset = 0;
    }

    Ok(results)
}

async fn local_read(
    cache_manager: &Arc<StorageCacheManager>,
    rocksdb_engine_handler: &Arc<RocksDBEngine>,
    segment_iden: &SegmentIdentity,
    offset: u64,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let mut segment_file = open_segment_write(cache_manager, segment_iden).await?;
    let batch = segment_read_by_offset(
        rocksdb_engine_handler,
        &mut segment_file,
        segment_iden,
        offset,
        read_config.max_size,
        read_config.max_record_num,
    )
    .await?;
    Ok(batch.into_iter().map(|r| r.record).collect())
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
                let file_segment_offset =
                    ShardOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
                let earliest_offset = file_segment_offset.get_earliest_offset(shard_name)?;
                let latest_offset = file_segment_offset.get_latest_offset(shard_name)?;
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

#[cfg(test)]
mod tests {
    use super::read_by_segment;
    use crate::clients::manager::ClientConnectionManager;
    use crate::core::cache::StorageCacheManager;
    use crate::core::segment::create_local_segment;
    use crate::core::test_tool::test_init_segment;
    use crate::filesegment::replica::FileSegmentReplicaLog;
    use crate::isr::log::ReplicaLog;
    use bytes::Bytes;
    use common_config::storage::StorageType;
    use metadata_struct::adapter::adapter_read_config::AdapterReadConfig;
    use metadata_struct::storage::record::{StorageRecord, StorageRecordMetadata};
    use metadata_struct::storage::segment::{EngineSegment, Replica, SegmentStatus};
    use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
    use std::sync::Arc;

    fn make_client(cm: Arc<StorageCacheManager>) -> Arc<ClientConnectionManager> {
        Arc::new(ClientConnectionManager::new(cm, 2))
    }

    fn record(offset: u64, data: &str, shard: &str, seg: u32) -> StorageRecord {
        StorageRecord {
            metadata: StorageRecordMetadata {
                offset,
                shard: shard.to_string(),
                segment: seg,
                ..Default::default()
            },
            data: Bytes::from(data.to_string()),
            protocol_data: None,
        }
    }

    async fn append(log: &FileSegmentReplicaLog, shard: &str, seg: u32, recs: Vec<StorageRecord>) {
        let base = recs.first().unwrap().metadata.offset;
        log.append_at(shard, seg, base, recs).await.unwrap();
    }

    #[tokio::test]
    async fn reads_within_single_segment() {
        let (iden, cm, fold, db) = test_init_segment(StorageType::EngineSegment).await;
        let shard = &iden.shard_name;

        let log = FileSegmentReplicaLog::new(cm.clone(), db.clone());
        append(
            &log,
            shard,
            0,
            vec![record(0, "a", shard, 0), record(1, "b", shard, 0)],
        )
        .await;

        let cfg = AdapterReadConfig {
            max_record_num: 10,
            max_size: 1024 * 1024,
        };
        let client = make_client(cm.clone());
        let results = read_by_segment(&cm, &db, &client, shard, 0, 0, &cfg, false)
            .await
            .unwrap();
        assert_eq!(results.len(), 2);
        let _ = fold;
    }

    #[tokio::test]
    async fn continues_into_next_segment_when_first_is_exhausted() {
        let (iden, cm, fold, db) = test_init_segment(StorageType::EngineSegment).await;
        let shard = &iden.shard_name;

        let seg1 = EngineSegment {
            shard_name: shard.clone(),
            segment_seq: 1,
            replicas: vec![Replica {
                replica_seq: 0,
                node_id: 1,
                fold: fold.clone(),
            }],
            leader: 1,
            leader_epoch: 0,
            status: SegmentStatus::Write,
            isr: vec![1],
            ..Default::default()
        };
        create_local_segment(&cm, &seg1).await.unwrap();
        cm.set_segment_meta(EngineSegmentMetadata {
            shard_name: shard.clone(),
            segment_seq: 1,
            start_offset: 2,
            ..Default::default()
        });
        // segment 1 is now the active segment
        let mut updated_shard = cm.shards.get(shard).unwrap().clone();
        updated_shard.active_segment_seq = 1;
        cm.set_shard(updated_shard);
        cm.sort_offset_index(shard);

        let log = FileSegmentReplicaLog::new(cm.clone(), db.clone());
        append(
            &log,
            shard,
            0,
            vec![record(0, "a", shard, 0), record(1, "b", shard, 0)],
        )
        .await;
        append(
            &log,
            shard,
            1,
            vec![record(2, "c", shard, 1), record(3, "d", shard, 1)],
        )
        .await;

        let cfg = AdapterReadConfig {
            max_record_num: 10,
            max_size: 1024 * 1024,
        };
        let client = make_client(cm.clone());
        let results = read_by_segment(&cm, &db, &client, shard, 0, 0, &cfg, false)
            .await
            .unwrap();
        assert_eq!(results.len(), 4, "expected records from both segments");
        assert_eq!(results[2].data, Bytes::from("c"));
        assert_eq!(results[3].data, Bytes::from("d"));
    }

    #[tokio::test]
    async fn respects_max_record_num_across_segments() {
        let (iden, cm, fold, db) = test_init_segment(StorageType::EngineSegment).await;
        let shard = &iden.shard_name;

        let seg1 = EngineSegment {
            shard_name: shard.clone(),
            segment_seq: 1,
            replicas: vec![Replica {
                replica_seq: 0,
                node_id: 1,
                fold: fold.clone(),
            }],
            leader: 1,
            leader_epoch: 0,
            status: SegmentStatus::Write,
            isr: vec![1],
            ..Default::default()
        };
        create_local_segment(&cm, &seg1).await.unwrap();
        cm.set_segment_meta(EngineSegmentMetadata {
            shard_name: shard.clone(),
            segment_seq: 1,
            start_offset: 2,
            ..Default::default()
        });
        // segment 1 is now the active segment
        let mut updated_shard = cm.shards.get(shard).unwrap().clone();
        updated_shard.active_segment_seq = 1;
        cm.set_shard(updated_shard);
        cm.sort_offset_index(shard);

        let log = FileSegmentReplicaLog::new(cm.clone(), db.clone());
        append(
            &log,
            shard,
            0,
            vec![record(0, "a", shard, 0), record(1, "b", shard, 0)],
        )
        .await;
        append(
            &log,
            shard,
            1,
            vec![record(2, "c", shard, 1), record(3, "d", shard, 1)],
        )
        .await;

        let cfg = AdapterReadConfig {
            max_record_num: 3,
            max_size: 1024 * 1024,
        };
        let client = make_client(cm.clone());
        let results = read_by_segment(&cm, &db, &client, shard, 0, 0, &cfg, false)
            .await
            .unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[2].data, Bytes::from("c"));
    }
}
