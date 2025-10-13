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
}
