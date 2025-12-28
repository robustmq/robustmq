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

use crate::storage::StorageAdapter;
use axum::async_trait;
use common_base::error::common::CommonError;
use metadata_struct::storage::adapter_offset::{
    AdapterMessageExpireConfig, AdapterReadShardOffset, AdapterShardInfo,
};
use metadata_struct::storage::adapter_read_config::{AdapterReadConfig, AdapterWriteRespRow};
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::storage_record::StorageRecord;
use std::collections::HashMap;
use std::sync::Arc;
use storage_engine::memory::engine::MemoryStorageEngine;

#[derive(Clone, Debug, Default)]
pub struct ShardState {
    pub earliest_offset: u64,
    pub latest_offset: u64,
}

#[derive(Clone)]
pub struct MemoryStorageAdapter {
    pub memory_storage_engine: Arc<MemoryStorageEngine>,
}

impl MemoryStorageAdapter {
    pub fn new(memory_storage_engine: Arc<MemoryStorageEngine>) -> Self {
        MemoryStorageAdapter {
            memory_storage_engine,
        }
    }
}

#[async_trait]
impl StorageAdapter for MemoryStorageAdapter {
    async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), CommonError> {
        self.memory_storage_engine.create_shard(shard).await
    }

    async fn list_shard(
        &self,
        shard: Option<String>,
    ) -> Result<Vec<AdapterShardInfo>, CommonError> {
        self.memory_storage_engine.list_shard(shard).await
    }

    async fn delete_shard(&self, shard_name: &str) -> Result<(), CommonError> {
        self.memory_storage_engine.delete_shard(shard_name).await
    }

    async fn batch_write(
        &self,
        shard: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, CommonError> {
        self.memory_storage_engine
            .batch_write(shard, messages)
            .await
    }

    async fn write(
        &self,
        shard: &str,
        data: &AdapterWriteRecord,
    ) -> Result<AdapterWriteRespRow, CommonError> {
        self.memory_storage_engine.write(shard, data).await
    }

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.memory_storage_engine
            .read_by_offset(shard, offset, read_config)
            .await
    }

    async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.memory_storage_engine
            .read_by_tag(shard, tag, start_offset, read_config)
            .await
    }

    async fn read_by_key(&self, shard: &str, key: &str) -> Result<Vec<StorageRecord>, CommonError> {
        self.memory_storage_engine.read_by_key(shard, key).await
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<AdapterReadShardOffset>, CommonError> {
        self.memory_storage_engine
            .get_offset_by_timestamp(shard, timestamp)
            .await
    }

    async fn get_offset_by_group(
        &self,
        group_name: &str,
    ) -> Result<Vec<AdapterReadShardOffset>, CommonError> {
        self.memory_storage_engine
            .get_offset_by_group(group_name)
            .await
    }

    async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        self.memory_storage_engine
            .commit_offset(group_name, offset)
            .await
    }

    async fn message_expire(
        &self,
        _config: &AdapterMessageExpireConfig,
    ) -> Result<(), CommonError> {
        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        driver::build_message_storage_driver,
        offset::OffsetManager,
        storage::ArcStorageAdapter,
        tests::{
            test_consumer_group_offset, test_shard_lifecycle,
            test_timestamp_index_with_multiple_entries, test_write_and_read,
        },
    };
    use common_config::storage::{
        memory::StorageDriverMemoryConfig, StorageAdapterConfig, StorageAdapterType,
    };
    use grpc_clients::pool::ClientPool;
    use rocksdb_engine::test::test_rocksdb_instance;
    use storage_engine::{
        memory::engine::MemoryStorageEngine, rocksdb::engine::RocksDBStorageEngine,
    };

    async fn build_adapter() -> ArcStorageAdapter {
        let rocksdb_engine_handler = test_rocksdb_instance();
        let client_pool = Arc::new(ClientPool::new(2));
        let offset_manager = Arc::new(OffsetManager::new(
            client_pool.clone(),
            rocksdb_engine_handler.clone(),
        ));
        let config = StorageAdapterConfig {
            storage_type: StorageAdapterType::Memory,
            memory_config: Some(StorageDriverMemoryConfig::default()),
            ..Default::default()
        };

        let memory_storage_engine = Arc::new(MemoryStorageEngine::new(
            StorageDriverMemoryConfig::default(),
        ));
        let rocksdb_storage_engine =
            Arc::new(RocksDBStorageEngine::new(rocksdb_engine_handler.clone()));
        build_message_storage_driver(
            offset_manager.clone(),
            memory_storage_engine,
            rocksdb_storage_engine.clone(),
            config,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_memory_shard_lifecycle() {
        let adapter = build_adapter().await;
        test_shard_lifecycle(adapter).await;
    }

    #[tokio::test]
    async fn test_memory_write_and_read() {
        let adapter = build_adapter().await;
        test_write_and_read(adapter).await;
    }

    #[tokio::test]
    async fn test_memory_consumer_group_offset() {
        let adapter = build_adapter().await;
        test_consumer_group_offset(adapter).await;
    }

    #[tokio::test]
    async fn test_memory_timestamp_index_with_multiple_entries() {
        let adapter = build_adapter().await;
        test_timestamp_index_with_multiple_entries(adapter).await;
    }
}
