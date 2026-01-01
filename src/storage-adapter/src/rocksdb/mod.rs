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
    AdapterConsumerGroupOffset, AdapterOffsetStrategy, AdapterReadShardInfo, AdapterShardInfo,
};
use metadata_struct::storage::adapter_read_config::{AdapterReadConfig, AdapterWriteRespRow};
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
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
    async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), CommonError> {
        self.rocksdb_storage_engine
            .create_shard(shard)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn list_shard(
        &self,
        shard: Option<String>,
    ) -> Result<Vec<AdapterReadShardInfo>, CommonError> {
        self.rocksdb_storage_engine
            .list_shard(shard)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError> {
        self.rocksdb_storage_engine
            .delete_shard(shard)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn write(
        &self,
        shard: &str,
        message: &AdapterWriteRecord,
    ) -> Result<AdapterWriteRespRow, CommonError> {
        self.rocksdb_storage_engine
            .write(shard, message)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn batch_write(
        &self,
        shard: &str,
        messages: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, CommonError> {
        self.rocksdb_storage_engine
            .batch_write(shard, messages)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.rocksdb_storage_engine
            .read_by_offset(shard, offset, read_config)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.rocksdb_storage_engine
            .read_by_tag(shard, tag, start_offset, read_config)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn read_by_key(&self, shard: &str, key: &str) -> Result<Vec<StorageRecord>, CommonError> {
        self.rocksdb_storage_engine
            .read_by_key(shard, key)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, CommonError> {
        self.rocksdb_storage_engine
            .get_offset_by_timestamp(shard, timestamp, strategy)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn get_offset_by_group(
        &self,
        group_name: &str,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, CommonError> {
        self.rocksdb_storage_engine
            .get_offset_by_group(group_name)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn commit_offset(
        &self,
        group_name: &str,
        offsets: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        self.rocksdb_storage_engine
            .commit_offset(group_name, offsets)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::{
        build_adapter, test_consumer_group_offset, test_shard_lifecycle,
        test_timestamp_index_with_multiple_entries, test_write_and_read,
    };
    use common_config::storage::StorageAdapterType;

    #[tokio::test]
    #[ignore = "reason"]
    async fn test_file_shard_lifecycle() {
        let adapter = build_adapter(StorageAdapterType::RocksDB).await;
        test_shard_lifecycle(adapter).await;
    }

    #[tokio::test]
    #[ignore = "reason"]
    async fn test_file_write_and_read() {
        let adapter = build_adapter(StorageAdapterType::RocksDB).await;
        test_write_and_read(adapter).await;
    }

    #[tokio::test]
    #[ignore = "reason"]
    async fn test_file_consumer_group_offset() {
        let adapter = build_adapter(StorageAdapterType::RocksDB).await;
        test_consumer_group_offset(adapter).await;
    }

    #[tokio::test]
    #[ignore = "reason"]
    async fn test_file_timestamp_index_with_multiple_entries() {
        let adapter = build_adapter(StorageAdapterType::RocksDB).await;
        test_timestamp_index_with_multiple_entries(adapter).await;
    }
}
