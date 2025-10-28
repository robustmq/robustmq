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
use std::time::Duration;

use common_base::tools::now_mills;
use dashmap::DashMap;
use metadata_struct::adapter::read_config::ReadConfig;
use metadata_struct::journal::segment::segment_name;
use metadata_struct::journal::shard::shard_name_iden;
use protocol::journal::journal_engine::{
    FetchOffsetReqBody, FetchOffsetShard, ReadReqBody, ReadReqFilter, ReadReqMessage,
    ReadReqOptions, ReadType,
};
use tokio::select;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::time::sleep;
use tracing::error;

use crate::cache::{get_active_segment, get_metadata_by_shard, get_segment_leader, MetadataCache};
use crate::connection::ConnectionManager;
use crate::error::JournalClientError;
use crate::service::{batch_read, fetch_offset};

#[derive(Clone)]
pub struct ReadShardByOffset {
    pub namespace: String,
    pub shard_name: String,
    pub offset: u64,
}

#[derive(Clone)]
pub struct ReadMessageData {
    pub namespace: String,
    pub shard_name: String,
    pub segment: u32,
    pub offset: u64,
    pub key: String,
    pub value: Vec<u8>,
    pub tags: Vec<String>,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct AsyncReader {
    metadata_cache: Arc<MetadataCache>,
    connection_manager: Arc<ConnectionManager>,
    data_sender: Sender<ReadMessageData>,
    stop_send: Option<Sender<bool>>,
}

impl AsyncReader {
    pub fn new(
        metadata_cache: Arc<MetadataCache>,
        connection_manager: Arc<ConnectionManager>,
    ) -> Self {
        let (data_sender, data_recv) = broadcast::channel::<ReadMessageData>(500);
        AsyncReader {
            metadata_cache,
            connection_manager,
            data_sender,
            stop_send: None,
        }
    }

    pub async fn start_read_by_offset(
        &self,
        shards: Vec<ReadShardByOffset>,
        read_config: ReadConfig,
    ) {
        let (stop_send, stop_recv) = broadcast::channel::<bool>(1);
        start_read_thread_by_group(
            self.connection_manager.clone(),
            self.metadata_cache.clone(),
            self.data_sender.clone(),
            shards,
            read_config,
            stop_recv,
        );
    }

    pub async fn read(&self) -> Result<Vec<ReadMessageData>, JournalClientError> {
        let mut results: Vec<ReadMessageData> = Vec::new();
        let start_time = now_mills();
        let mut recv = self.data_sender.subscribe();
        loop {
            if (now_mills() - start_time) >= 100 {
                break;
            }
            let data = recv.recv().await?;
            results.push(data);
        }
        Ok(results)
    }

    pub async fn close(&self) -> Result<(), JournalClientError> {
        if let Some(stop) = self.stop_send.clone() {
            stop.send(true)?;
        }
        Ok(())
    }
}

fn start_read_thread_by_group(
    connection_manager: Arc<ConnectionManager>,
    metadata_cache: Arc<MetadataCache>,
    data_sender: Sender<ReadMessageData>,
    shards: Vec<ReadShardByOffset>,
    read_config: ReadConfig,
    mut stop_recv: Receiver<bool>,
) {
    tokio::spawn(async move {
        loop {
            select! {
                val = stop_recv.recv()=>{
                    if let Ok(flag) = val {

                    }
                },
                val = async_read_data_by_offset(&connection_manager, &metadata_cache, &shards, &read_config)=>{
                    match val{
                        Ok(messages) => {
                            if messages.is_empty() {
                                sleep(Duration::from_millis(100)).await;
                                continue;
                            }
                            for raw in messages{
                                if let Err(e) = data_sender.send(raw){
                                    error!("{}",e);
                                }
                            }
                        },
                        Err(e) =>{
                            sleep(Duration::from_millis(100)).await;
                            error!("{}",e);
                        }
                    }

                }
            }
        }
    });
}

pub async fn async_read_data_by_offset(
    connection_manager: &Arc<ConnectionManager>,
    metadata_cache: &Arc<MetadataCache>,
    shard: &Vec<ReadShardByOffset>,
    read_config: &ReadConfig,
) -> Result<Vec<ReadMessageData>, JournalClientError> {
    let leader_shards = group_by_reader_leader(connection_manager, metadata_cache, shard).await;
    let mut results = Vec::new();
    for (leader_id, shards) in leader_shards {
        let mut messages = Vec::new();
        for raw in shards {
            messages.push(ReadReqMessage {
                namespace: raw.1.namespace,
                shard_name: raw.1.shard_name,
                segment: raw.0,
                ready_type: ReadType::Offset.into(),
                filter: Some(ReadReqFilter {
                    offset: raw.1.offset,
                    ..Default::default()
                }),
                options: Some(ReadReqOptions {
                    max_size: read_config.max_size,
                    max_record: read_config.max_record_num,
                }),
            });
        }

        let body = ReadReqBody { messages };
        let result = batch_read(connection_manager, leader_id, body).await?;
        for shard_data in result.messages {
            for message in shard_data.messages {
                let val = ReadMessageData {
                    namespace: shard_data.namespace.clone(),
                    shard_name: shard_data.shard_name.clone(),
                    segment: shard_data.segment,
                    offset: message.offset,
                    key: message.key,
                    value: message.value,
                    tags: message.tags,
                    timestamp: message.timestamp,
                };
                results.push(val);
            }
        }
    }
    Ok(results)
}

pub async fn async_read_data_by_key(
    connection_manager: &Arc<ConnectionManager>,
    metadata_cache: &Arc<MetadataCache>,
    shards: &Vec<ReadShardByOffset>,
    key: &str,
    read_config: &ReadConfig,
) -> Result<Vec<ReadMessageData>, JournalClientError> {
    let leader_shards = group_by_reader_leader(connection_manager, metadata_cache, shards).await;
    let mut results = Vec::new();
    for (leader_id, shards) in leader_shards {
        let mut messages = Vec::new();
        for raw in shards {
            messages.push(ReadReqMessage {
                namespace: raw.1.namespace,
                shard_name: raw.1.shard_name,
                segment: raw.0,
                ready_type: ReadType::Key.into(),
                filter: Some(ReadReqFilter {
                    offset: raw.1.offset,
                    key: key.to_owned(),
                    ..Default::default()
                }),
                options: Some(ReadReqOptions {
                    max_size: read_config.max_size,
                    max_record: read_config.max_record_num,
                }),
            });
        }

        let body = ReadReqBody { messages };
        let result = batch_read(connection_manager, leader_id, body).await?;
        for shard_data in result.messages {
            for message in shard_data.messages {
                let val = ReadMessageData {
                    namespace: shard_data.namespace.clone(),
                    shard_name: shard_data.shard_name.clone(),
                    segment: shard_data.segment,
                    offset: message.offset,
                    key: message.key,
                    value: message.value,
                    tags: message.tags,
                    timestamp: message.timestamp,
                };
                results.push(val);
            }
        }
    }
    Ok(results)
}

pub async fn async_read_data_by_tag(
    connection_manager: &Arc<ConnectionManager>,
    metadata_cache: &Arc<MetadataCache>,
    shards: &Vec<ReadShardByOffset>,
    tag: &str,
    read_config: &ReadConfig,
) -> Result<Vec<ReadMessageData>, JournalClientError> {
    let leader_shards = group_by_reader_leader(connection_manager, metadata_cache, shards).await;
    let mut results = Vec::new();
    for (leader_id, shards) in leader_shards {
        let mut messages = Vec::new();
        for raw in shards {
            messages.push(ReadReqMessage {
                namespace: raw.1.namespace,
                shard_name: raw.1.shard_name,
                segment: raw.0,
                ready_type: ReadType::Tag.into(),
                filter: Some(ReadReqFilter {
                    offset: raw.1.offset,
                    tag: tag.to_owned(),
                    ..Default::default()
                }),
                options: Some(ReadReqOptions {
                    max_size: read_config.max_size,
                    max_record: read_config.max_record_num,
                }),
            });
        }

        let body = ReadReqBody { messages };
        let result = batch_read(connection_manager, leader_id, body).await?;
        for shard_data in result.messages {
            for message in shard_data.messages {
                let val = ReadMessageData {
                    namespace: shard_data.namespace.clone(),
                    shard_name: shard_data.shard_name.clone(),
                    segment: shard_data.segment,
                    offset: message.offset,
                    key: message.key,
                    value: message.value,
                    tags: message.tags,
                    timestamp: message.timestamp,
                };
                results.push(val);
            }
        }
    }
    Ok(results)
}

async fn group_by_reader_leader(
    connection_manager: &Arc<ConnectionManager>,
    metadata_cache: &Arc<MetadataCache>,
    shards: &Vec<ReadShardByOffset>,
) -> DashMap<u64, Vec<(u32, ReadShardByOffset)>> {
    let result: DashMap<u64, Vec<(u32, ReadShardByOffset)>> = DashMap::with_capacity(2);
    for shard in shards {
        let segment = get_active_segment(
            metadata_cache,
            connection_manager,
            &shard.namespace,
            &shard.shard_name,
        )
        .await;
        let leader = get_segment_leader(
            metadata_cache,
            connection_manager,
            &shard.namespace,
            &shard.shard_name,
        )
        .await;
        if let Some(mut node) = result.get_mut(&leader) {
            node.push((segment, shard.to_owned()));
        } else {
            result.insert(leader, vec![(segment, shard.to_owned())]);
        }
    }
    result
}

pub async fn fetch_offset_by_timestamp(
    connection_manager: &Arc<ConnectionManager>,
    metadata_cache: &Arc<MetadataCache>,
    namespace: &str,
    shard_name: &str,
    timestamp: u64,
) -> Result<(u32, u64), JournalClientError> {
    let metadatas =
        get_metadata_by_shard(metadata_cache, connection_manager, namespace, shard_name).await;

    if metadatas.is_empty() {
        return Err(JournalClientError::NotShardMetadata(shard_name_iden(
            namespace, shard_name,
        )));
    }

    let first = metadatas.first().unwrap();
    let end = metadatas.last().unwrap();

    let segment = if timestamp < first.start_offset as u64 {
        first.segment_no
    } else if timestamp > end.end_offset as u64 {
        end.segment_no
    } else {
        let mut segment_opt: Option<u32> = None;
        for meta in metadatas.iter() {
            if timestamp >= meta.start_offset as u64 && timestamp <= meta.end_offset as u64 {
                segment_opt = Some(meta.segment_no);
            }
        }
        if let Some(segment) = segment_opt {
            segment
        } else {
            first.segment_no
        }
    };

    let node_id = if let Some(node_id) =
        metadata_cache.get_leader_by_segment(namespace, shard_name, segment)
    {
        node_id
    } else {
        return Err(JournalClientError::NotLeader(segment_name(
            namespace, shard_name, segment,
        )));
    };

    let body = FetchOffsetReqBody {
        shards: vec![FetchOffsetShard {
            namespace: namespace.to_owned(),
            shard_name: shard_name.to_owned(),
            segment_no: segment,
            timestamp,
        }],
    };
    let resp = fetch_offset(connection_manager, node_id, body).await?;
    if let Some(resp_shard) = resp.shard_offsets.first() {
        return Ok((segment, resp_shard.offset as u64));
    }
    Ok((first.segment_no, first.start_offset as u64))
}
