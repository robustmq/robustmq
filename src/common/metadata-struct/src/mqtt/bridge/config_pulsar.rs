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

fn default_connection_timeout_secs() -> u64 {
    30
}

fn default_operation_timeout_secs() -> u64 {
    30
}

fn default_batch_size() -> usize {
    100
}

fn default_max_pending_messages() -> u32 {
    1000
}

fn default_send_timeout_secs() -> u64 {
    30
}

impl Default for PulsarConnectorConfig {
    fn default() -> Self {
        Self {
            server: String::new(),
            topic: String::new(),
            token: None,
            oauth: None,
            basic_name: None,
            basic_password: None,
            connection_timeout_secs: default_connection_timeout_secs(),
            operation_timeout_secs: default_operation_timeout_secs(),
            batch_size: default_batch_size(),
            max_pending_messages: default_max_pending_messages(),
            send_timeout_secs: default_send_timeout_secs(),
            compression: None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PulsarConnectorConfig {
    pub server: String,
    pub topic: String,
    pub token: Option<String>,
    pub oauth: Option<String>,
    pub basic_name: Option<String>,
    pub basic_password: Option<String>,

    #[serde(default = "default_connection_timeout_secs")]
    pub connection_timeout_secs: u64,
    #[serde(default = "default_operation_timeout_secs")]
    pub operation_timeout_secs: u64,

    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_max_pending_messages")]
    pub max_pending_messages: u32,
    #[serde(default = "default_send_timeout_secs")]
    pub send_timeout_secs: u64,

    pub compression: Option<String>,
}

impl PulsarConnectorConfig {
    pub fn new(server: String, topic: String) -> Self {
        PulsarConnectorConfig {
            server,
            topic,
            ..Default::default()
        }
    }

    pub fn with_token(&mut self, token: String) -> &mut Self {
        self.token = Some(token);
        self
    }

    pub fn with_oauth(&mut self, oauth: String) -> &mut Self {
        self.oauth = Some(oauth);
        self
    }

    pub fn with_basic(&mut self, name: String, password: String) -> &mut Self {
        self.basic_name = Some(name);
        self.basic_password = Some(password);
        self
    }

    pub fn validate(&self) -> Result<(), CommonError> {
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

        if self.topic.is_empty() {
            return Err(CommonError::CommonError(
                "topic cannot be empty".to_string(),
            ));
        }

        if self.topic.len() > 256 {
            return Err(CommonError::CommonError(
                "topic length cannot exceed 256 characters".to_string(),
            ));
        }

        if let Some(token) = &self.token {
            if token.len() > 1024 {
                return Err(CommonError::CommonError(
                    "token length cannot exceed 1024 characters".to_string(),
                ));
            }
        }

        if let Some(oauth) = &self.oauth {
            if oauth.len() > 1024 {
                return Err(CommonError::CommonError(
                    "oauth length cannot exceed 1024 characters".to_string(),
                ));
            }
        }

        if let Some(name) = &self.basic_name {
            if name.len() > 256 {
                return Err(CommonError::CommonError(
                    "basic_name length cannot exceed 256 characters".to_string(),
                ));
            }
        }

        if let Some(password) = &self.basic_password {
            if password.len() > 256 {
                return Err(CommonError::CommonError(
                    "basic_password length cannot exceed 256 characters".to_string(),
                ));
            }
        }

        let auth_count = [
            self.token.is_some(),
            self.oauth.is_some(),
            self.basic_name.is_some() || self.basic_password.is_some(),
        ]
        .iter()
        .filter(|&&x| x)
        .count();

        if auth_count > 1 {
            return Err(CommonError::CommonError(
                "Only one authentication method can be specified (token, oauth, or basic)"
                    .to_string(),
            ));
        }

        if self.basic_name.is_some() != self.basic_password.is_some() {
            return Err(CommonError::CommonError(
                "basic_name and basic_password must be provided together".to_string(),
            ));
        }

        if self.connection_timeout_secs == 0 || self.connection_timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "connection_timeout_secs must be between 1 and 300 seconds".to_string(),
            ));
        }

        if self.operation_timeout_secs == 0 || self.operation_timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "operation_timeout_secs must be between 1 and 300 seconds".to_string(),
            ));
        }

        if self.send_timeout_secs == 0 || self.send_timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "send_timeout_secs must be between 1 and 300 seconds".to_string(),
            ));
        }

        if self.batch_size == 0 || self.batch_size > 10000 {
            return Err(CommonError::CommonError(
                "batch_size must be between 1 and 10000".to_string(),
            ));
        }

        if self.max_pending_messages == 0 || self.max_pending_messages > 100000 {
            return Err(CommonError::CommonError(
                "max_pending_messages must be between 1 and 100000".to_string(),
            ));
        }

        if let Some(compression) = &self.compression {
            let valid_compressions = ["none", "lz4", "zlib", "zstd", "snappy"];
            if !valid_compressions.contains(&compression.to_lowercase().as_str()) {
                return Err(CommonError::CommonError(format!(
                    "compression must be one of: none, lz4, zlib, zstd, snappy (got: {})",
                    compression
                )));
            }
        }

        Ok(())
    }
}
