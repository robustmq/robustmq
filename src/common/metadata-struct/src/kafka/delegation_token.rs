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

use common_base::error::common::CommonError;
use serde::{Deserialize, Serialize};

// A Kafka SASL principal: (principal_type, principal_name), e.g. ("User", "alice").
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
pub struct KafkaTokenPrincipal {
    pub principal_type: String,
    pub principal_name: String,
}

// Metadata-only record for one delegation token (KIP-48). `hmac` and the
// cluster secret key used to derive it are the actual authentication
// material; nothing in RobustMQ verifies a token against `hmac` yet — SASL
// authentication (including delegation-token-based re-auth) is a separate,
// not-yet-implemented effort. This struct exists to support
// create/renew/expire/describe as metadata management.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct KafkaDelegationToken {
    pub tenant: String,
    pub token_id: String,
    pub hmac: Vec<u8>,
    pub owner: KafkaTokenPrincipal,
    pub token_requester: KafkaTokenPrincipal,
    pub renewers: Vec<KafkaTokenPrincipal>,
    pub issue_timestamp_ms: i64,
    pub expiry_timestamp_ms: i64,
    pub max_timestamp_ms: i64,
}

impl KafkaDelegationToken {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        Ok(serde_json::from_slice(data)?)
    }
}
