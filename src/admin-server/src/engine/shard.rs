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
use crate::tool::{
    query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
    PageReplyData,
};
use axum::{extract::State, Json};
use common_base::http_response::{error_response, success_response};
use metadata_struct::storage::adapter_offset::{
    AdapterConsumerGroupOffset, AdapterOffsetStrategy, AdapterShardInfo,
};
use metadata_struct::storage::segment::EngineSegment;
use metadata_struct::storage::segment_meta::EngineSegmentMetadata;
use metadata_struct::storage::shard::{EngineShard, EngineShardConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use storage_engine::segment::SegmentIdentity;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ShardListReq {
    pub shard_name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
    pub filter_field: Option<String>,
    pub filter_values: Option<Vec<String>>,
    pub exact_match: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShardCreateReq {
    pub shard_name: String,
    pub config: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShardDeleteReq {
    pub shard_name: String,
}

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

#[derive(Serialize, Deserialize, Debug)]
pub struct GetOffsetByTimestampReq {
    pub shard_name: String,
    pub timestamp: u64,
    pub strategy: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetOffsetByTimestampResp {
    pub offset: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetOffsetByGroupReq {
    pub group_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetOffsetByGroupResp {
    pub offsets: Vec<AdapterConsumerGroupOffset>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitOffsetReq {
    pub group_name: String,
    pub offsets: HashMap<String, u64>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ShardListRow {
    pub shard_info: EngineShard,
}

impl Queryable for ShardListRow {
    fn get_field_str(&self, _field: &str) -> Option<String> {
        None
    }
}

pub async fn shard_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<ShardListReq>,
) -> String {
    let options = build_query_params(
        params.page,
        params.limit,
        params.sort_field,
        params.sort_by,
        params.filter_field,
        params.filter_values,
        params.exact_match,
    );

    let result = match state
        .engine_context
        .engine_adapter_handler
        .list_shard(params.shard_name)
        .await
    {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    let shards: Vec<ShardListRow> = result
        .into_iter()
        .map(|shard| ShardListRow { shard_info: shard })
        .collect();

    let filtered = apply_filters(shards, &options);
    let sorted = apply_sorting(filtered, &options);
    let pagination = apply_pagination(sorted, &options);

    success_response(PageReplyData {
        data: pagination.0,
        total_count: pagination.1,
    })
}

pub async fn shard_create(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<ShardCreateReq>,
) -> String {
    if params.shard_name.is_empty() {
        return error_response("shard_name cannot be empty".to_string());
    }

    if params.config.is_empty() {
        return error_response("config cannot be empty".to_string());
    }

    let config: EngineShardConfig = match serde_json::from_str(&params.config) {
        Ok(c) => c,
        Err(e) => {
            return error_response(format!("Invalid config JSON: {}", e));
        }
    };

    let shard_info = AdapterShardInfo {
        shard_name: params.shard_name.clone(),
        config,
    };

    if let Err(e) = state
        .engine_context
        .engine_adapter_handler
        .create_shard(&shard_info)
        .await
    {
        return error_response(e.to_string());
    }

    success_response("success")
}

pub async fn shard_delete(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<ShardDeleteReq>,
) -> String {
    if params.shard_name.is_empty() {
        return error_response("shard_name cannot be empty".to_string());
    }

    if let Err(e) = state
        .engine_context
        .engine_adapter_handler
        .delete_shard(&params.shard_name)
        .await
    {
        return error_response(e.to_string());
    }

    success_response("success")
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

pub async fn get_offset_by_timestamp(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<GetOffsetByTimestampReq>,
) -> String {
    if params.shard_name.is_empty() {
        return error_response("shard_name cannot be empty".to_string());
    }

    let strategy = match params.strategy.to_lowercase().as_str() {
        "earliest" => AdapterOffsetStrategy::Earliest,
        "latest" => AdapterOffsetStrategy::Latest,
        _ => {
            return error_response(format!(
                "Invalid strategy '{}', must be 'earliest' or 'latest'",
                params.strategy
            ));
        }
    };

    let offset = match state
        .engine_context
        .engine_adapter_handler
        .get_offset_by_timestamp(&params.shard_name, params.timestamp, strategy)
        .await
    {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    success_response(GetOffsetByTimestampResp { offset })
}

pub async fn get_offset_by_group(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<GetOffsetByGroupReq>,
) -> String {
    if params.group_name.is_empty() {
        return error_response("group_name cannot be empty".to_string());
    }

    let offsets = match state
        .engine_context
        .engine_adapter_handler
        .get_offset_by_group(&params.group_name)
        .await
    {
        Ok(data) => data,
        Err(e) => {
            return error_response(e.to_string());
        }
    };

    success_response(GetOffsetByGroupResp { offsets })
}

pub async fn commit_offset(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<CommitOffsetReq>,
) -> String {
    if params.group_name.is_empty() {
        return error_response("group_name cannot be empty".to_string());
    }

    if params.offsets.is_empty() {
        return error_response("offsets cannot be empty".to_string());
    }

    if let Err(e) = state
        .engine_context
        .engine_adapter_handler
        .commit_offset(&params.group_name, &params.offsets)
        .await
    {
        return error_response(e.to_string());
    }

    success_response("success")
}
