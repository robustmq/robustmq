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

use common_base::{error::common::CommonError, utils::serialize};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct MQ9Email {
    /// Unique mailbox identifier. System-generated UUID for private mailboxes;
    /// user-defined name for public mailboxes.
    pub mail_address: String,
    pub tenant: String,
    pub desc: String,
    /// Whether this mailbox is registered to PUBLIC.LIST.
    pub public: bool,
    /// Lifetime in seconds. 0 means no expiry.
    pub ttl: u64,
    pub create_time: u64,
}

impl MQ9Email {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}
