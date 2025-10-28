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

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MySQLConnectorConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub table: String,
    pub sql_template: Option<String>,
    pub pool_size: Option<u32>,
    pub enable_batch_insert: Option<bool>,
    pub enable_upsert: Option<bool>,
    pub conflict_columns: Option<String>,
}

impl MySQLConnectorConfig {
    pub fn connection_url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }

    pub fn get_pool_size(&self) -> u32 {
        self.pool_size.unwrap_or(10)
    }

    pub fn is_batch_insert_enabled(&self) -> bool {
        self.enable_batch_insert.unwrap_or(false)
    }

    pub fn is_upsert_enabled(&self) -> bool {
        self.enable_upsert.unwrap_or(false)
    }

    pub fn validate(&self) -> Result<(), CommonError> {
        if self.host.is_empty() {
            return Err(CommonError::CommonError("host cannot be empty".to_string()));
        }

        if self.host.len() > 512 {
            return Err(CommonError::CommonError(
                "host length cannot exceed 512 characters".to_string(),
            ));
        }

        if self.port == 0 {
            return Err(CommonError::CommonError(
                "port must be greater than 0".to_string(),
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

        if self.username.is_empty() {
            return Err(CommonError::CommonError(
                "username cannot be empty".to_string(),
            ));
        }

        if self.username.len() > 256 {
            return Err(CommonError::CommonError(
                "username length cannot exceed 256 characters".to_string(),
            ));
        }

        if self.password.len() > 256 {
            return Err(CommonError::CommonError(
                "password length cannot exceed 256 characters".to_string(),
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

        if let Some(sql) = &self.sql_template {
            if sql.len() > 4096 {
                return Err(CommonError::CommonError(
                    "sql_template length cannot exceed 4096 characters".to_string(),
                ));
            }
        }

        if let Some(size) = self.pool_size {
            if size == 0 || size > 1000 {
                return Err(CommonError::CommonError(
                    "pool_size must be between 1 and 1000".to_string(),
                ));
            }
        }

        if self.enable_upsert.unwrap_or(false) {
            if let Some(columns) = &self.conflict_columns {
                if columns.is_empty() {
                    return Err(CommonError::CommonError(
                        "conflict_columns cannot be empty when upsert is enabled".to_string(),
                    ));
                }
            } else {
                return Err(CommonError::CommonError(
                    "conflict_columns must be provided when upsert is enabled".to_string(),
                ));
            }
        }

        Ok(())
    }
}
