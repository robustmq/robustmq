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
    client: JournalClient,
    offset_manager: Arc<OffsetManager>,
}

impl JournalStorageAdapter {
    pub async fn new(
        offset_manager: Arc<OffsetManager>,
        config: StorageDriverJournalConfig,
    ) -> Result<JournalStorageAdapter, CommonError> {
        let client = match JournalClient::new(config.journal_addrs.clone()).await {
            Ok(client) => client,
            Err(e) => return Err(CommonError::CommonError(e.to_string())),
        };
        let adapter = JournalStorageAdapter {
            offset_manager,
            client,
        };
        Ok(adapter)
    }
}

#[async_trait]
impl StorageAdapter for JournalStorageAdapter {
    async fn create_shard(&self, shard: &ShardInfo) -> Result<(), CommonError> {
        if let Err(e) = self
            .client
            .create_shard("", &shard.shard_name, shard.replica_num)
            .await
        {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }

    async fn list_shard(&self, shard: &str) -> Result<Vec<ShardInfo>, CommonError> {
        let reply = self
            .client
            .list_shard("", shard)
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))?;

        let mut res = Vec::new();

        for shard in reply {
            let shard_info = ShardInfo {
                shard_name: shard.shard_name,
                ..Default::default()
            };

            res.push(shard_info);
        }

        Ok(res)
    }

    async fn delete_shard(&self, shard: &str) -> Result<(), CommonError> {
        if let Err(e) = self.client.delete_shard("", shard).await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }

    async fn write(&self, shard: &str, record: &Record) -> Result<u64, CommonError> {
        let data = JournalClientWriteData {
            key: record.key.clone().unwrap_or_default(),
            content: record.data.to_vec(),
            tags: record.tags.clone().unwrap_or_default(),
        };

        match self.client.write("", shard, data).await {
            Ok(resp) => {
                if let Some(err) = resp.error {
                    return Err(CommonError::CommonError(err));
                }
                Ok(resp.offset)
            }
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    async fn batch_write(&self, shard: &str, records: &[Record]) -> Result<Vec<u64>, CommonError> {
        let mut data = Vec::new();
        for record in records {
            data.push(JournalClientWriteData {
                key: record.key.clone().unwrap_or_default(),
                content: record.data.to_vec(),
                tags: record.tags.clone().unwrap_or_default(),
            });
        }

        match self.client.batch_write("", shard, data).await {
            Ok(resp) => {
                let mut resp_offsets = Vec::new();
                for raw in resp {
                    if let Some(err) = raw.error {
                        return Err(CommonError::CommonError(err));
                    }
                    resp_offsets.push(raw.offset);
                }
                Ok(resp_offsets)
            }
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    async fn read_by_offset(
        &self,
        shard: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        match self
            .client
            .read_by_offset("", shard, offset, read_config)
            .await
        {
            Ok(results) => Ok(results),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    async fn read_by_tag(
        &self,
        shard: &str,
        offset: u64,
        tag: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        match self
            .client
            .read_by_tag("", shard, offset, tag, read_config)
            .await
        {
            Ok(results) => Ok(results),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    async fn read_by_key(
        &self,
        shard: &str,
        offset: u64,
        key: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        match self
            .client
            .read_by_key("", shard, offset, key, read_config)
            .await
        {
            Ok(results) => Ok(results),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    async fn get_offset_by_group(&self, group: &str) -> Result<Vec<ShardOffset>, CommonError> {
        self.offset_manager.get_offset(group).await
    }

    async fn get_offset_by_timestamp(
        &self,
        shard: &str,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        match self
            .client
            .get_offset_by_timestamp("", shard, timestamp)
            .await
        {
            Ok(result) => Ok(Some(ShardOffset {
                shard_name: shard.to_string(),
                segment_no: result.0,
                offset: result.1,
                ..Default::default()
            })),
            Err(e) => Err(CommonError::CommonError(e.to_string())),
        }
    }

    async fn commit_offset(
        &self,
        group_name: &str,
        offset: &HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        self.offset_manager.commit_offset(group_name, offset).await
    }

    async fn message_expire(&self, _config: &MessageExpireConfig) -> Result<(), CommonError> {
        Ok(())
    }

    async fn close(&self) -> Result<(), CommonError> {
        if let Err(e) = self.client.close().await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }
}
