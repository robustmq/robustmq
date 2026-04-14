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

use crate::state::HttpState;
use axum::{extract::State, Json};
use common_base::http_response::success_response;
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use storage_engine::filesegment::SegmentIdentity;

#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentListReq {
    pub shard_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentListResp {
    pub segment_list: Vec<SegmentListRespRaw>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SegmentListRespRaw {
    pub segment: EngineSegment,
    pub segment_meta: Option<EngineSegmentMetadata>,
}

pub async fn segment_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<SegmentListReq>,
) -> String {
    let segment_list = state
        .engine_context
        .cache_manager
        .get_segments_list_by_shard(&params.shard_name);

    let mut results: Vec<_> = Vec::new();
    for segment in segment_list {
        let segment_iden = SegmentIdentity::new(&segment.shard_name, segment.segment_seq);
        let meta = state
            .engine_context
            .cache_manager
            .get_segment_meta(&segment_iden);
        results.push(SegmentListRespRaw {
            segment: segment.clone(),
            segment_meta: meta,
        });
    }
    success_response(SegmentListResp {
        segment_list: results,
    })
}
