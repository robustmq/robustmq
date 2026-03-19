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

pub const DEFAULT_TENANT: &str = "default";

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Tenant {
    pub tenant_name: String,
    pub desc: String,
    pub config: TenantConfig,
    pub create_time: u64,
}

impl Tenant {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TenantConfig {
    pub max_connections_per_node: u64,
    pub max_create_connection_rate_per_second: u32,
    pub max_topics: u64,
    pub max_sessions: u64,
    pub max_publish_rate: u32,
}

impl TenantConfig {
    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}

impl Default for TenantConfig {
    fn default() -> Self {
        TenantConfig {
            max_connections_per_node: 10000000,
            max_create_connection_rate_per_second: 10000,
            max_topics: 5000000,
            max_sessions: 50000000,
            max_publish_rate: 10000,
        }
    }
}
