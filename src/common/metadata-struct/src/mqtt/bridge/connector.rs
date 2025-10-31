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

use serde::{Deserialize, Serialize};

use super::{connector_type::ConnectorType, status::MQTTStatus};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MQTTConnector {
    pub cluster_name: String,
    pub connector_name: String,
    pub connector_type: ConnectorType,
    pub config: String,
    pub failure_strategy: FailureHandlingStrategy,
    pub topic_name: String,
    pub status: MQTTStatus,
    pub broker_id: Option<u64>,
    pub create_time: u64,
    pub update_time: u64,
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
pub enum FailureHandlingStrategy {
    #[default]
    Discard,
    DiscardAfterRetry(DiscardAfterRetryStrategy),
    DeadMessageQueue(DeadMessageQueueStrategy),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DiscardAfterRetryStrategy {
    pub retry_total_times: u32,
    pub wait_time_ms: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DeadMessageQueueStrategy {
    pub topic_name: String,
}

impl MQTTConnector {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }

    pub fn decode(data: &[u8]) -> Self {
        serde_json::from_slice(data).unwrap()
    }
}
