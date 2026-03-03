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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MqttProtocolVersion {
    #[default]
    V5,
    V4,
    V3,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct MqttBridgeConnectorConfig {
    pub server: String,
    #[serde(default)]
    pub client_id_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default)]
    pub protocol_version: MqttProtocolVersion,
    #[serde(default = "default_keepalive_secs")]
    pub keepalive_secs: u64,
    #[serde(default = "default_connect_timeout_secs")]
    pub connect_timeout_secs: u64,
    #[serde(default)]
    pub enable_tls: bool,
    pub topic_prefix: Option<String>,
    #[serde(default = "default_qos")]
    pub qos: i32,
    #[serde(default)]
    pub retain: bool,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_keepalive_secs() -> u64 {
    60
}

fn default_connect_timeout_secs() -> u64 {
    10
}

fn default_qos() -> i32 {
    1
}

fn default_max_retries() -> u32 {
    3
}

impl MqttBridgeConnectorConfig {
    pub fn validate(&self) -> Result<(), common_base::error::common::CommonError> {
        use common_base::error::common::CommonError;

        if self.server.is_empty() {
            return Err(CommonError::CommonError(
                "server cannot be empty".to_string(),
            ));
        }

        if self.server.len() > 512 {
            return Err(CommonError::CommonError(
                "server length cannot exceed 512 characters".to_string(),
            ));
        }

        if !(0..=2).contains(&self.qos) {
            return Err(CommonError::CommonError(
                "qos must be 0, 1 or 2".to_string(),
            ));
        }

        if self.keepalive_secs == 0 || self.keepalive_secs > 65535 {
            return Err(CommonError::CommonError(
                "keepalive_secs must be between 1 and 65535".to_string(),
            ));
        }

        if self.connect_timeout_secs == 0 || self.connect_timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "connect_timeout_secs must be between 1 and 300".to_string(),
            ));
        }

        if self.max_retries > 10 {
            return Err(CommonError::CommonError(
                "max_retries cannot exceed 10".to_string(),
            ));
        }

        if let Some(prefix) = &self.client_id_prefix {
            if prefix.len() > 64 {
                return Err(CommonError::CommonError(
                    "client_id_prefix length cannot exceed 64 characters".to_string(),
                ));
            }
        }

        Ok(())
    }
}
