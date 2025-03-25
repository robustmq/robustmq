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
use common_base::utils::crc::calc_crc32;
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::adapter::record::Record;
use metadata_struct::journal::shard::JournalShard;
use protocol::journal_server::journal_engine::{
    CreateShardReqBody, DeleteShardReqBody, GetClusterMetadataNode, GetShardMetadataRespShard,
    ListShardReqBody,
};
use tokio::sync::broadcast::{self, Sender};

use super::cache::{load_node_cache, MetadataCache};
use super::connection::ConnectionManager;
use super::error::JournalClientError;
use crate::async_reader::{
    async_read_data_by_key, async_read_data_by_offset, async_read_data_by_tag,
    fetch_offset_by_timestamp, AsyncReader, ReadShardByOffset,
};
use crate::async_writer::{AsyncWriter, SenderMessage, SenderMessageResp};
use crate::cache::get_active_segment;
use crate::service::{create_shard, delete_shard, list_shard};

#[derive(Default, Clone)]
pub struct JournalClientWriteData {
    pub key: String,
    pub content: Vec<u8>,
    pub tags: Vec<String>,
}

#[derive(Clone)]
pub struct JournalClient {
    connection_manager: Arc<ConnectionManager>,
    metadata_cache: Arc<MetadataCache>,
    writer: Arc<AsyncWriter>,
    reader: Arc<AsyncReader>,
    stop_send: Sender<bool>,
}

impl JournalClient {
    pub async fn new(addrs: Vec<String>) -> Result<JournalClient, JournalClientError> {
        if addrs.is_empty() {
            return Err(JournalClientError::AddrsNotEmpty);
        }

        let metadata_cache = Arc::new(MetadataCache::new(addrs));
        let connection_manager = Arc::new(ConnectionManager::new(metadata_cache.clone()));
        let (stop_send, _) = broadcast::channel::<bool>(2);
        let writer = Arc::new(AsyncWriter::new(
            connection_manager.clone(),
            metadata_cache.clone(),
        ));

        let reader = Arc::new(AsyncReader::new(
            metadata_cache.clone(),
            connection_manager.clone(),
        ));

        let client = JournalClient {
            metadata_cache,
            connection_manager,
            writer,
            reader,
            stop_send,
        };
        client.validate()?;
        client.connect().await?;
        Ok(client)
    }

    pub async fn create_shard(
        &self,
        namespace: &str,
        shard_name: &str,
        replica_num: u32,
    ) -> Result<(), JournalClientError> {
        let body = CreateShardReqBody {
            namespace: namespace.to_string(),
            shard_name: shard_name.to_string(),
            replica_num,
        };
        let _ = create_shard(&self.connection_manager, body).await?;
        Ok(())
    }

    pub async fn delete_shard(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Result<(), JournalClientError> {
        let body = DeleteShardReqBody {
            namespace: namespace.to_string(),
            shard_name: shard_name.to_string(),
        };
        let _ = delete_shard(&self.connection_manager, body).await?;
        Ok(())
    }

    pub async fn list_shard(
        &self,
        namespace: &str,
        shard_name: &str,
    ) -> Result<Vec<JournalShard>, JournalClientError> {
        let body = ListShardReqBody {
            namespace: namespace.to_string(),
            shard_name: shard_name.to_string(),
        };

        let resp = list_shard(&self.connection_manager, body).await?;

        let mut res = Vec::new();

        for raw in resp.shards {
            let shard = serde_json::from_slice::<JournalShard>(&raw)?;
            res.push(shard);
        }

        Ok(res)
    }

    pub async fn batch_write(
        &self,
        namespace: String,
        shard_name: String,
        data: Vec<JournalClientWriteData>,
    ) -> Result<Vec<SenderMessageResp>, JournalClientError> {
        let active_segment = get_active_segment(
            &self.metadata_cache,
            &self.connection_manager,
            &namespace,
            &shard_name,
        )
        .await;

        let message = SenderMessage::build(&namespace, &shard_name, active_segment, data.clone());
        self.writer.send(&message).await
    }

    pub async fn write(
        &self,
        namespace: String,
        shard_name: String,
        data: JournalClientWriteData,
    ) -> Result<SenderMessageResp, JournalClientError> {
        let resp_vec = self.batch_write(namespace, shard_name, vec![data]).await?;
        if let Some(resp) = resp_vec.first() {
            return Ok(resp.to_owned());
        }
        Err(JournalClientError::WriteReqReturnEmpty)
    }

    pub async fn read_by_offset(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, JournalClientError> {
        let shards = vec![ReadShardByOffset {
            namespace: namespace.to_owned(),
            shard_name: shard_name.to_owned(),
            offset,
        }];
        let data_list = async_read_data_by_offset(
            &self.connection_manager,
            &self.metadata_cache,
            &shards,
            read_config,
        )
        .await?;

        let mut results = Vec::new();
        for raw in data_list {
            let record = Record {
                offset: Some(raw.offset),
                key: raw.key,
                data: raw.value.clone(),
                tags: raw.tags,
                header: Vec::new(),
                timestamp: raw.timestamp,
                delay_timestamp: 0,
                crc_num: calc_crc32(&raw.value),
            };
            results.push(record);
        }
        Ok(results)
    }

    pub async fn get_offset_by_timestamp(
        &self,
        namespace: &str,
        shard_name: &str,
        timestamp: u64,
    ) -> Result<(u32, u64), JournalClientError> {
        let (segment, offset) = fetch_offset_by_timestamp(
            &self.connection_manager,
            &self.metadata_cache,
            namespace,
            shard_name,
            timestamp,
        )
        .await?;

        Ok((segment, offset))
    }

    pub async fn read_by_key(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        key: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, JournalClientError> {
        let shards = vec![ReadShardByOffset {
            namespace: namespace.to_owned(),
            shard_name: shard_name.to_owned(),
            offset,
        }];
        let data_list = async_read_data_by_key(
            &self.connection_manager,
            &self.metadata_cache,
            &shards,
            key,
            read_config,
        )
        .await?;

        let mut results = Vec::new();
        for raw in data_list {
            let record = Record {
                offset: Some(raw.offset),
                key: raw.key,
                data: raw.value.clone(),
                tags: raw.tags,
                header: Vec::new(),
                timestamp: raw.timestamp,
                delay_timestamp: 0,
                crc_num: calc_crc32(&raw.value),
            };
            results.push(record);
        }
        Ok(results)
    }

    pub async fn read_by_tag(
        &self,
        namespace: &str,
        shard_name: &str,
        offset: u64,
        tag: &str,
        read_config: &ReadConfig,
    ) -> Result<Vec<Record>, JournalClientError> {
        let shards = vec![ReadShardByOffset {
            namespace: namespace.to_owned(),
            shard_name: shard_name.to_owned(),
            offset,
        }];
        let data_list = async_read_data_by_tag(
            &self.connection_manager,
            &self.metadata_cache,
            &shards,
            tag,
            read_config,
        )
        .await?;

        let mut results = Vec::new();
        for raw in data_list {
            let record = Record {
                offset: Some(raw.offset),
                key: raw.key,
                data: raw.value.clone(),
                tags: raw.tags,
                header: Vec::new(),
                timestamp: raw.timestamp,
                delay_timestamp: 0,
                crc_num: calc_crc32(&raw.value),
            };
            results.push(record);
        }
        Ok(results)
    }

    pub fn metadata(&self) -> (Vec<GetShardMetadataRespShard>, Vec<GetClusterMetadataNode>) {
        self.metadata_cache.all_metadata()
    }

    fn validate(&self) -> Result<(), JournalClientError> {
        Ok(())
    }

    async fn connect(&self) -> Result<(), JournalClientError> {
        load_node_cache(&self.metadata_cache, &self.connection_manager).await?;

        // start_update_cache_thread(
        //     self.metadata_cache.clone(),
        //     self.connection_manager.clone(),
        //     self.stop_send.subscribe(),
        // )
        // .await;

        // start_conn_gc_thread(
        //     self.metadata_cache.clone(),
        //     self.connection_manager.clone(),
        //     self.stop_send.subscribe(),
        // );
        Ok(())
    }

    pub async fn close(&self) -> Result<(), CommonError> {
        // self.stop_send.send(true)?;
        Ok(())
    }
}
