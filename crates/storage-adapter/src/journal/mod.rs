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

use std::collections::HashMap;
use std::sync::Arc;

use axum::async_trait;
use common_base::error::common::CommonError;
use grpc_clients::pool::ClientPool;
use journal_client::client::{JournalClient, JournalClientWriteData};
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;
use offset::PlaceOffsetManager;

use crate::storage::{ShardInfo, ShardOffset, StorageAdapter};

pub mod offset;

pub struct JournalStorageAdapter {
    cluster_name: String,
    client: JournalClient,
    offset_manager: PlaceOffsetManager,
}

impl JournalStorageAdapter {
    pub async fn new(
        client_pool: Arc<ClientPool>,
        cluster_name: String,
        journal_addrs: Vec<String>,
        place_addrs: Vec<String>,
    ) -> Result<JournalStorageAdapter, CommonError> {
        let offset_manager = PlaceOffsetManager::new(client_pool, place_addrs.clone());
        let client = match JournalClient::new(journal_addrs.clone()).await {
            Ok(client) => client,
            Err(e) => return Err(CommonError::CommonError(e.to_string())),
        };
        let adapter = JournalStorageAdapter {
            offset_manager,
            cluster_name,
            client,
        };
        Ok(adapter)
    }
}

#[async_trait]
impl StorageAdapter for JournalStorageAdapter {
    async fn create_shard(&self, shard: ShardInfo) -> Result<(), CommonError> {
        if let Err(e) = self
            .client
            .create_shard(&shard.namespace, &shard.shard_name, shard.replica_num)
            .await
        {
            return Err(CommonError::CommonError(e.to_string()));
        }
        return Ok(());
    }

    async fn list_shard(
        &self,
        namespace: String,
        shard_name: String,
    ) -> Result<Vec<ShardInfo>, CommonError> {
        let reply = self
            .client
            .list_shard(namespace.as_str(), shard_name.as_str())
            .await
            .map_err(|e| CommonError::CommonError(e.to_string()))?;

        let mut res = Vec::new();

        for shard in reply {
            let shard_info = ShardInfo {
                namespace: shard.namespace,
                shard_name: shard.shard_name,
                ..Default::default()
            };

            res.push(shard_info);
        }

        Ok(res)
    }

    async fn delete_shard(&self, namespace: String, shard_name: String) -> Result<(), CommonError> {
        if let Err(e) = self.client.delete_shard(&namespace, &shard_name).await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }

    async fn write(
        &self,
        namespace: String,
        shard_name: String,
        record: Record,
    ) -> Result<u64, CommonError> {
        let data = JournalClientWriteData {
            key: record.key,
            content: record.data,
            tags: record.tags,
        };

        match self.client.write(namespace, shard_name, data).await {
            Ok(resp) => {
                if let Some(err) = resp.error {
                    return Err(CommonError::CommonError(err));
                }
                return Ok(resp.offset);
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn batch_write(
        &self,
        namespace: String,
        shard_name: String,
        records: Vec<Record>,
    ) -> Result<Vec<u64>, CommonError> {
        let mut data = Vec::new();
        for record in records {
            data.push(JournalClientWriteData {
                key: record.key,
                content: record.data,
                tags: record.tags,
            });
        }

        match self.client.batch_write(namespace, shard_name, data).await {
            Ok(resp) => {
                let mut resp_offsets = Vec::new();
                for raw in resp {
                    if let Some(err) = raw.error {
                        return Err(CommonError::CommonError(err));
                    }
                    resp_offsets.push(raw.offset);
                }
                return Ok(resp_offsets);
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn read_by_offset(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        match self
            .client
            .read_by_offset(&namespace, &shard_name, offset, &read_config)
            .await
        {
            Ok(results) => Ok(results),
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn read_by_tag(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        tag: String,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        match self
            .client
            .read_by_tag(&namespace, &shard_name, offset, &tag, &read_config)
            .await
        {
            Ok(results) => Ok(results),
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn read_by_key(
        &self,
        namespace: String,
        shard_name: String,
        offset: u64,
        key: String,
        read_config: ReadConfig,
    ) -> Result<Vec<Record>, CommonError> {
        match self
            .client
            .read_by_key(&namespace, &shard_name, offset, &key, &read_config)
            .await
        {
            Ok(results) => Ok(results),
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn get_offset_by_group(&self, group: String) -> Result<Vec<ShardOffset>, CommonError> {
        self.offset_manager
            .get_shard_offset(&self.cluster_name, &group)
            .await
    }

    async fn get_offset_by_timestamp(
        &self,
        namespace: String,
        shard_name: String,
        timestamp: u64,
    ) -> Result<Option<ShardOffset>, CommonError> {
        match self
            .client
            .get_offset_by_timestamp(&namespace, &shard_name, timestamp)
            .await
        {
            Ok(result) => {
                return Ok(Some(ShardOffset {
                    shard_name: shard_name.clone(),
                    segment_no: result.0,
                    offset: result.1,
                    ..Default::default()
                }));
            }
            Err(e) => {
                return Err(CommonError::CommonError(e.to_string()));
            }
        }
    }

    async fn commit_offset(
        &self,
        group_name: String,
        namespace: String,
        offset: HashMap<String, u64>,
    ) -> Result<(), CommonError> {
        self.offset_manager
            .commit_offset(&self.cluster_name, &group_name, &namespace, offset)
            .await
    }

    async fn close(&self) -> Result<(), CommonError> {
        if let Err(e) = self.client.close().await {
            return Err(CommonError::CommonError(e.to_string()));
        }
        Ok(())
    }
}
