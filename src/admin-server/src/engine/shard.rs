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
use common_base::http_response::{error_response, success_response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct ShardListReq {
    pub shard_name: Option<String>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub sort_field: Option<String>,
    pub sort_by: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShardDetailReq {
    pub shard_name: String,
}

pub async fn shard_list(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<ShardListReq>,
) -> String {
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
    success_response("shard_list")
}

pub async fn shard_create(
    State(_state): State<Arc<HttpState>>,
    Json(_params): Json<ShardListReq>,
) -> String {
    success_response("shard_list")
}

pub async fn shard_delete(
    State(_state): State<Arc<HttpState>>,
    Json(_params): Json<ShardListReq>,
) -> String {
    success_response("shard_list")
}

pub async fn segment_list(
    State(_state): State<Arc<HttpState>>,
    Json(_params): Json<ShardDetailReq>,
) -> String {
    success_response("shard_detail")
}
