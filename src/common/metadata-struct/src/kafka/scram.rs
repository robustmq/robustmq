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

pub const SCRAM_MECHANISM_SHA_256: i8 = 1;
pub const SCRAM_MECHANISM_SHA_512: i8 = 2;
pub const SCRAM_MIN_ITERATIONS: i32 = 4096;

// One SCRAM credential for a (user, mechanism) pair. Only the RFC 5802 derived
// keys are persisted: StoredKey = H(HMAC(salted_password, "Client Key")) and
// ServerKey = HMAC(salted_password, "Server Key"). The salted password itself
// is discarded after derivation and plaintext passwords never reach the broker.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct KafkaScramCredential {
    pub tenant: String,
    pub user: String,
    pub mechanism: i8,
    pub iterations: i32,
    pub salt: Vec<u8>,
    pub stored_key: Vec<u8>,
    pub server_key: Vec<u8>,
}

impl KafkaScramCredential {
    pub fn entity_key(&self) -> String {
        format!("{}/{}", self.user, self.mechanism)
    }

    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        Ok(serde_json::to_vec(&self)?)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        Ok(serde_json::from_slice(data)?)
    }
}
