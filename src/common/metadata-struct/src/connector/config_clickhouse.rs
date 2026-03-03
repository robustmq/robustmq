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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct ClickHouseConnectorConfig {
    pub url: String,
    pub database: String,
    pub table: String,

    #[serde(default)]
    pub username: String,

    #[serde(default)]
    pub password: String,

    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_pool_size() -> u32 {
    8
}

fn default_timeout_secs() -> u64 {
    15
}

impl Default for ClickHouseConnectorConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            database: String::new(),
            table: String::new(),
            username: String::new(),
            password: String::new(),
            pool_size: default_pool_size(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

impl ClickHouseConnectorConfig {
    pub fn validate(&self) -> Result<(), CommonError> {
        if self.url.is_empty() {
            return Err(CommonError::CommonError("url cannot be empty".to_string()));
        }

        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err(CommonError::CommonError(
                "url must start with http:// or https://".to_string(),
            ));
        }

        if self.url.len() > 1024 {
            return Err(CommonError::CommonError(
                "url length cannot exceed 1024 characters".to_string(),
            ));
        }

        if self.database.is_empty() {
            return Err(CommonError::CommonError(
                "database cannot be empty".to_string(),
            ));
        }

        if self.database.len() > 256 {
            return Err(CommonError::CommonError(
                "database length cannot exceed 256 characters".to_string(),
            ));
        }

        if self.table.is_empty() {
            return Err(CommonError::CommonError(
                "table cannot be empty".to_string(),
            ));
        }

        if self.table.len() > 256 {
            return Err(CommonError::CommonError(
                "table length cannot exceed 256 characters".to_string(),
            ));
        }

        if self.timeout_secs == 0 || self.timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "timeout_secs must be between 1 and 300".to_string(),
            ));
        }

        if self.pool_size == 0 || self.pool_size > 64 {
            return Err(CommonError::CommonError(
                "pool_size must be between 1 and 64".to_string(),
            ));
        }

        Ok(())
    }
}
