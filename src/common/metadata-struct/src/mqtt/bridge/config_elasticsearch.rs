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
pub enum ElasticsearchAuthType {
    #[default]
    None,
    Basic,
    ApiKey,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ElasticsearchConnectorConfig {
    pub url: String,
    pub index: String,
    #[serde(default)]
    pub auth_type: ElasticsearchAuthType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default)]
    pub enable_tls: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ca_cert_path: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

impl ElasticsearchConnectorConfig {
    pub fn validate(&self) -> Result<(), common_base::error::common::CommonError> {
        use common_base::error::common::CommonError;

        if self.url.is_empty() {
            return Err(CommonError::CommonError("url cannot be empty".to_string()));
        }

        if self.url.len() > 512 {
            return Err(CommonError::CommonError(
                "url length cannot exceed 512 characters".to_string(),
            ));
        }

        if self.index.is_empty() {
            return Err(CommonError::CommonError(
                "index cannot be empty".to_string(),
            ));
        }

        if self.index.len() > 256 {
            return Err(CommonError::CommonError(
                "index length cannot exceed 256 characters".to_string(),
            ));
        }

        match self.auth_type {
            ElasticsearchAuthType::Basic => {
                if self.username.is_none() || self.password.is_none() {
                    return Err(CommonError::CommonError(
                        "username and password are required for Basic auth".to_string(),
                    ));
                }
            }
            ElasticsearchAuthType::ApiKey => {
                if self.api_key.is_none() {
                    return Err(CommonError::CommonError(
                        "api_key is required for ApiKey auth".to_string(),
                    ));
                }
            }
            ElasticsearchAuthType::None => {}
        }

        if let Some(username) = &self.username {
            if username.len() > 256 {
                return Err(CommonError::CommonError(
                    "username length cannot exceed 256 characters".to_string(),
                ));
            }
        }

        if let Some(password) = &self.password {
            if password.len() > 256 {
                return Err(CommonError::CommonError(
                    "password length cannot exceed 256 characters".to_string(),
                ));
            }
        }

        if self.timeout_secs == 0 || self.timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "timeout_secs must be between 1 and 300".to_string(),
            ));
        }

        if self.max_retries > 10 {
            return Err(CommonError::CommonError(
                "max_retries cannot exceed 10".to_string(),
            ));
        }

        Ok(())
    }
}
