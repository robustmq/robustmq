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

use crate::{extractor::ValidatedJson, state::HttpState};
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PublishReq {
    #[validate(length(min = 1, max = 256, message = "Topic length must be between 1-256"))]
    pub topic: String,

    #[validate(length(
        max = 1048576,
        message = "Payload length cannot exceed 1MB (1048576 bytes)"
    ))]
    pub payload: String,

    pub retain: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadReq {
    pub topic: String,
    pub offset: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SendMessageResp {
    pub offsets: Vec<u64>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ReadMessageResp {
    pub messages: Vec<ReadMessageRow>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ReadMessageRow {
    pub offset: u64,
    pub content: String,
    pub timestamp: u64,
}

use bytes::Bytes;
use common_base::{
    error::common::CommonError,
    http_response::{error_response, success_response},
    tools::now_second,
};
use common_config::broker::broker_config;
use metadata_struct::mqtt::message::MqttMessage;
use mqtt_broker::{
    handler::{retain::save_retain_message, topic::try_init_topic},
    storage::message::MessageStorage,
};
use protocol::mqtt::common::{Publish, PublishProperties};
use std::sync::Arc;

pub async fn send(
    State(state): State<Arc<HttpState>>,
    ValidatedJson(params): ValidatedJson<PublishReq>,
) -> String {
    match send_inner(state, params).await {
        Ok(offsets) => success_response(SendMessageResp { offsets }),
        Err(e) => error_response(e.to_string()),
    }
}

async fn send_inner(state: Arc<HttpState>, params: PublishReq) -> Result<Vec<u64>, CommonError> {
    let message_storage = MessageStorage::new(state.storage_driver_manager.clone());
    let config = broker_config();
    let client_id = format!("{}_{}", config.cluster_name, config.broker_id);

    if let Err(e) = try_init_topic(
        &params.topic,
        &state.mqtt_context.cache_manager,
        &state.storage_driver_manager,
        &state.client_pool,
    )
    .await
    {
        return Err(CommonError::CommonError(e.to_string()));
    }

    let publish = Publish {
        dup: false,
        qos: protocol::mqtt::common::QoS::AtLeastOnce,
        p_kid: 0,
        retain: params.retain,
        topic: Bytes::from(params.topic.clone()),
        payload: Bytes::from(params.payload.clone()),
    };

    let publish_properties = Some(PublishProperties::default());

    if params.retain {
        if let Err(e) = save_retain_message(
            &state.mqtt_context.cache_manager,
            &state.client_pool,
            params.topic.clone(),
            &client_id,
            &publish,
            &publish_properties,
        )
        .await
        {
            return Err(CommonError::CommonError(e.to_string()));
        }
    }

    let mut offset = Vec::new();
    let message_expire = now_second() + 3600;
    if let Some(record) =
        MqttMessage::build_record(&client_id, &publish, &publish_properties, message_expire)
    {
        offset = message_storage
            .append_topic_message(&params.topic.clone(), vec![record])
            .await?;
    }

    Ok(offset)
}

pub async fn read(State(state): State<Arc<HttpState>>, Json(params): Json<ReadReq>) -> String {
    match read_inner(state, params).await {
        Ok(messages) => success_response(ReadMessageResp { messages }),
        Err(e) => error_response(e.to_string()),
    }
}

pub async fn read_inner(
    state: Arc<HttpState>,
    params: ReadReq,
) -> Result<Vec<ReadMessageRow>, CommonError> {
    let message_storage = MessageStorage::new(state.storage_driver_manager.clone());
    let mut results = Vec::new();
    let data = message_storage
        .read_topic_message(&params.topic, params.offset, 100)
        .await?;

    for row in data {
        let message = MqttMessage::decode(&row.data)?;
        let content = String::from_utf8_lossy(&message.payload).to_string();
        results.push(ReadMessageRow {
            offset: row.pkid,
            content,
            timestamp: row.timestamp,
        });
    }

    Ok(results)
}
