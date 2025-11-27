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

use crate::expire::MessageExpireConfig;
use crate::memory::MemoryStorageAdapter;
use axum::async_trait;
use common_base::error::common::CommonError;
use common_config::storage::memory::StorageDriverMemoryConfig;
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};

pub type ArcStorageAdapter = Arc<dyn StorageAdapter + Send + Sync>;

pub enum OffsetStrategy {
    Earliest,
    Latest,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    pub shard_name: String,
    pub replica_num: u32,
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct ShardOffset {
    pub group: String,
    pub shard_name: String,
    pub segment_no: u32,
    pub offset: u64,
}

#[async_trait]
pub trait StorageAdapter {
    async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError>;

    async fn list_shard(&self, shard: &str) -> Result<Vec<ShardInfo>, CommonError>;

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError>;

    async fn write(&self, shard: &str, data: &Record) -> Result<u64, CommonError>;

    async fn batch_write(&self, shard: &str, data: &[Record]) -> Result<Vec<u64>, CommonError>;

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError>;

    async fn read_by_tag(
        &self,
        shard: &str,
        offset: u64,
        tag: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError>;

    async fn read_by_key(
        &self,
        shard: &str,
        offset: u64,
        key: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError>;

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError>;

    async fn get_offset_by_group(&self, group_name: &str) -> Result<Vec<ShardOffset>, CommonError>;

    async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError>;

    async fn message_expire(&self, config: &MessageExpireConfig) -> Result<(), CommonError>;

    async fn close(&self) -> Result<(), CommonError>;
}

pub fn build_memory_storage_driver() -> ArcStorageAdapter {
    Arc::new(MemoryStorageAdapter::new(
        StorageDriverMemoryConfig::default(),
    ))
}
