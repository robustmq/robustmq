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
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum WebhookHttpMethod {
    #[default]
    Post,
    Put,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum WebhookAuthType {
    #[default]
    None,
    Basic,
    Bearer,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct WebhookConnectorConfig {
    pub url: String,
    #[serde(default)]
    pub method: WebhookHttpMethod,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,
    #[serde(default)]
    pub auth_type: WebhookAuthType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bearer_token: Option<String>,
}

fn default_timeout_ms() -> u64 {
    5000
}

impl WebhookConnectorConfig {
    pub fn validate(&self) -> Result<(), common_base::error::common::CommonError> {
        use common_base::error::common::CommonError;

        if self.url.is_empty() {
            return Err(CommonError::CommonError("url cannot be empty".to_string()));
        }

        if self.url.len() > 2048 {
            return Err(CommonError::CommonError(
                "url length cannot exceed 2048 characters".to_string(),
            ));
        }

        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err(CommonError::CommonError(
                "url must start with http:// or https://".to_string(),
            ));
        }

        if self.timeout_ms == 0 || self.timeout_ms > 60000 {
            return Err(CommonError::CommonError(
                "timeout_ms must be between 1 and 60000".to_string(),
            ));
        }

        match self.auth_type {
            WebhookAuthType::Basic => {
                if self.username.is_none() || self.password.is_none() {
                    return Err(CommonError::CommonError(
                        "username and password are required for Basic auth".to_string(),
                    ));
                }
            }
            WebhookAuthType::Bearer => {
                if self.bearer_token.is_none() {
                    return Err(CommonError::CommonError(
                        "bearer_token is required for Bearer auth".to_string(),
                    ));
                }
            }
            WebhookAuthType::None => {}
        }

        Ok(())
    }
}
