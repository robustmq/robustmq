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

pub mod fastembed;

use crate::client::{LLMClient, LLMResult};
use common_base::error::common::CommonError;
use common_config::config::BrokerConfig;

fn err(msg: impl Into<String>) -> Box<CommonError> {
    Box::new(CommonError::CommonError(msg.into()))
}

pub async fn embed(text: &str, config: &BrokerConfig) -> LLMResult<Vec<f32>> {
    let llm = &config.llm_client;
    match llm.embedding.as_deref().unwrap_or_default() {
        "fastembed" => fastembed::embed(text).await,
        "api" => LLMClient::new(llm.clone())?.embed(text).await,
        other => Err(err(format!("unknown embedding type '{other}'"))),
    }
}

pub async fn embed_batch(texts: Vec<String>, config: &BrokerConfig) -> LLMResult<Vec<Vec<f32>>> {
    let llm = &config.llm_client;
    match llm.embedding.as_deref().unwrap_or_default() {
        "fastembed" => fastembed::embed_batch(texts).await,
        "api" => LLMClient::new(llm.clone())?.embed_batch(texts).await,
        other => Err(err(format!("unknown embedding type '{other}'"))),
    }
}
