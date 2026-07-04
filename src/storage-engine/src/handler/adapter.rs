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

use crate::core::error::StorageEngineError;
use crate::core::offset::ShardOffset;
use crate::core::read_key::{read_by_key, ReadByKeyParams};
use crate::core::read_offset::{read_by_offset, ReadByOffsetParams};
use crate::core::read_tag::{read_by_tag, ReadByTagParams};
use crate::{
    clients::manager::ClientConnectionManager,
    commitlog::memory::engine::MemoryStorageEngine,
    commitlog::rocksdb::engine::RocksDBStorageEngine,
    core::{
        cache::StorageCacheManager,
        shard::{create_shard_to_place, delete_shard_to_place},
        write::batch_write,
    },
    filesegment::write_manager::WriteManager,
};
use common_base::error::common::CommonError;
use common_config::broker::broker_config;
use common_config::storage::StorageType;
use common_metrics::storage_engine::{
    record_storage_engine_ops, record_storage_engine_ops_duration, record_storage_engine_ops_fail,
};
use grpc_clients::pool::ClientPool;
use metadata_struct::adapter::adapter_offset::{AdapterOffsetStrategy, AdapterShardInfo};
use metadata_struct::adapter::adapter_read_config::{AdapterReadConfig, AdapterWriteRespRow};
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::adapter::adapter_shard::{AdapterShardDetail, AdapterShardDetailOffset};
use metadata_struct::storage::record::StorageRecord;
use metadata_struct::storage::shard::EngineShard;
use protocol::storage::protocol::{DeleteReqBody, ShardOffsetReqBody, ShardOffsetRespBody};
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

pub struct StorageEngineHandlerParams {
    pub cache_manager: Arc<StorageCacheManager>,
    pub client_pool: Arc<ClientPool>,
    pub memory_storage_engine: Arc<MemoryStorageEngine>,
    pub rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
    pub client_connection_manager: Arc<ClientConnectionManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub write_manager: Arc<WriteManager>,
}

#[derive(Clone)]
pub struct StorageEngineHandler {
    pub cache_manager: Arc<StorageCacheManager>,
    pub memory_storage_engine: Arc<MemoryStorageEngine>,
    pub rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
    pub client_connection_manager: Arc<ClientConnectionManager>,
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub write_manager: Arc<WriteManager>,
    pub client_pool: Arc<ClientPool>,
}

impl StorageEngineHandler {
    pub fn new(params: StorageEngineHandlerParams) -> Self {
        StorageEngineHandler {
            cache_manager: params.cache_manager,
            client_pool: params.client_pool,
            memory_storage_engine: params.memory_storage_engine,
            rocksdb_storage_engine: params.rocksdb_storage_engine,
            rocksdb_engine_handler: params.rocksdb_engine_handler,
            client_connection_manager: params.client_connection_manager,
            write_manager: params.write_manager,
        }
    }

    pub async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), CommonError> {
        let start = std::time::Instant::now();
        let result = create_shard_to_place(&self.cache_manager, &self.client_pool, shard).await;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("create_shard");
        record_storage_engine_ops_duration("create_shard", duration_ms);
        if let Err(e) = result {
            record_storage_engine_ops_fail("create_shard");
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }

    /// Query a shard's offsets from its leader (used when this node is not the
    /// leader and therefore has no local copy of the shard's offset state).
    async fn shard_offset_remote(
        &self,
        leader_id: u64,
        shard_name: &str,
        by_timestamp: bool,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<ShardOffsetRespBody, StorageEngineError> {
        let body = ShardOffsetReqBody {
            shard_name: shard_name.to_string(),
            by_timestamp,
            timestamp,
            strategy: strategy as u8,
        };
        let resp = self
            .client_connection_manager
            .send_shard_offset(leader_id, body)
            .await?;
        if resp.error_code != 0 {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "Leader {leader_id} failed to resolve offsets for shard {shard_name} (error_code={})",
                resp.error_code
            )));
        }
        Ok(resp)
    }

    pub async fn list_shard(
        &self,
        shard: Option<String>,
    ) -> Result<Vec<AdapterShardDetail>, CommonError> {
        let shards: Vec<EngineShard> = if let Some(shard_name) = shard {
            self.cache_manager
                .shards
                .get(&shard_name)
                .map(|r| vec![r.clone()])
                .unwrap_or_default()
        } else {
            self.cache_manager
                .shards
                .iter()
                .map(|r| r.clone())
                .collect()
        };

        let mut results = Vec::with_capacity(shards.len());
        let local_broker_id = broker_config().broker_id;
        for shard in shards {
            let leader = self
                .cache_manager
                .get_active_segment(&shard.shard_name)
                .ok_or_else(|| {
                    CommonError::CommonError(format!(
                        "No active segment for shard {}",
                        shard.shard_name
                    ))
                })?
                .leader;

            let (start_offset, end_offset, high_watermark) = if leader != local_broker_id {
                let body = self
                    .shard_offset_remote(
                        leader,
                        &shard.shard_name,
                        false,
                        0,
                        AdapterOffsetStrategy::Earliest,
                    )
                    .await
                    .map_err(|e| CommonError::CommonError(e.to_string()))?;
                (body.start_offset, body.end_offset, body.high_watermark)
            } else {
                let offsets = ShardOffset::new(
                    self.cache_manager.clone(),
                    self.rocksdb_engine_handler.clone(),
                )
                .get_shard_offsets(&shard.shard_name)
                .map_err(|e| CommonError::CommonError(e.to_string()))?;
                (
                    offsets.earliest_offset,
                    offsets.latest_offset.saturating_sub(1),
                    offsets.high_watermark_offset,
                )
            };

            results.push(AdapterShardDetail {
                shard_name: shard.shard_name.clone(),
                topic_name: shard.topic_name.clone(),
                config: shard.config.clone(),
                desc: shard.desc.clone(),
                shard,
                offset: AdapterShardDetailOffset {
                    start_offset,
                    end_offset,
                    high_watermark,
                },
            });
        }
        Ok(results)
    }

    pub async fn delete_shard(&self, shard_name: &str) -> Result<(), CommonError> {
        let start = std::time::Instant::now();
        let result = delete_shard_to_place(&self.client_pool, shard_name).await;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("delete_shard");
        record_storage_engine_ops_duration("delete_shard", duration_ms);
        if let Err(e) = result {
            record_storage_engine_ops_fail("delete_shard");
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }

    pub async fn write(
        &self,
        shard: &str,
        records: &[AdapterWriteRecord],
        acks: i8,
    ) -> Result<Vec<AdapterWriteRespRow>, CommonError> {
        let start = std::time::Instant::now();
        let result = batch_write(
            &self.write_manager,
            &self.cache_manager,
            &self.memory_storage_engine,
            &self.rocksdb_storage_engine,
            &self.client_connection_manager,
            shard,
            records,
            acks,
            0,
        )
        .await;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("write");
        record_storage_engine_ops_duration("write", duration_ms);
        match result {
            Ok(offsets) => Ok(offsets),
            Err(e) => {
                record_storage_engine_ops_fail("write");
                Err(CommonError::CommonError(e.to_string()))
            }
        }
    }

    pub async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        let start = std::time::Instant::now();
        let result = read_by_offset(ReadByOffsetParams {
            rocksdb_engine_handler: self.rocksdb_engine_handler.clone(),
            cache_manager: self.cache_manager.clone(),
            memory_storage_engine: self.memory_storage_engine.clone(),
            rocksdb_storage_engine: self.rocksdb_storage_engine.clone(),
            client_connection_manager: self.client_connection_manager.clone(),
            shard_name: shard.to_string(),
            offset,
            read_config: read_config.clone(),
            single_segment: false,
        })
        .await;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("read_offset");
        record_storage_engine_ops_duration("read_offset", duration_ms);
        match result {
            Ok(data) => Ok(data),
            Err(e) => {
                record_storage_engine_ops_fail("read_offset");
                Err(CommonError::CommonError(e.to_string()))
            }
        }
    }

    pub async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        let start = std::time::Instant::now();
        let result = read_by_tag(ReadByTagParams {
            rocksdb_engine_handler: self.rocksdb_engine_handler.clone(),
            cache_manager: self.cache_manager.clone(),
            memory_storage_engine: self.memory_storage_engine.clone(),
            rocksdb_storage_engine: self.rocksdb_storage_engine.clone(),
            client_connection_manager: self.client_connection_manager.clone(),
            shard_name: shard.to_string(),
            tag: tag.to_string(),
            start_offset,
            batch_call_source: false,
            read_config: read_config.clone(),
        })
        .await;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("read_tag");
        record_storage_engine_ops_duration("read_tag", duration_ms);
        match result {
            Ok(data) => Ok(data),
            Err(e) => {
                record_storage_engine_ops_fail("read_tag");
                Err(CommonError::CommonError(e.to_string()))
            }
        }
    }

    pub async fn read_by_key(
        &self,
        shard: &str,
        key: &[u8],
    ) -> Result<Vec<StorageRecord>, CommonError> {
        let start = std::time::Instant::now();
        let result = read_by_key(ReadByKeyParams {
            rocksdb_engine_handler: self.rocksdb_engine_handler.clone(),
            cache_manager: self.cache_manager.clone(),
            memory_storage_engine: self.memory_storage_engine.clone(),
            rocksdb_storage_engine: self.rocksdb_storage_engine.clone(),
            client_connection_manager: self.client_connection_manager.clone(),
            shard_name: shard.to_string(),
            batch_call_source: false,
            key: bytes::Bytes::copy_from_slice(key),
        })
        .await;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("read_key");
        record_storage_engine_ops_duration("read_key", duration_ms);
        match result {
            Ok(data) => Ok(data),
            Err(e) => {
                record_storage_engine_ops_fail("read_key");
                Err(CommonError::CommonError(e.to_string()))
            }
        }
    }

    pub async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, CommonError> {
        let start = std::time::Instant::now();
        let result = self
            .get_offset_by_timestamp0(shard, timestamp, strategy)
            .await;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("get_offset_by_timestamp");
        record_storage_engine_ops_duration("get_offset_by_timestamp", duration_ms);
        match result {
            Ok(offset) => Ok(offset),
            Err(e) => {
                record_storage_engine_ops_fail("get_offset_by_timestamp");
                Err(CommonError::CommonError(e.to_string()))
            }
        }
    }

    pub async fn delete_by_key(
        &self,
        shard_name: &str,
        key: &[u8],
    ) -> Result<(), StorageEngineError> {
        self.delete_by_keys(shard_name, &[key]).await
    }

    pub async fn delete_by_keys(
        &self,
        shard_name: &str,
        keys: &[&[u8]],
    ) -> Result<(), StorageEngineError> {
        let start = std::time::Instant::now();
        let result = self.delete_by_keys_inner(shard_name, keys).await;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("delete_by_key");
        record_storage_engine_ops_duration("delete_by_key", duration_ms);
        if result.is_err() {
            record_storage_engine_ops_fail("delete_by_key");
        }
        result
    }

    async fn delete_by_keys_inner(
        &self,
        shard_name: &str,
        keys: &[&[u8]],
    ) -> Result<(), StorageEngineError> {
        if keys.is_empty() {
            return Ok(());
        }
        let Some(shard) = self.cache_manager.shards.get(shard_name) else {
            return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
        };

        // For memory/rocksdb shards the data lives on the segment leader. If this node
        // is not the leader, forward the delete; otherwise it is a no-op on local-empty state.
        if matches!(
            shard.config.storage_type,
            StorageType::EngineMemory | StorageType::EngineRocksDB
        ) {
            if let Some(leader) = self
                .cache_manager
                .get_active_segment(shard_name)
                .map(|s| s.leader)
            {
                if leader != broker_config().broker_id {
                    let body = DeleteReqBody {
                        shard_name: shard_name.to_string(),
                        keys: keys
                            .iter()
                            .map(|k| bytes::Bytes::copy_from_slice(k))
                            .collect(),
                        offsets: Vec::new(),
                        delete_before_offset: None,
                    };
                    let resp = self
                        .client_connection_manager
                        .send_delete(leader, body)
                        .await?;
                    if resp.error_code != 0 {
                        return Err(StorageEngineError::CommonErrorStr(format!(
                            "Leader {leader} failed to delete keys for shard {shard_name} (error_code={})",
                            resp.error_code
                        )));
                    }
                    return Ok(());
                }
            }
        }

        match shard.config.storage_type {
            StorageType::EngineMemory => {
                for key in keys {
                    self.memory_storage_engine
                        .delete_by_key(shard_name, key)
                        .await?;
                }
            }

            StorageType::EngineRocksDB => {
                self.rocksdb_storage_engine
                    .delete_by_keys(shard_name, keys)
                    .await?;
            }

            StorageType::EngineSegment => {
                return Err(StorageEngineError::CommonErrorStr(
                    "delete_by_key operation is not supported".to_string(),
                ));
            }

            _ => {
                return Err(StorageEngineError::CommonErrorStr(format!(
                    "Unsupported storage type {:?} for shard {} when delete by key",
                    shard.config.storage_type, shard_name
                )))
            }
        }
        Ok(())
    }

    pub async fn delete_by_offset(
        &self,
        shard_name: &str,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        self.delete_by_offsets(shard_name, &[offset]).await
    }

    pub async fn delete_by_offsets(
        &self,
        shard_name: &str,
        offsets: &[u64],
    ) -> Result<(), StorageEngineError> {
        let start = std::time::Instant::now();
        let result = self.delete_by_offsets_inner(shard_name, offsets).await;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("delete_by_offset");
        record_storage_engine_ops_duration("delete_by_offset", duration_ms);
        if result.is_err() {
            record_storage_engine_ops_fail("delete_by_offset");
        }
        result
    }

    async fn delete_by_offsets_inner(
        &self,
        shard_name: &str,
        offsets: &[u64],
    ) -> Result<(), StorageEngineError> {
        if offsets.is_empty() {
            return Ok(());
        }
        let Some(shard) = self.cache_manager.shards.get(shard_name) else {
            return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
        };

        if matches!(
            shard.config.storage_type,
            StorageType::EngineMemory | StorageType::EngineRocksDB
        ) {
            if let Some(leader) = self
                .cache_manager
                .get_active_segment(shard_name)
                .map(|s| s.leader)
            {
                if leader != broker_config().broker_id {
                    let body = DeleteReqBody {
                        shard_name: shard_name.to_string(),
                        keys: Vec::new(),
                        offsets: offsets.to_vec(),
                        delete_before_offset: None,
                    };
                    let resp = self
                        .client_connection_manager
                        .send_delete(leader, body)
                        .await?;
                    if resp.error_code != 0 {
                        return Err(StorageEngineError::CommonErrorStr(format!(
                            "Leader {leader} failed to delete offsets for shard {shard_name} (error_code={})",
                            resp.error_code
                        )));
                    }
                    return Ok(());
                }
            }
        }

        match shard.config.storage_type {
            StorageType::EngineMemory => {
                for &offset in offsets {
                    self.memory_storage_engine
                        .delete_by_offset(shard_name, offset)
                        .await?;
                }
            }

            StorageType::EngineRocksDB => {
                self.rocksdb_storage_engine
                    .delete_by_offsets(shard_name, offsets)
                    .await?;
            }

            StorageType::EngineSegment => {
                return Err(StorageEngineError::CommonErrorStr(
                    "delete_by_offset operation is not supported".to_string(),
                ));
            }

            _ => {
                return Err(StorageEngineError::CommonErrorStr(format!(
                    "Unsupported storage type {:?} for shard {} when delete by offset",
                    shard.config.storage_type, shard_name
                )))
            }
        }
        Ok(())
    }

    /// Delete all records with offset < `target_offset` (Kafka DeleteRecords
    /// semantics). Returns the achieved low_watermark.
    pub async fn delete_records_before(
        &self,
        shard_name: &str,
        target_offset: u64,
    ) -> Result<u64, StorageEngineError> {
        let start = std::time::Instant::now();
        let result = self
            .delete_records_before_inner(shard_name, target_offset)
            .await;
        let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
        record_storage_engine_ops("delete_records_before");
        record_storage_engine_ops_duration("delete_records_before", duration_ms);
        if result.is_err() {
            record_storage_engine_ops_fail("delete_records_before");
        }
        result
    }

    async fn delete_records_before_inner(
        &self,
        shard_name: &str,
        target_offset: u64,
    ) -> Result<u64, StorageEngineError> {
        if !self.cache_manager.shards.contains_key(shard_name) {
            return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
        }

        // The shard's data (memory/rocksdb records, or segment files) only
        // lives on the active segment's leader; forward there otherwise.
        if let Some(leader) = self
            .cache_manager
            .get_active_segment(shard_name)
            .map(|s| s.leader)
        {
            if leader != broker_config().broker_id {
                let body = DeleteReqBody {
                    shard_name: shard_name.to_string(),
                    keys: Vec::new(),
                    offsets: Vec::new(),
                    delete_before_offset: Some(target_offset),
                };
                let resp = self
                    .client_connection_manager
                    .send_delete(leader, body)
                    .await?;
                if resp.error_code != 0 {
                    return Err(StorageEngineError::CommonErrorStr(format!(
                        "Leader {leader} failed to delete records before offset for shard {shard_name} (error_code={})",
                        resp.error_code
                    )));
                }
                return Ok(resp.achieved_offset);
            }
        }

        crate::handler::data::delete_records_before_req(
            &self.cache_manager,
            &self.rocksdb_engine_handler,
            &self.memory_storage_engine,
            &self.rocksdb_storage_engine,
            shard_name,
            target_offset,
        )
        .await
    }

    async fn get_offset_by_timestamp0(
        &self,
        shard_name: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, StorageEngineError> {
        let Some(shard) = self.cache_manager.shards.get(shard_name) else {
            return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
        };

        // For memory/rocksdb shards, offsets live only on the segment leader. If
        // this node is not the leader, ask the leader to resolve the timestamp.
        if matches!(
            shard.config.storage_type,
            StorageType::EngineMemory | StorageType::EngineRocksDB
        ) {
            if let Some(leader) = self
                .cache_manager
                .get_active_segment(shard_name)
                .map(|s| s.leader)
            {
                if leader != broker_config().broker_id {
                    let body = self
                        .shard_offset_remote(leader, shard_name, true, timestamp, strategy.clone())
                        .await?;
                    return Ok(body.offset);
                }
            }
        }

        let result = match shard.config.storage_type {
            StorageType::EngineMemory => {
                self.memory_storage_engine
                    .get_offset_by_timestamp(shard_name, timestamp, strategy)
                    .await?
            }

            StorageType::EngineRocksDB => {
                self.rocksdb_storage_engine
                    .get_offset_by_timestamp(shard_name, timestamp, strategy)
                    .await?
            }

            StorageType::EngineSegment => {
                use crate::filesegment::index::read::get_in_segment_by_timestamp;
                use crate::filesegment::SegmentIdentity;

                // Find which segment owns this timestamp; route to that segment's leader.
                let target_segment =
                    get_in_segment_by_timestamp(&self.cache_manager, shard_name, timestamp as i64)?;

                if let Some(seg_seq) = target_segment {
                    let seg_iden = SegmentIdentity::new(shard_name, seg_seq);
                    if let Some(seg) = self.cache_manager.get_segment(&seg_iden) {
                        if seg.leader != broker_config().broker_id {
                            let body = self
                                .shard_offset_remote(
                                    seg.leader,
                                    shard_name,
                                    true,
                                    timestamp,
                                    strategy.clone(),
                                )
                                .await?;
                            return Ok(body.offset);
                        }
                    }
                }

                crate::filesegment::read::get_segment_offset_by_timestamp(
                    &self.cache_manager,
                    &self.rocksdb_engine_handler,
                    shard_name,
                    timestamp,
                    strategy,
                )?
            }

            _ => {
                return Err(StorageEngineError::CommonErrorStr(format!(
                    "Unsupported storage type {:?} for shard {} when getting offset by timestamp",
                    shard.config.storage_type, shard_name
                )))
            }
        };

        Ok(result)
    }
}
