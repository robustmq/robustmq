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

fn default_connect_timeout_secs() -> u64 {
    10
}

fn default_acquire_timeout_secs() -> u64 {
    30
}

fn default_idle_timeout_secs() -> u64 {
    600
}

fn default_max_lifetime_secs() -> u64 {
    1800
}

fn default_batch_size() -> usize {
    100
}

fn default_min_pool_size() -> u32 {
    2
}

impl Default for PostgresConnectorConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: 5432,
            database: String::new(),
            username: String::new(),
            password: String::new(),
            table: String::new(),
            sql_template: None,
            pool_size: None,
            enable_batch_insert: None,
            enable_upsert: None,
            conflict_columns: None,
            connect_timeout_secs: default_connect_timeout_secs(),
            acquire_timeout_secs: default_acquire_timeout_secs(),
            idle_timeout_secs: default_idle_timeout_secs(),
            max_lifetime_secs: default_max_lifetime_secs(),
            batch_size: default_batch_size(),
            min_pool_size: default_min_pool_size(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PostgresConnectorConfig {
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

    #[serde(default = "default_connect_timeout_secs")]
    pub connect_timeout_secs: u64,
    #[serde(default = "default_acquire_timeout_secs")]
    pub acquire_timeout_secs: u64,
    #[serde(default = "default_idle_timeout_secs")]
    pub idle_timeout_secs: u64,
    #[serde(default = "default_max_lifetime_secs")]
    pub max_lifetime_secs: u64,

    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_min_pool_size")]
    pub min_pool_size: u32,
}

impl PostgresConnectorConfig {
    pub fn connection_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
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

        if !self
            .table
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
        {
            return Err(CommonError::CommonError(
                "table name can only contain letters, numbers, underscores and dots (for schema.table)".to_string(),
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

        if self.connect_timeout_secs == 0 || self.connect_timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "connect_timeout_secs must be between 1 and 300 seconds".to_string(),
            ));
        }

        if self.acquire_timeout_secs == 0 || self.acquire_timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "acquire_timeout_secs must be between 1 and 300 seconds".to_string(),
            ));
        }

        if self.idle_timeout_secs > 3600 {
            return Err(CommonError::CommonError(
                "idle_timeout_secs cannot exceed 3600 seconds".to_string(),
            ));
        }

        if self.max_lifetime_secs > 7200 {
            return Err(CommonError::CommonError(
                "max_lifetime_secs cannot exceed 7200 seconds (2 hours)".to_string(),
            ));
        }

        if self.batch_size == 0 || self.batch_size > 10000 {
            return Err(CommonError::CommonError(
                "batch_size must be between 1 and 10000".to_string(),
            ));
        }

        if self.min_pool_size > self.pool_size.unwrap_or(10) {
            return Err(CommonError::CommonError(
                "min_pool_size cannot be greater than pool_size".to_string(),
            ));
        }

        if let Some(sql) = &self.sql_template {
            let placeholder_count = (1..=10)
                .filter(|i| sql.contains(&format!("${}", i)))
                .count();
            if placeholder_count != 5 {
                return Err(CommonError::CommonError(format!(
                    "sql_template must contain exactly 5 placeholders ($1-$5), found {}",
                    placeholder_count
                )));
            }

            if self.is_batch_insert_enabled() {
                return Err(CommonError::CommonError(
                    "sql_template cannot be used with batch insert mode. Please disable batch insert or remove sql_template.".to_string(),
                ));
            }
        }

        Ok(())
    }
}
