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
use metadata_struct::adapter::MessageExpireConfig;
use metadata_struct::adapter::{read_config::ReadConfig, adapter_record::AdapterWriteRecord};
use metadata_struct::adapter::{ShardInfo, ShardOffset};
use metadata_struct::storage::storage_record::StorageRecord;
use std::{collections::HashMap, sync::Arc};
use storage_engine::rocksdb::engine::RocksDBStorageEngine;

#[derive(Clone)]
pub struct RocksDBStorageAdapter {
    pub rocksdb_storage_engine: Arc<RocksDBStorageEngine>,
}

impl RocksDBStorageAdapter {
    pub fn new(rocksdb_storage_engine: Arc<RocksDBStorageEngine>) -> Self {
        RocksDBStorageAdapter {
            rocksdb_storage_engine,
        }
    }
}

#[async_trait]
impl StorageAdapter for RocksDBStorageAdapter {
    async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
        self.rocksdb_storage_engine.create_shard(shard).await
    }

    async fn list_shard(&self, shard: Option<String>) -> Result<Vec<ShardInfo>, CommonError> {
        self.rocksdb_storage_engine.list_shard(shard).await
    }

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError> {
        self.rocksdb_storage_engine.delete_shard(shard).await
    }

    async fn write(&self, shard: &str, message: &AdapterWriteRecord) -> Result<u64, CommonError> {
        self.rocksdb_storage_engine.write(shard, message).await
    }

    async fn batch_write(
        &self,
        shard: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<u64>, CommonError> {
        self.rocksdb_storage_engine
            .batch_write(shard, messages)
            .await
    }

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.rocksdb_storage_engine
            .read_by_offset(shard, offset, read_config)
            .await
    }

    async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &ReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.rocksdb_storage_engine
            .read_by_tag(shard, tag, start_offset, read_config)
            .await
    }

    async fn read_by_key(
        &self,
        shard: &str,
        key: &str,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.rocksdb_storage_engine.read_by_key(shard, key).await
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        self.rocksdb_storage_engine
            .get_offset_by_timestamp(shard, timestamp)
            .await
    }

    async fn get_offset_by_group(&self, group_name: &str) -> Result<Vec<ShardOffset>, CommonError> {
        self.rocksdb_storage_engine
            .get_offset_by_group(group_name)
            .await
    }

    async fn commit_offset(
        &self,
        group_name: &str,
        offsets: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        self.rocksdb_storage_engine
            .commit_offset(group_name, offsets)
            .await
    }

    async fn message_expire(&self, _config: &MessageExpireConfig) -> Result<(), CommonError> {
        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
        memory::StorageDriverMemoryConfig, rocksdb::StorageDriverRocksDBConfig,
        StorageAdapterConfig, StorageAdapterType,
    };
    use grpc_clients::pool::ClientPool;
    use rocksdb_engine::test::test_rocksdb_instance;
    use std::sync::Arc;
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
            storage_type: StorageAdapterType::RocksDB,
            rocksdb_config: Some(StorageDriverRocksDBConfig::default()),
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
    async fn test_file_shard_lifecycle() {
        let adapter = build_adapter().await;
        test_shard_lifecycle(adapter).await;
    }

    #[tokio::test]
    async fn test_file_write_and_read() {
        let adapter = build_adapter().await;
        test_write_and_read(adapter).await;
    }

    #[tokio::test]
    async fn test_file_consumer_group_offset() {
        let adapter = build_adapter().await;
        test_consumer_group_offset(adapter).await;
    }

    #[tokio::test]
    async fn test_file_timestamp_index_with_multiple_entries() {
        let adapter = build_adapter().await;
        test_timestamp_index_with_multiple_entries(adapter).await;
    }
}
