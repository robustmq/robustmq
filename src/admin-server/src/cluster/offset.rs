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
use metadata_struct::adapter::adapter_offset::{AdapterConsumerGroupOffset, AdapterOffsetStrategy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

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
    pub tenant: String,
    pub group_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetOffsetByGroupResp {
    pub offsets: Vec<AdapterConsumerGroupOffset>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitOffsetReq {
    pub tenant: String,
    pub group_name: String,
    pub offsets: HashMap<String, u64>,
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
        .storage_driver_manager
        .offset_manager
        .get_offset(&params.tenant, &params.group_name)
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
        .storage_driver_manager
        .offset_manager
        .commit_offset(&params.tenant, &params.group_name, &params.offsets)
        .await
    {
        return error_response(e.to_string());
    }

    success_response("success")
}
