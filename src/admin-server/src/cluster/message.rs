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

use crate::{state::HttpState, tool::extractor::ValidatedJson};
use axum::{extract::State, Json};
use bytes::Bytes;
use common_base::{
    error::common::CommonError,
    http_response::{error_response, success_response},
};
use metadata_struct::storage::adapter_record::AdapterWriteRecord;
use mqtt_broker::{core::topic::try_init_topic, storage::message::MessageStorage};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SendMessageReq {
    #[validate(length(min = 1, max = 256, message = "Tenant length must be between 1-256"))]
    pub tenant: String,

    #[validate(length(min = 1, max = 256, message = "Topic length must be between 1-256"))]
    pub topic: String,

    #[validate(length(
        max = 1048576,
        message = "Payload length cannot exceed 1MB (1048576 bytes)"
    ))]
    pub payload: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadMessageReq {
    pub tenant: String,
    pub topic: String,
    pub offset: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SendMessageResp {
    pub offsets: Vec<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadMessageResp {
    pub messages: Vec<ReadMessageRow>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReadMessageRow {
    pub offset: u64,
    pub content: String,
    pub timestamp: u64,
}

pub async fn send_message(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<SendMessageReq>,
) -> String {
    match send_message_inner(state, params).await {
        Ok(offsets) => success_response(SendMessageResp { offsets }),
        Err(e) => error_response(e.to_string()),
    }
}

async fn send_message_inner(
    state: Arc<HttpState>,
    params: SendMessageReq,
) -> Result<Vec<u64>, CommonError> {
    let message_storage = MessageStorage::new(state.storage_driver_manager.clone());

    if let Err(e) = try_init_topic(
        &params.tenant,
        &params.topic,
        &state.mqtt_context.cache_manager,
        &state.storage_driver_manager,
        &state.client_pool,
    )
    .await
    {
        return Err(CommonError::CommonError(e.to_string()));
    }

    let record = AdapterWriteRecord::new(params.topic.clone(), Bytes::from(params.payload));
    let offset = message_storage
        .append_topic_message(&params.tenant, &params.topic, vec![record])
        .await?;

    Ok(offset)
}

pub async fn read_message(
    State(state): State<Arc<HttpState>>,
    Json(params): Json<ReadMessageReq>,
) -> String {
    match read_message_inner(state, params).await {
        Ok(messages) => success_response(ReadMessageResp { messages }),
        Err(e) => error_response(e.to_string()),
    }
}

async fn read_message_inner(
    state: Arc<HttpState>,
    params: ReadMessageReq,
) -> Result<Vec<ReadMessageRow>, CommonError> {
    let shard_info = state
        .storage_driver_manager
        .list_storage_resource(&params.tenant, &params.topic)
        .await?;
    let detail = shard_info.get(&0).unwrap();

    let offset = if params.offset < detail.offset.start_offset {
        detail.offset.start_offset
    } else if params.offset > detail.offset.end_offset {
        detail.offset.end_offset
    } else {
        params.offset
    };

    let message_storage = MessageStorage::new(state.storage_driver_manager.clone());
    let mut offsets = HashMap::new();
    offsets.insert(params.topic.clone(), offset);

    let data = message_storage
        .read_topic_message(&params.tenant, &params.topic, &offsets, 100)
        .await?;

    let mut results = Vec::new();
    for row in data {
        let content = String::from_utf8_lossy(&row.data).to_string();
        results.push(ReadMessageRow {
            offset: row.metadata.offset,
            content,
            timestamp: row.metadata.create_t,
        });
    }

    Ok(results)
}
