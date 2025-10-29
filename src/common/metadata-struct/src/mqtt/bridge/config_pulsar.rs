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

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct PulsarConnectorConfig {
    pub server: String,
    pub topic: String,
    pub token: Option<String>,
    pub oauth: Option<String>,
    pub basic_name: Option<String>,
    pub basic_password: Option<String>,
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

        Ok(())
    }
}
