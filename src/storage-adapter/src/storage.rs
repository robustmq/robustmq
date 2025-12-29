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

use crate::memory::MemoryStorageAdapter;
use axum::async_trait;
use common_base::error::common::CommonError;
use common_config::storage::memory::StorageDriverMemoryConfig;
use metadata_struct::storage::adapter_offset::{
    AdapterConsumerGroupOffset, AdapterMessageExpireConfig, AdapterOffsetStrategy, AdapterShardInfo,
};
use metadata_struct::storage::adapter_read_config::{AdapterReadConfig, AdapterWriteRespRow};
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use metadata_struct::storage::storage_record::StorageRecord;
use std::{collections::HashMap, sync::Arc};
use storage_engine::memory::engine::MemoryStorageEngine;

pub type ArcStorageAdapter = Arc<dyn StorageAdapter + Send + Sync>;

#[async_trait]
pub trait StorageAdapter {
    async fn create_shard(&self, shard: &AdapterShardInfo) -> Result<(), CommonError>;

    async fn list_shard(&self, shard: Option<String>)
        -> Result<Vec<AdapterShardInfo>, CommonError>;

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError>;

    async fn write(
        &self,
        shard: &str,
        data: &AdapterWriteRecord,
    ) -> Result<AdapterWriteRespRow, CommonError>;

    async fn batch_write(
        &self,
        shard: &str,
        data: &[AdapterWriteRecord],
    ) -> Result<Vec<AdapterWriteRespRow>, CommonError>;

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError>;

    async fn read_by_tag(
        &self,
        shard: &str,
        tag: &str,
        start_offset: Option<u64>,
        read_config: &AdapterReadConfig,
    ) -> Result<Vec<StorageRecord>, CommonError>;

    async fn read_by_key(&self, shard: &str, key: &str) -> Result<Vec<StorageRecord>, CommonError>;

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<Option<AdapterConsumerGroupOffset>, CommonError>;

    async fn get_offset_by_group(
        &self,
        group_name: &str,
        strategy: AdapterOffsetStrategy,
    ) -> Result<Vec<AdapterConsumerGroupOffset>, CommonError>;

    async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError>;

    async fn message_expire(&self, config: &AdapterMessageExpireConfig) -> Result<(), CommonError>;

    async fn close(&self) -> Result<(), CommonError>;
}

pub fn build_memory_storage_driver() -> ArcStorageAdapter {
    let memory_storage_engine = Arc::new(MemoryStorageEngine::new(
        StorageDriverMemoryConfig::default(),
    ));
    Arc::new(MemoryStorageAdapter::new(memory_storage_engine))
}
