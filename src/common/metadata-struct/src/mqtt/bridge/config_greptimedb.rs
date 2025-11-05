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
use std::fmt::Display;

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct GreptimeDBConnectorConfig {
    pub server_addr: String,
    pub database: String,
    pub user: String,
    pub password: String,
    pub precision: TimePrecision,
}

impl GreptimeDBConnectorConfig {
    pub fn new(server_addr: String, database: String, user: String, password: String) -> Self {
        GreptimeDBConnectorConfig {
            server_addr,
            database,
            user,
            password,
            precision: TimePrecision::Second,
        }
    }

    pub fn validate(&self) -> Result<(), CommonError> {
        if self.server_addr.is_empty() {
            return Err(CommonError::CommonError(
                "server_addr cannot be empty".to_string(),
            ));
        }

        if self.server_addr.len() > 512 {
            return Err(CommonError::CommonError(
                "server_addr length cannot exceed 512 characters".to_string(),
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

        if self.user.is_empty() {
            return Err(CommonError::CommonError("user cannot be empty".to_string()));
        }

        if self.user.len() > 256 {
            return Err(CommonError::CommonError(
                "user length cannot exceed 256 characters".to_string(),
            ));
        }

        if self.password.len() > 256 {
            return Err(CommonError::CommonError(
                "password length cannot exceed 256 characters".to_string(),
            ));
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub enum TimePrecision {
    #[default]
    Second,
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl Display for TimePrecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimePrecision::Second => write!(f, "s"),
            TimePrecision::Millisecond => write!(f, "ms"),
            TimePrecision::Microsecond => write!(f, "us"),
            TimePrecision::Nanosecond => write!(f, "ns"),
        }
    }
}
