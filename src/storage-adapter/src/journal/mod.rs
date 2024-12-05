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

use axum::async_trait;
use common_base::error::common::CommonError;
use journal_client::option::JournalClientOption;
use journal_client::JournalEngineClient;
use metadata_struct::adapter::record::Record;

use crate::storage::{ReadConfig, ShardConfig, StorageAdapter};

#[derive(Clone)]
pub struct JournalStorageAdapter {
    client: JournalEngineClient,
}

impl JournalStorageAdapter {
    pub fn new(addrs: Vec<String>) -> Self {
        let mut options = JournalClientOption::build();
        options.set_addrs(addrs);
        let client = JournalEngineClient::new(options);
        JournalStorageAdapter { client }
    }
}

#[async_trait]
impl StorageAdapter for JournalStorageAdapter {
    async fn create_shard(
        &self,
        namespace: String,
        shard_name: String,
        shard_config: ShardConfig,
    ) -> Result<(), CommonError> {
        if let Err(e) = self
            .client
            .create_shard(&namespace, &shard_name, shard_config.replica_num)
            .await
        {
            return Err(CommonError::CommonError(e.to_string()));
        }
        return Ok(());
    }

    async fn delete_shard(&self, namespace: String, shard_name: String) -> Result<(), CommonError> {
        if let Err(e) = self.client.delete_shard(&namespace, &shard_name).await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }

    async fn write(&self, _: String, _: String, _: Record) -> Result<usize, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn batch_write(
        &self,
        _: String,
        _: String,
        _: Vec<Record>,
    ) -> Result<Vec<usize>, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn read_by_offset(
        &self,
        _: String,
        _: String,
        _offset: u64,
        _: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn read_by_tag(
        &self,
        _: String,
        _: String,
        _tag: String,
        _: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn read_by_key(
        &self,
        _: String,
        _: String,
        _key: String,
        _: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn get_offset_by_timestamp(
        &self,
        _: String,
        _: String,
        _timestamp: u64,
    ) -> Result<u64, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn get_offset_by_group(
        &self,
        _group_name: String,
        _namespace: String,
        _shard_name: String,
    ) -> Result<u64, CommonError> {
        Ok(0)
    }

    async fn commit_offset(
        &self,
        _: String,
        _: String,
        _: String,
        _: u64,
    ) -> Result<(), CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn close(&self) -> Result<(), CommonError> {
        if let Err(e) = self.client.close().await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }
}
