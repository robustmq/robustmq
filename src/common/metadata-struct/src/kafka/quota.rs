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

use std::collections::HashMap;

use common_base::error::common::CommonError;
use serde::{Deserialize, Serialize};

pub const QUOTA_ENTITY_CLIENT_ID: &str = "client-id";
pub const QUOTA_KEY_PRODUCER_BYTE_RATE: &str = "producer_byte_rate";
pub const QUOTA_KEY_CONSUMER_BYTE_RATE: &str = "consumer_byte_rate";

// Sentinel path segment for the entity-type default quota (entity_name = None).
pub const QUOTA_DEFAULT_NAME: &str = "__default__";

// One Kafka client-quota entity and all quota values set on it. Quotas are
// per-broker limits: every broker enforces the configured value independently.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct KafkaClientQuota {
    pub tenant: String,
    pub entity_type: String,
    // None means "the default quota for this entity type".
    pub entity_name: Option<String>,
    // quota key -> value, e.g. "producer_byte_rate" -> 1048576.0
    pub quotas: HashMap<String, f64>,
}

impl KafkaClientQuota {
    pub fn name_key(&self) -> &str {
        self.entity_name.as_deref().unwrap_or(QUOTA_DEFAULT_NAME)
    }

    pub fn entity_key(&self) -> String {
        format!("{}/{}", self.entity_type, self.name_key())
    }

    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        Ok(serde_json::from_slice(data)?)
    }
}
