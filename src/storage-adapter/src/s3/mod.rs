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
use crate::storage::{ShardInfo, ShardOffset, StorageAdapter};
use axum::async_trait;
use common_base::error::common::CommonError;
use common_config::storage::s3::StorageDriverS3Config;
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;
use std::collections::HashMap;

pub struct S3StorageAdapter {
    config: StorageDriverS3Config,
}

impl S3StorageAdapter {
    pub fn new(config: StorageDriverS3Config) -> Self {
        S3StorageAdapter { config }
    }
}

#[async_trait]
impl StorageAdapter for S3StorageAdapter {
    /// Create a new shard
    async fn create_shard(&self, _shard: &ShardInfo) -> Result<(), CommonError> {
        Err(CommonError::CommonError(
            "S3StorageAdapter::create_shard not implemented".to_string(),
        ))
    }

    /// List shards by namespace and shard name
    async fn list_shard(
        &self,
        _namespace: &str,
        _shard_name: &str,
    ) -> Result<Vec<ShardInfo>, CommonError> {
        Ok(Vec::new())
    }

    /// Delete a shard
    async fn delete_shard(&self, _namespace: &str, _shard_name: &str) -> Result<(), CommonError> {
        Err(CommonError::CommonError(
            "S3StorageAdapter::delete_shard not implemented".to_string(),
        ))
    }

    /// Write a single record to storage
    async fn write(
        &self,
        _namespace: &str,
        _shard_name: &str,
        _data: &Record,
    ) -> Result<u64, CommonError> {
        Err(CommonError::CommonError(
            "S3StorageAdapter::write not implemented".to_string(),
        ))
    }

    /// Batch write multiple records to storage
    async fn batch_write(
        &self,
        _namespace: &str,
        _shard_name: &str,
        _data: &[Record],
    ) -> Result<Vec<u64>, CommonError> {
        Err(CommonError::CommonError(
            "S3StorageAdapter::batch_write not implemented".to_string(),
        ))
    }

    /// Read records by offset
    async fn read_by_offset(
        &self,
        _namespace: &str,
        _shard_name: &str,
        _offset: u64,
        _read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        Ok(Vec::new())
    }

    /// Read records by tag
    async fn read_by_tag(
        &self,
        _namespace: &str,
        _shard_name: &str,
        _offset: u64,
        _tag: &str,
        _read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        Ok(Vec::new())
    }

    /// Read records by key
    async fn read_by_key(
        &self,
        _namespace: &str,
        _shard_name: &str,
        _offset: u64,
        _key: &str,
        _read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        Ok(Vec::new())
    }

    /// Get offset by timestamp
    async fn get_offset_by_timestamp(
        &self,
        _namespace: &str,
        _shard_name: &str,
        _timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        Ok(None)
    }

    /// Get offset by consumer group
    async fn get_offset_by_group(
        &self,
        _group_name: &str,
    ) -> Result<Vec<ShardOffset>, CommonError> {
        Ok(Vec::new())
    }

    /// Commit consumer group offset
    async fn commit_offset(
        &self,
        _group_name: &str,
        _namespace: &str,
        _offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        Ok(())
    }

    /// Handle message expiration
    async fn message_expire(&self, _config: &MessageExpireConfig) -> Result<(), CommonError> {
        Ok(())
    }

    /// Close the storage adapter
    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}
