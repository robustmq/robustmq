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
        packet::{build_write_req, write_resp_parse},
    },
    commitlog::memory::engine::MemoryStorageEngine,
    commitlog::rocksdb::engine::RocksDBStorageEngine,
    core::{cache::StorageCacheManager, error::StorageEngineError, segment::segment_validator},
    filesegment::{
        write::{WriteChannelDataRecord, WriteManager},
        SegmentIdentity,
    },
    isr::follower::{advance_hw, wait_for_hw},
};
use common_base::utils::serialize::serialize;
use common_config::{broker::broker_config, storage::StorageType};
use metadata_struct::storage::{
    adapter_read_config::AdapterWriteRespRow, adapter_record::AdapterWriteRecord,
};
use protocol::storage::codec::StorageEnginePacket;
use std::sync::Arc;
use tracing::warn;

const ACKS_ALL: i8 = -1;

#[allow(clippy::too_many_arguments)]
pub async fn batch_write(
    write_manager: &Arc<WriteManager>,
    cache_manager: &Arc<StorageCacheManager>,
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    client_connection_manager: &Arc<ClientConnectionManager>,
    shard_name: &str,
    records: &[AdapterWriteRecord],
    acks: i8,
) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
    let Some(shard) = cache_manager.shards.get(shard_name) else {
        return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
    };

    let Some(active_segment) = cache_manager.get_active_segment(shard_name) else {
        return Err(StorageEngineError::SegmentNotExist(shard_name.to_owned()));
    };

    let segment_iden = SegmentIdentity::new(shard_name, active_segment.segment_seq);
    segment_validator(cache_manager, &shard, &active_segment, &segment_iden)?;

    let conf = broker_config();

    if conf.broker_id != active_segment.leader {
        return write_data_to_remote(
            client_connection_manager,
            active_segment.leader,
            shard_name,
            records,
        )
        .await;
    }

    if acks == ACKS_ALL {
        let isr_size = active_segment.isr.len() as u32;
        let min_isr = shard.config.min_in_sync_replicas;
        if isr_size < min_isr {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "NotEnoughReplicas: ISR size {isr_size} < min_in_sync_replicas {min_isr} for shard {shard_name}"
            )));
        }
    }

    let offsets = match shard.config.storage_type {
        StorageType::EngineMemory => {
            write_memory_to_local(memory_storage_engine, shard_name, records).await?
        }
        StorageType::EngineRocksDB => {
            write_rocksdb_to_local(rocksdb_storage_engine, shard_name, records).await?
        }
        StorageType::EngineSegment => {
            write_segment_to_local(
                write_manager,
                shard_name,
                active_segment.segment_seq,
                records,
            )
            .await?
        }
        _ => {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "Unsupported storage type {:?} for shard {} when writing data",
                shard.config.storage_type, shard_name
            )))
        }
    };

    let leader_leo = cache_manager
        .get_offset_state(shard_name)
        .map(|s| s.latest_offset)
        .ok_or_else(|| {
            StorageEngineError::CommonErrorStr(format!(
                "offset state not found for shard {shard_name}, shard may not be initialized"
            ))
        })?;

    if advance_hw(
        cache_manager,
        shard_name,
        active_segment.segment_seq,
        &active_segment.isr,
        active_segment.leader,
        leader_leo,
    )
    .is_none()
    {
        warn!(
            "advance_hw: replica state missing for shard {shard_name} segment {}",
            active_segment.segment_seq
        );
    }

    if acks == ACKS_ALL {
        let max_wait_ms = conf.storage_runtime.replica_fetch_max_wait_ms.max(1);
        if !wait_for_hw(cache_manager, shard_name, leader_leo, max_wait_ms).await {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "acks=all timed out waiting for HW to reach {leader_leo} on shard {shard_name}"
            )));
        }
    }

    Ok(offsets)
}

async fn write_data_to_remote(
    client_connection_manager: &Arc<ClientConnectionManager>,
    target_broker_id: u64,
    shard_name: &str,
    records: &[AdapterWriteRecord],
) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
    let messages = records
        .iter()
        .map(serialize)
        .collect::<Result<Vec<_>, _>>()?;
    let write_req = build_write_req(shard_name.to_string(), messages);
    let resp = client_connection_manager
        .write_send(target_broker_id, StorageEnginePacket::WriteReq(write_req))
        .await?;

    match resp {
        StorageEnginePacket::WriteResp(resp) => Ok(write_resp_parse(&resp)?),
        packet => Err(StorageEngineError::ReceivedPacketError(
            target_broker_id,
            format!("Expected WriteResp, got {:?}", packet),
        )),
    }
}

async fn write_memory_to_local(
    memory_storage_engine: &Arc<MemoryStorageEngine>,
    shard_name: &str,
    records: &[AdapterWriteRecord],
) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
    let offsets = memory_storage_engine
        .batch_write(shard_name, records)
        .await?;
    Ok(offsets)
}

async fn write_rocksdb_to_local(
    rocksdb_storage_engine: &Arc<RocksDBStorageEngine>,
    shard_name: &str,
    records: &[AdapterWriteRecord],
) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
    let offsets = rocksdb_storage_engine
        .batch_write(shard_name, records)
        .await?;
    Ok(offsets)
}

async fn write_segment_to_local(
    write_manager: &Arc<WriteManager>,
    shard_name: &str,
    segment: u32,
    records: &[AdapterWriteRecord],
) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
    let segment_iden = SegmentIdentity::new(shard_name, segment);
    let data_list = records
        .iter()
        .map(|record| WriteChannelDataRecord {
            pkid: record.record_id,
            header: record.header.as_ref().map(|headers| {
                headers
                    .iter()
                    .map(|h| metadata_struct::storage::record::StorageHeader {
                        name: h.name.clone(),
                        value: h.value.clone(),
                    })
                    .collect()
            }),
            key: record.key.clone(),
            tags: record.tags.clone(),
            value: record.data.clone(),
            expire_at: record.expire_at,
            protocol_data: record.protocol_data.clone(),
        })
        .collect();
    let resp = write_manager.write(&segment_iden, data_list).await?;
    if let Some(err) = resp.error {
        return Err(StorageEngineError::CommonErrorStr(err));
    }

    Ok(resp.offsets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commitlog::offset::CommitLogOffset;
    use crate::core::test_tool::test_init_segment;
    use crate::isr::follower::update_follower_progress;
    use bytes::Bytes;
    use grpc_clients::pool::ClientPool;
    use std::time::Duration;
    use tokio::sync::broadcast;

    struct Env {
        write_manager: Arc<WriteManager>,
        cache_manager: Arc<StorageCacheManager>,
        memory: Arc<MemoryStorageEngine>,
        rocksdb: Arc<RocksDBStorageEngine>,
        client: Arc<ClientConnectionManager>,
        shard: String,
    }

    async fn env() -> Env {
        let (segment_iden, cache_manager, _, rocksdb_engine_handler) =
            test_init_segment(StorageType::EngineMemory).await;
        let offset = CommitLogOffset::new(cache_manager.clone(), rocksdb_engine_handler.clone());
        offset
            .save_earliest_offset(&segment_iden.shard_name, 0)
            .unwrap();
        offset
            .save_latest_offset(&segment_iden.shard_name, 0)
            .unwrap();
        cache_manager.save_offset_state(
            segment_iden.shard_name.clone(),
            crate::core::shard::ShardOffsetState::default(),
        );
        cache_manager.add_segment_replica(&segment_iden.shard_name, segment_iden.segment);

        let client_pool = Arc::new(ClientPool::new(100));
        let write_manager = Arc::new(WriteManager::new(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
            client_pool,
            3,
        ));
        let (stop, _) = broadcast::channel(2);
        write_manager.start(stop);
        let memory = Arc::new(MemoryStorageEngine::new(
            rocksdb_engine_handler.clone(),
            cache_manager.clone(),
            Default::default(),
        ));
        let rocksdb = Arc::new(RocksDBStorageEngine::new(
            cache_manager.clone(),
            rocksdb_engine_handler.clone(),
        ));
        let client = Arc::new(ClientConnectionManager::new(cache_manager.clone(), 8));
        Env {
            write_manager,
            cache_manager,
            memory,
            rocksdb,
            client,
            shard: segment_iden.shard_name,
        }
    }

    fn records(n: usize) -> Vec<AdapterWriteRecord> {
        (0..n)
            .map(|i| AdapterWriteRecord::new("s".to_string(), Bytes::from(format!("v{i}"))))
            .collect()
    }

    async fn write(e: &Env, acks: i8) -> Result<Vec<AdapterWriteRespRow>, StorageEngineError> {
        batch_write(
            &e.write_manager,
            &e.cache_manager,
            &e.memory,
            &e.rocksdb,
            &e.client,
            &e.shard,
            &records(3),
            acks,
        )
        .await
    }

    #[tokio::test]
    async fn acks_all_single_leader_commits_immediately() {
        let e = env().await;
        let rows = write(&e, -1).await.unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(
            e.cache_manager
                .get_offset_state(&e.shard)
                .unwrap()
                .high_watermark_offset,
            3
        );
    }

    #[tokio::test]
    async fn acks_all_times_out_when_follower_lags() {
        let e = env().await;
        if let Some(mut seg) = e.cache_manager.get_active_segment(&e.shard) {
            seg.isr = vec![1, 2];
            e.cache_manager.set_segment(&seg);
        }
        e.cache_manager.add_segment_replica(&e.shard, 0);
        let state = e.cache_manager.get_segment_replica(&e.shard, 0).unwrap();
        update_follower_progress(&state, 2, 1, 0, 0, 0);
        let err = tokio::time::timeout(Duration::from_secs(5), write(&e, -1))
            .await
            .unwrap();
        assert!(err.is_err());
    }
}
