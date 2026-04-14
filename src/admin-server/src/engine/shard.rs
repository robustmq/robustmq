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

// Offset-related handlers (get_offset_by_timestamp, get_offset_by_group, commit_offset)
// have been moved to: src/admin-server/src/cluster/offset.rs
//
// Segment-related handlers (segment_list) have been moved to:
// src/admin-server/src/engine/segment.rs

use crate::state::HttpState;
use crate::tool::{
    query::{apply_filters, apply_pagination, apply_sorting, build_query_params, Queryable},
    PageReplyData,
};
use axum::{extract::State, Json};
use common_base::http_response::{error_response, success_response};
use metadata_struct::adapter::adapter_offset::AdapterShardInfo;
use metadata_struct::adapter::adapter_shard::AdapterShardDetail;
use metadata_struct::storage::shard::EngineShardConfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ShardListRow {
    pub shard_info: AdapterShardDetail,
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
        desc: "".to_string(),
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
