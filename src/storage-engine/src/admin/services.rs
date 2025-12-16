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
use crate::core::error::JournalServerError;
use crate::segment::SegmentIdentity;
use protocol::journal::journal_admin::{
    ListSegmentReply, ListSegmentRequest, ListShardReply, ListShardRequest,
};

/// List shards based on the request parameters
pub async fn list_shard_by_req(
    cache_manager: &Arc<CacheManager>,
    request: &ListShardRequest,
) -> Result<ListShardReply, JournalServerError> {
    let mut shards = Vec::new();

    if request.shard_name.is_empty() {
        // Get all shards
        for shard in cache_manager.get_shards() {
            let data = serde_json::to_string(&shard)?;
            shards.push(data);
        }
    } else {
        // Get shard by name
        if let Some(shard) = cache_manager.get_shard(&request.shard_name) {
            let data = serde_json::to_string(&shard)?;
            shards.push(data);
        }
    }

    Ok(ListShardReply { shards })
}

/// List segments based on the request parameters
pub async fn list_segment_by_req(
    cache_manager: &Arc<CacheManager>,
    request: &ListSegmentRequest,
) -> Result<ListSegmentReply, JournalServerError> {
    let mut segments = Vec::new();

    if request.segment_no == -1 {
        // Get all segments by shard
        for segment in cache_manager.get_segments_list_by_shard(&request.shard_name) {
            let data = serde_json::to_string(&segment)?;
            segments.push(data);
        }
    } else {
        // Get specific segment
        let segment_iden = SegmentIdentity {
            shard_name: request.shard_name.clone(),
            segment_seq: request.segment_no as u32,
        };

        if let Some(segment) = cache_manager.get_segment(&segment_iden) {
            let data = serde_json::to_string(&segment)?;
            segments.push(data);
        }
    }

    Ok(ListSegmentReply { segments })
}
