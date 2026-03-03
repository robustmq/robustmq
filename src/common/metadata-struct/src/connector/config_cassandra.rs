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

fn default_port() -> u16 {
    9042
}

fn default_replication_factor() -> u32 {
    1
}

fn default_timeout_secs() -> u64 {
    15
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct CassandraConnectorConfig {
    pub nodes: Vec<String>,

    #[serde(default = "default_port")]
    pub port: u16,

    pub keyspace: String,
    pub table: String,

    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,

    #[serde(default = "default_replication_factor")]
    pub replication_factor: u32,

    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

impl Default for CassandraConnectorConfig {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            port: default_port(),
            keyspace: String::new(),
            table: String::new(),
            username: String::new(),
            password: String::new(),
            replication_factor: default_replication_factor(),
            timeout_secs: default_timeout_secs(),
        }
    }
}

impl CassandraConnectorConfig {
    pub fn validate(&self) -> Result<(), CommonError> {
        if self.nodes.is_empty() {
            return Err(CommonError::CommonError(
                "nodes cannot be empty, at least one Cassandra node is required".to_string(),
            ));
        }

        for node in &self.nodes {
            if node.is_empty() {
                return Err(CommonError::CommonError(
                    "node address cannot be empty".to_string(),
                ));
            }
        }

        if self.keyspace.is_empty() {
            return Err(CommonError::CommonError(
                "keyspace cannot be empty".to_string(),
            ));
        }

        if self.table.is_empty() {
            return Err(CommonError::CommonError(
                "table cannot be empty".to_string(),
            ));
        }

        if self.port == 0 {
            return Err(CommonError::CommonError(
                "port must be greater than 0".to_string(),
            ));
        }

        if self.timeout_secs == 0 || self.timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "timeout_secs must be between 1 and 300".to_string(),
            ));
        }

        Ok(())
    }

    pub fn known_nodes(&self) -> Vec<String> {
        self.nodes
            .iter()
            .map(|node| format!("{}:{}", node, self.port))
            .collect()
    }
}
