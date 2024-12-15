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

use std::sync::Arc;

use common_base::error::common::CommonError;
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;

use crate::async_reader::AsyncReader;
use crate::cache::MetadataCache;
use crate::connection::ConnectionManager;
use crate::error::JournalClientError;

pub struct JournalRead {
    connection_manager: Arc<ConnectionManager>,
    metadata_cache: Arc<MetadataCache>,
    reader: AsyncReader,
}
impl JournalRead {
    pub fn new(addrs: Vec<String>) -> Self {
        let metadata_cache = Arc::new(MetadataCache::new(addrs));
        let connection_manager = Arc::new(ConnectionManager::new(metadata_cache.clone()));
        let reader = AsyncReader::new(metadata_cache.clone(), connection_manager.clone());
        JournalRead {
            connection_manager,
            metadata_cache,
            reader,
        }
    }
    pub async fn read_by_offset(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, JournalClientError> {
        let messages = self.reader.read().await?;
        let mut results = Vec::new();
        for raw in messages {
            let record = Record {
                offset: Some(raw.offset),
                key: raw.key,
                data: raw.value,
                tags: raw.tags,
                header: Vec::new(),
                timestamp: raw.timestamp,
            };
            results.push(record);
        }
        Ok(results)
    }

    pub async fn read_by_key(
        &self,
        namespace: &str,
        shard_name: &str,
        key: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, JournalClientError> {
        Ok(Vec::new())
    }

    pub async fn read_by_tag(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        tag: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, JournalClientError> {
        Ok(Vec::new())
    }

    pub async fn get_offset_by_timestamp(
        &self,
        namespace: &str,
        shard_name: &str,
        timestamp: u64,
    ) -> Result<(u32, u64), JournalClientError> {
        Ok((0, 0))
    }

    pub async fn close(&self) -> Result<(), CommonError> {
        Ok(())
    }
}
