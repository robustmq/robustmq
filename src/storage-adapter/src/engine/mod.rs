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
use crate::offset::OffsetManager;
use crate::storage::{ShardInfo, ShardOffset, StorageAdapter};
use axum::async_trait;
use common_base::error::common::CommonError;
use common_config::storage::journal::StorageDriverJournalConfig;
use journal_client::client::{JournalClient, JournalClientWriteData};
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;
use std::collections::HashMap;
use std::sync::Arc;

pub struct JournalStorageAdapter {
    offset_manager: Arc<OffsetManager>,
}

impl JournalStorageAdapter {
    pub async fn new(
        offset_manager: Arc<OffsetManager>,
        config: StorageDriverJournalConfig,
    ) -> JournalStorageAdapter {
        JournalStorageAdapter { offset_manager }
    }
}

#[async_trait]
impl StorageAdapter for JournalStorageAdapter {
    async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
        Ok(())
    }

    async fn list_shard(&self, shard: &str) -> Result<Vec<ShardInfo>, CommonError> {
        Ok(Vec::new())
    }

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError> {
        Ok(())
    }

    async fn write(&self, shard: &str, record: &Record) -> Result<u64, CommonError> {
        Ok(0)
    }

    async fn batch_write(&self, shard: &str, records: &[Record]) -> Result<Vec<u64>, CommonError> {
        Ok(Vec::new())
    }

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        Ok(Vec::new())
    }

    async fn read_by_tag(
        &self,
        shard: &str,
        offset: u64,
        tag: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        Ok(Vec::new())
    }

    async fn read_by_key(
        &self,
        _shard: &str,
        _offset: u64,
        _key: &str,
        _read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        Ok(Vec::new())
    }

    async fn get_offset_by_group(&self, group: &str) -> Result<Vec<ShardOffset>, CommonError> {
        Ok(Vec::new())
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        Ok(None)
    }

    async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        Ok(())
    }

    async fn message_expire(&self, _config: &MessageExpireConfig) -> Result<(), CommonError> {
        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}
