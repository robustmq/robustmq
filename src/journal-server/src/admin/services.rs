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

use crate::core::cache::CacheManager;
use crate::segment::SegmentIdentity;
use protocol::journal_server::journal_admin::{
    ListSegmentReply, ListSegmentRequest, ListShardReply, ListShardRequest,
};
use tonic::Status;

/// List shards based on the request parameters
pub async fn list_shard_by_req(
    cache_manager: &Arc<CacheManager>,
    request: &ListShardRequest,
) -> Result<ListShardReply, Status> {
    let mut shards = Vec::new();

    if request.shard_name.is_empty() {
        // Get all shards
        for shard in cache_manager.get_shards() {
            match serde_json::to_string(&shard) {
                Ok(data) => {
                    shards.push(data);
                }
                Err(e) => {
                    return Err(Status::internal(format!(
                        "Failed to serialize shard: {}",
                        e
                    )));
                }
            }
        }
    } else {
        // Get shard by name
        if let Some(shard) = cache_manager.get_shard(&request.namespace, &request.shard_name) {
            match serde_json::to_string(&shard) {
                Ok(data) => {
                    shards.push(data);
                }
                Err(e) => {
                    return Err(Status::internal(format!(
                        "Failed to serialize shard: {}",
                        e
                    )));
                }
            }
        }
    }

    Ok(ListShardReply { shards })
}

/// List segments based on the request parameters
pub async fn list_segment_by_req(
    cache_manager: &Arc<CacheManager>,
    request: &ListSegmentRequest,
) -> Result<ListSegmentReply, Status> {
    let mut segments = Vec::new();

    if request.segment_no == -1 {
        // Get all segments by shard
        for segment in
            cache_manager.get_segments_list_by_shard(&request.namespace, &request.shard_name)
        {
            match serde_json::to_string(&segment) {
                Ok(data) => {
                    segments.push(data);
                }
                Err(e) => {
                    return Err(Status::internal(format!(
                        "Failed to serialize segment: {}",
                        e
                    )));
                }
            }
        }
    } else {
        // Get specific segment
        let segment_iden = SegmentIdentity {
            namespace: request.namespace.clone(),
            shard_name: request.shard_name.clone(),
            segment_seq: request.segment_no as u32,
        };

        if let Some(segment) = cache_manager.get_segment(&segment_iden) {
            match serde_json::to_string(&segment) {
                Ok(data) => {
                    segments.push(data);
                }
                Err(e) => {
                    return Err(Status::internal(format!(
                        "Failed to serialize segment: {}",
                        e
                    )));
                }
            }
        }
    }

    Ok(ListSegmentReply { segments })
}
