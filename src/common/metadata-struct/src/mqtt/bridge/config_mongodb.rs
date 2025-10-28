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

/// MongoDB deployment mode
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MongoDBDeploymentMode {
    /// Single standalone MongoDB instance
    #[default]
    Single,
    /// MongoDB replica set
    ReplicaSet,
    /// MongoDB sharded cluster
    Sharded,
}

/// MongoDB connector configuration
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MongoDBConnectorConfig {
    /// MongoDB server host
    pub host: String,

    /// MongoDB server port (default: 27017)
    #[serde(default = "default_port")]
    pub port: u16,

    /// Database name
    pub database: String,

    /// Collection name
    pub collection: String,

    /// Username for authentication (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Password for authentication (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Authentication source database (default: admin)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_source: Option<String>,

    /// Deployment mode
    #[serde(default)]
    pub deployment_mode: MongoDBDeploymentMode,

    /// Replica set name (required when deployment_mode is ReplicaSet)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replica_set_name: Option<String>,

    /// Enable TLS/SSL connection
    #[serde(default)]
    pub enable_tls: bool,

    /// Maximum pool size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pool_size: Option<u32>,

    /// Minimum pool size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_pool_size: Option<u32>,
}

fn default_port() -> u16 {
    27017
}

impl MongoDBConnectorConfig {
    /// Build MongoDB connection URI
    pub fn build_connection_uri(&self) -> String {
        let mut uri = String::from("mongodb://");

        // Add credentials if provided
        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            uri.push_str(&format!("{}:{}@", username, password));
        }

        // Add host and port
        uri.push_str(&format!("{}:{}", self.host, self.port));

        // Add database name
        uri.push_str(&format!("/{}", self.database));

        // Add query parameters
        let mut params = Vec::new();

        if let Some(auth_source) = &self.auth_source {
            params.push(format!("authSource={}", auth_source));
        }

        if self.enable_tls {
            params.push("tls=true".to_string());
        }

        if self.deployment_mode == MongoDBDeploymentMode::ReplicaSet {
            if let Some(replica_set) = &self.replica_set_name {
                params.push(format!("replicaSet={}", replica_set));
            }
        }

        if !params.is_empty() {
            uri.push('?');
            uri.push_str(&params.join("&"));
        }

        uri
    }
}
