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
use crate::core::read_key::{read_by_key, ReadByKeyParams};
use crate::core::read_offset::{read_by_offset, ReadByOffsetParams};
use crate::core::read_tag::{read_by_tag, ReadByTagParams};
use crate::core::shard::get_shard_offset_by_timestamp;
use crate::{
    clients::manager::ClientConnectionManager,
    core::{
        cache::StorageCacheManager,
        shard::{create_shard_to_place, delete_shard_to_place},
        wirte::batch_write,
    },
    memory::engine::MemoryStorageEngine,
    rocksdb::engine::RocksDBStorageEngine,
    segment::write::WriteManager,
};
use common_base::error::common::CommonError;
use grpc_clients::pool::ClientPool;
use metadata_struct::storage::adapter_offset::{
    AdapterOffsetStrategy, AdapterReadShardOffset, AdapterShardInfo,
};
use metadata_struct::storage::adapter_read_config::{AdapterReadConfig, AdapterWriteRespRow};
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::shard::EngineType;
use metadata_struct::storage::storage_record::StorageRecord;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

#[derive(Clone)]
pub struct AdapterHandler {
    cache_manager: Arc<StorageCacheManager>,
    memory_storage_engine: Arc<MemoryStorageEngine>,
    rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
    client_connection_manager: Arc<ClientConnectionManager>,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    write_manager: Arc<WriteManager>,
    client_pool: Arc<ClientPool>,
}

impl AdapterHandler {
    pub fn new(
        cache_manager: Arc<StorageCacheManager>,
        client_pool: Arc<ClientPool>,
        memory_storage_engine: Arc<MemoryStorageEngine>,
        rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
        client_connection_manager: Arc<ClientConnectionManager>,
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        write_manager: Arc<WriteManager>,
    ) -> Self {
        AdapterHandler {
            cache_manager,
            client_pool,
            memory_storage_engine,
            rocksdb_storage_engine,
            rocksdb_engine_handler,
            client_connection_manager,
            write_manager,
        }
    }

    pub async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), CommonError> {
        if let Err(e) = create_shard_to_place(&self.cache_manager, &self.client_pool, shard).await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }

    pub async fn list_shard(
        &self,
        shard: Option<String>,
    ) -> Result<Vec<AdapterShardInfo>, CommonError> {
        if let Some(shard_name) = shard {
            if let Some(raw) = self.cache_manager.shards.get(&shard_name) {
                return Ok(vec![AdapterShardInfo {
                    shard_name: raw.shard_name.clone(),
                    replica_num: 1,
                }]);
            }
            return Ok(Vec::new());
        }

        let res = self
            .cache_manager
            .shards
            .iter()
            .map(|raw| AdapterShardInfo {
                shard_name: raw.shard_name.clone(),
                replica_num: 1,
            })
            .collect();

        Ok(res)
    }

    pub async fn delete_shard(&self, shard_name: &str) -> Result<(), CommonError> {
        if let Err(e) = delete_shard_to_place(&self.client_pool, shard_name).await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }

    pub async fn batch_write(
        &self,
        shard: &str,
        records: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, CommonError> {
        match batch_write(
            &self.write_manager,
            &self.cache_manager,
            &self.memory_storage_engine,
            &self.rocksdb_storage_engine,
            &self.client_connection_manager,
            shard,
            records,
        )
        .await
        {
            Ok(offsets) => Ok(offsets),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    pub async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        match read_by_offset(ReadByOffsetParams {
            rocksdb_engine_handler: self.rocksdb_engine_handler.clone(),
            cache_manager: self.cache_manager.clone(),
            memory_storage_engine: self.memory_storage_engine.clone(),
            rocksdb_storage_engine: self.rocksdb_storage_engine.clone(),
            client_connection_manager: self.client_connection_manager.clone(),
            shard_name: shard.to_string(),
            offset,
            read_config: read_config.clone(),
        })
        .await
        {
            Ok(data) => Ok(data),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    pub async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        match read_by_tag(ReadByTagParams {
            rocksdb_engine_handler: self.rocksdb_engine_handler.clone(),
            cache_manager: self.cache_manager.clone(),
            memory_storage_engine: self.memory_storage_engine.clone(),
            rocksdb_storage_engine: self.rocksdb_storage_engine.clone(),
            client_connection_manager: self.client_connection_manager.clone(),
            shard_name: shard.to_string(),
            tag: tag.to_string(),
            start_offset,
            read_config: read_config.clone(),
        })
        .await
        {
            Ok(data) => Ok(data),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    pub async fn read_by_key(
        &self,
        shard: &str,
        key: &str,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        match read_by_key(ReadByKeyParams {
            rocksdb_engine_handler: self.rocksdb_engine_handler.clone(),
            cache_manager: self.cache_manager.clone(),
            memory_storage_engine: self.memory_storage_engine.clone(),
            rocksdb_storage_engine: self.rocksdb_storage_engine.clone(),
            client_connection_manager: self.client_connection_manager.clone(),
            shard_name: shard.to_string(),
            key: key.to_string(),
        })
        .await
        {
            Ok(data) => Ok(data),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    pub async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<Option<AdapterReadShardOffset>, CommonError> {
        match self
            .get_offset_by_timestamp0(shard, timestamp, strategy)
            .await
        {
            Ok(data) => Ok(data),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    async fn get_offset_by_timestamp0(
        &self,
        shard_name: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<Option<AdapterReadShardOffset>, StorageEngineError> {
        let Some(shard) = self.cache_manager.shards.get(shard_name) else {
            return Err(StorageEngineError::ShardNotExist(shard_name.to_owned()));
        };
        let result = match shard.engine_type {
            EngineType::Memory => {
                self.memory_storage_engine
                    .get_offset_by_timestamp(shard_name, timestamp, strategy)
                    .await?
            }
            EngineType::RocksDB => {
                self.rocksdb_storage_engine
                    .get_offset_by_timestamp(shard_name, timestamp, strategy)
                    .await?
            }
            EngineType::Segment => Some(get_shard_offset_by_timestamp(
                &self.cache_manager,
                &self.rocksdb_engine_handler,
                shard_name,
                timestamp,
                strategy,
            )?),
        };
        Ok(result)
    }
}
