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
pub struct KafkaConnectorConfig {
    pub bootstrap_servers: String,
    pub topic: String,
    pub key: String,
}

impl KafkaConnectorConfig {
    pub fn validate(&self) -> Result<(), CommonError> {
        if self.bootstrap_servers.is_empty() {
            return Err(CommonError::CommonError(
                "bootstrap_servers cannot be empty".to_string(),
            ));
        }

        if self.bootstrap_servers.len() > 1024 {
            return Err(CommonError::CommonError(
                "bootstrap_servers length cannot exceed 1024 characters".to_string(),
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

        if self.key.len() > 256 {
            return Err(CommonError::CommonError(
                "key length cannot exceed 256 characters".to_string(),
            ));
        }

        Ok(())
    }
}
