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
pub struct RecordDeleteByKeysReq {
    pub shard_name: String,
    pub keys: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RecordDeleteByOffsetsReq {
    pub shard_name: String,
    pub offsets: Vec<u64>,
}

pub async fn record_delete_by_offsets(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<RecordDeleteByOffsetsReq>,
) -> String {
    if params.shard_name.is_empty() {
        return error_response("shard_name cannot be empty".to_string());
    }
    if params.offsets.is_empty() {
        return error_response("offsets cannot be empty".to_string());
    }

    if let Err(e) = state
        .engine_context
        .engine_adapter_handler
        .delete_by_offsets(&params.shard_name, &params.offsets)
        .await
    {
        return error_response(e.to_string());
    }

    success_response("success")
}

pub async fn record_delete_by_keys(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<RecordDeleteByKeysReq>,
) -> String {
    if params.shard_name.is_empty() {
        return error_response("shard_name cannot be empty".to_string());
    }
    if params.keys.is_empty() {
        return error_response("keys cannot be empty".to_string());
    }

    let key_refs: Vec<&[u8]> = params.keys.iter().map(|s| s.as_bytes()).collect();
    if let Err(e) = state
        .engine_context
        .engine_adapter_handler
        .delete_by_keys(&params.shard_name, &key_refs)
        .await
    {
        return error_response(e.to_string());
    }

    success_response("success")
}
