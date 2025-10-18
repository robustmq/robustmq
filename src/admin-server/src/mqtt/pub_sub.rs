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
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishReq {
    pub topic: String,
    pub payload: String,
    pub qos: Option<u8>,
    pub retain: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubscribeReq {
    pub topic: String,
    pub qos: Option<u8>,
}

pub async fn publish(
    State(_state): State<Arc<HttpState>>,
    Json(_params): Json<PublishReq>,
) -> String {
    // TODO: Implement publish logic
    success_response("success")
}

pub async fn subscribe(
    State(_state): State<Arc<HttpState>>,
    Json(_params): Json<SubscribeReq>,
) -> String {
    // TODO: Implement subscribe logic
    success_response("success")
}
