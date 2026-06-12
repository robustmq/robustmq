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

use crate::commitlog::memory::engine::MemoryStorageEngine;
use crate::commitlog::rocksdb::engine::RocksDBStorageEngine;
use crate::core::cache::StorageCacheManager;
use crate::core::error::StorageEngineError;
use crate::isr::log::ReplicaLog;
use async_trait::async_trait;
use common_config::storage::StorageType;
use metadata_struct::storage::record::StorageRecord;
use std::sync::Arc;

#[derive(Clone)]
pub struct EngineReplicaLog {
    memory: Arc<MemoryStorageEngine>,
    rocksdb: Arc<RocksDBStorageEngine>,
    cache_manager: Arc<StorageCacheManager>,
}

impl EngineReplicaLog {
    pub fn new(
        memory: Arc<MemoryStorageEngine>,
        rocksdb: Arc<RocksDBStorageEngine>,
        cache_manager: Arc<StorageCacheManager>,
    ) -> Self {
        EngineReplicaLog {
            memory,
            rocksdb,
            cache_manager,
        }
    }

    fn is_rocksdb(&self, shard: &str) -> Result<bool, StorageEngineError> {
        let shard_state = self
            .cache_manager
            .shards
            .get(shard)
            .ok_or_else(|| StorageEngineError::ShardNotExist(shard.to_string()))?;
        Ok(shard_state.config.storage_type == StorageType::EngineRocksDB)
    }
}

#[async_trait]
impl ReplicaLog for EngineReplicaLog {
    async fn append_at(
        &self,
        shard: &str,
        segment_seq: u32,
        base_offset: u64,
        records: Vec<StorageRecord>,
    ) -> Result<(), StorageEngineError> {
        if self.is_rocksdb(shard)? {
            self.rocksdb
                .append_at(shard, segment_seq, base_offset, records)
                .await
        } else {
            self.memory
                .append_at(shard, segment_seq, base_offset, records)
                .await
        }
    }

    async fn read_from(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
        max_bytes: u64,
    ) -> Result<Vec<StorageRecord>, StorageEngineError> {
        if self.is_rocksdb(shard)? {
            self.rocksdb
                .read_from(shard, segment_seq, offset, max_bytes)
                .await
        } else {
            self.memory
                .read_from(shard, segment_seq, offset, max_bytes)
                .await
        }
    }

    fn latest_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError> {
        if self.is_rocksdb(shard)? {
            self.rocksdb.latest_offset(shard, segment_seq)
        } else {
            self.memory.latest_offset(shard, segment_seq)
        }
    }

    async fn truncate_to(
        &self,
        shard: &str,
        segment_seq: u32,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        if self.is_rocksdb(shard)? {
            self.rocksdb.truncate_to(shard, segment_seq, offset).await
        } else {
            self.memory.truncate_to(shard, segment_seq, offset).await
        }
    }

    async fn clear(&self, shard: &str, segment_seq: u32) -> Result<(), StorageEngineError> {
        if self.is_rocksdb(shard)? {
            self.rocksdb.clear(shard, segment_seq).await
        } else {
            self.memory.clear(shard, segment_seq).await
        }
    }

    fn log_start_offset(&self, shard: &str, segment_seq: u32) -> Result<u64, StorageEngineError> {
        if self.is_rocksdb(shard)? {
            self.rocksdb.log_start_offset(shard, segment_seq)
        } else {
            self.memory.log_start_offset(shard, segment_seq)
        }
    }
}
