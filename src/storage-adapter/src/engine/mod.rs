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
use async_trait::async_trait;
use common_base::error::common::CommonError;
use metadata_struct::adapter::adapter_offset::{AdapterOffsetStrategy, AdapterShardInfo};
use metadata_struct::adapter::adapter_read_config::{AdapterReadConfig, AdapterWriteRespRow};
use metadata_struct::adapter::adapter_record::AdapterWriteRecord;
use metadata_struct::adapter::adapter_shard::AdapterShardDetail;
use metadata_struct::storage::record::StorageRecord;
use std::collections::HashMap;
use std::sync::Arc;
use storage_engine::handler::adapter::StorageEngineHandler;
pub struct EngineStorageAdapter {
    adapter: Arc<StorageEngineHandler>,
}

impl EngineStorageAdapter {
    pub async fn new(adapter: Arc<StorageEngineHandler>) -> EngineStorageAdapter {
        EngineStorageAdapter { adapter }
    }
}

#[async_trait]
impl StorageAdapter for EngineStorageAdapter {
    async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), CommonError> {
        self.adapter.create_shard(shard).await
    }

    async fn list_shard(
        &self,
        shard: Option<String>,
    ) -> Result<Vec<AdapterShardDetail>, CommonError> {
        self.adapter.list_shard(shard).await
    }

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError> {
        self.adapter.delete_shard(shard).await
    }

    async fn write(
        &self,
        shard: &str,
        records: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, CommonError> {
        self.adapter.write(shard, records).await
    }

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError> {
        self.adapter
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
        self.adapter
            .read_by_tag(shard, tag, start_offset, read_config)
            .await
    }

    async fn read_by_keys(
        &self,
        shard: &str,
        keys: &[&str],
    ) -> Result<HashMap<String, Vec<StorageRecord>>, CommonError> {
        let mut result = HashMap::with_capacity(keys.len());
        for &key in keys {
            let records = self
                .adapter
                .read_by_key(shard, key)
                .await
                .map_err(|e| CommonError::CommonError(e.to_string()))?;
            result.insert(key.to_string(), records);
        }
        Ok(result)
    }

    async fn delete_by_keys(&self, shard: &str, keys: &[&str]) -> Result<(), CommonError> {
        self.adapter
            .delete_by_keys(shard, keys)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn delete_by_offsets(&self, shard: &str, offsets: &[u64]) -> Result<(), CommonError> {
        self.adapter
            .delete_by_offsets(shard, offsets)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, CommonError> {
        self.adapter
            .get_offset_by_timestamp(shard, timestamp, strategy)
            .await
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}
