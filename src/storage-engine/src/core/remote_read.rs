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

use crate::{
    clients::{manager::ClientConnectionManager, packet::build_read_req},
    core::{cache::StorageCacheManager, error::StorageEngineError},
    filesegment::SegmentIdentity,
};
use common_config::broker::broker_config;
use metadata_struct::storage::{
    adapter_read_config::AdapterReadConfig, record::StorageRecord, segment::EngineSegment,
};
use protocol::storage::protocol::{ReadReqFilter, ReadReqMessage, ReadReqOptions, ReadType};
use std::sync::Arc;

const REMOTE_READ_MAX_RETRIES: usize = 6;

#[allow(clippy::too_many_arguments)]
pub async fn remote_read_by_offset(
    client_connection_manager: &Arc<ClientConnectionManager>,
    cache_manager: &Arc<StorageCacheManager>,
    segment_iden: &SegmentIdentity,
    initial_target: u64,
    shard_name: &str,
    offset: u64,
    read_config: &AdapterReadConfig,
    batch_call_source: bool,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let messages = vec![ReadReqMessage {
        shard_name: shard_name.to_string(),
        read_type: ReadType::Offset,
        batch_call_source,
        filter: ReadReqFilter {
            offset: Some(offset),
            ..Default::default()
        },
        options: ReadReqOptions {
            max_size: read_config.max_size,
            max_record: read_config.max_record_num,
        },
    }];
    retry_send(
        client_connection_manager,
        cache_manager,
        segment_iden,
        initial_target,
        messages,
    )
    .await
}

pub async fn remote_read_by_key(
    client_connection_manager: &Arc<ClientConnectionManager>,
    cache_manager: &Arc<StorageCacheManager>,
    segment_iden: &SegmentIdentity,
    initial_target: u64,
    shard_name: &str,
    key: &[u8],
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let messages = vec![ReadReqMessage {
        shard_name: shard_name.to_string(),
        read_type: ReadType::Key,
        batch_call_source: false,
        filter: ReadReqFilter {
            key: Some(bytes::Bytes::copy_from_slice(key)),
            ..Default::default()
        },
        options: ReadReqOptions::default(),
    }];
    retry_send(
        client_connection_manager,
        cache_manager,
        segment_iden,
        initial_target,
        messages,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub async fn remote_read_by_tag(
    client_connection_manager: &Arc<ClientConnectionManager>,
    cache_manager: &Arc<StorageCacheManager>,
    segment_iden: &SegmentIdentity,
    initial_target: u64,
    shard_name: &str,
    tag: &str,
    start_offset: Option<u64>,
    read_config: &AdapterReadConfig,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let messages = vec![ReadReqMessage {
        shard_name: shard_name.to_string(),
        read_type: ReadType::Tag,
        batch_call_source: false,
        filter: ReadReqFilter {
            tag: Some(tag.to_string()),
            offset: start_offset,
            ..Default::default()
        },
        options: ReadReqOptions {
            max_size: read_config.max_size,
            max_record: read_config.max_record_num,
        },
    }];
    retry_send(
        client_connection_manager,
        cache_manager,
        segment_iden,
        initial_target,
        messages,
    )
    .await
}

async fn retry_send(
    client_connection_manager: &Arc<ClientConnectionManager>,
    cache_manager: &Arc<StorageCacheManager>,
    segment_iden: &SegmentIdentity,
    initial_target: u64,
    messages: Vec<ReadReqMessage>,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    let mut tried: Vec<u64> = Vec::new();
    let mut target = initial_target;

    for _ in 0..REMOTE_READ_MAX_RETRIES {
        tried.push(target);
        match do_send(client_connection_manager, target, messages.clone()).await {
            Ok(data) => return Ok(data),
            Err(StorageEngineError::SegmentNotOnThisBroker(_)) => {
                tracing::warn!(
                    "segment {} not on broker {}, retrying with another replica",
                    segment_iden.name(),
                    target
                );
                let cur_segment = cache_manager
                    .get_segment(segment_iden)
                    .ok_or_else(|| StorageEngineError::SegmentNotExist(segment_iden.name()))?;
                let next = pick_replica_exclude_all(&cur_segment, &tried);
                if tried.contains(&next) {
                    break;
                }
                target = next;
            }
            Err(e) => return Err(e),
        }
    }

    Err(StorageEngineError::SegmentNotOnThisBroker(
        segment_iden.name(),
    ))
}

async fn do_send(
    client_connection_manager: &Arc<ClientConnectionManager>,
    target_broker_id: u64,
    messages: Vec<ReadReqMessage>,
) -> Result<Vec<StorageRecord>, StorageEngineError> {
    client_connection_manager
        .send_read(target_broker_id, build_read_req(messages))
        .await
}

pub(super) fn pick_replica_exclude_all(segment: &EngineSegment, exclude: &[u64]) -> u64 {
    let broker_id = broker_config().broker_id;
    let candidates: Vec<u64> = segment
        .replicas
        .iter()
        .map(|r| r.node_id)
        .filter(|id| !exclude.contains(id))
        .collect();
    if candidates.is_empty() {
        return broker_id;
    }
    candidates[broker_id as usize % candidates.len()]
}
