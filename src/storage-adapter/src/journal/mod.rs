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

use crate::storage::{ShardConfig, StorageAdapter};

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
    async fn create_shard(&self, _: String, _: ShardConfig) -> Result<(), CommonError> {
        // self.client.create_shard(namespace, shard_name, replica_num);
        self.client.close().await;
        return Ok(());
    }

    async fn delete_shard(&self, _: String) -> Result<(), CommonError> {
        return Ok(());
    }

    async fn stream_write(&self, _: String, _: Vec<Record>) -> Result<Vec<usize>, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read(
        &self,
        _: String,
        _: String,
        _: Option<u128>,
        _: Option<usize>,
    ) -> Result<Option<Vec<Record>>, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_commit_offset(
        &self,
        _: String,
        _: String,
        _: u128,
    ) -> Result<bool, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read_by_offset(
        &self,
        _: String,
        _: usize,
    ) -> Result<Option<Record>, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read_by_timestamp(
        &self,
        _: String,
        _: u128,
        _: u128,
        _: Option<usize>,
        _: Option<usize>,
    ) -> Result<Option<Vec<Record>>, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }

    async fn stream_read_by_key(
        &self,
        _: String,
        _: String,
    ) -> Result<Option<Record>, CommonError> {
        return Err(CommonError::NotSupportFeature(
            "JournalStorageAdapter".to_string(),
            "stream_write".to_string(),
        ));
    }
}
