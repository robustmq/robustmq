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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MongoDBDeploymentMode {
    #[default]
    Single,
    ReplicaSet,
    Sharded,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MongoDBConnectorConfig {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub database: String,
    pub collection: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_source: Option<String>,
    #[serde(default)]
    pub deployment_mode: MongoDBDeploymentMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replica_set_name: Option<String>,
    #[serde(default)]
    pub enable_tls: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pool_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_pool_size: Option<u32>,

    #[serde(default = "default_connect_timeout_secs")]
    pub connect_timeout_secs: u64,
    #[serde(default = "default_server_selection_timeout_secs")]
    pub server_selection_timeout_secs: u64,
    #[serde(default = "default_socket_timeout_secs")]
    pub socket_timeout_secs: u64,

    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_ordered_insert")]
    pub ordered_insert: bool,

    #[serde(default = "default_w")]
    pub w: String,
}

fn default_port() -> u16 {
    27017
}

fn default_connect_timeout_secs() -> u64 {
    10
}

fn default_server_selection_timeout_secs() -> u64 {
    30
}

fn default_socket_timeout_secs() -> u64 {
    60
}

fn default_batch_size() -> usize {
    100
}

fn default_ordered_insert() -> bool {
    false
}

fn default_w() -> String {
    "1".to_string()
}

impl Default for MongoDBConnectorConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: default_port(),
            database: String::new(),
            collection: String::new(),
            username: None,
            password: None,
            auth_source: None,
            deployment_mode: MongoDBDeploymentMode::Single,
            replica_set_name: None,
            enable_tls: false,
            max_pool_size: None,
            min_pool_size: None,
            connect_timeout_secs: default_connect_timeout_secs(),
            server_selection_timeout_secs: default_server_selection_timeout_secs(),
            socket_timeout_secs: default_socket_timeout_secs(),
            batch_size: default_batch_size(),
            ordered_insert: default_ordered_insert(),
            w: default_w(),
        }
    }
}

impl MongoDBConnectorConfig {
    pub fn build_connection_uri(&self) -> String {
        let mut uri = String::from("mongodb://");

        if let (Some(username), Some(password)) = (&self.username, &self.password) {
            uri.push_str(&format!("{}:{}@", username, password));
        }

        uri.push_str(&format!("{}:{}", self.host, self.port));
        uri.push_str(&format!("/{}", self.database));

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

    pub fn validate(&self) -> Result<(), common_base::error::common::CommonError> {
        use common_base::error::common::CommonError;

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

        if self.collection.is_empty() {
            return Err(CommonError::CommonError(
                "collection cannot be empty".to_string(),
            ));
        }

        if self.collection.len() > 256 {
            return Err(CommonError::CommonError(
                "collection length cannot exceed 256 characters".to_string(),
            ));
        }

        if let Some(username) = &self.username {
            if username.len() > 256 {
                return Err(CommonError::CommonError(
                    "username length cannot exceed 256 characters".to_string(),
                ));
            }
        }

        if let Some(password) = &self.password {
            if password.len() > 256 {
                return Err(CommonError::CommonError(
                    "password length cannot exceed 256 characters".to_string(),
                ));
            }
        }

        if let Some(auth_source) = &self.auth_source {
            if auth_source.len() > 256 {
                return Err(CommonError::CommonError(
                    "auth_source length cannot exceed 256 characters".to_string(),
                ));
            }
        }

        if self.deployment_mode == MongoDBDeploymentMode::ReplicaSet {
            if let Some(replica_set) = &self.replica_set_name {
                if replica_set.is_empty() {
                    return Err(CommonError::CommonError(
                        "replica_set_name cannot be empty for ReplicaSet deployment mode"
                            .to_string(),
                    ));
                }
            } else {
                return Err(CommonError::CommonError(
                    "replica_set_name must be provided for ReplicaSet deployment mode".to_string(),
                ));
            }
        }

        if let Some(max_pool) = self.max_pool_size {
            if max_pool == 0 || max_pool > 1000 {
                return Err(CommonError::CommonError(
                    "max_pool_size must be between 1 and 1000".to_string(),
                ));
            }
        }

        if let Some(min_pool) = self.min_pool_size {
            if let Some(max_pool) = self.max_pool_size {
                if min_pool > max_pool {
                    return Err(CommonError::CommonError(
                        "min_pool_size cannot be greater than max_pool_size".to_string(),
                    ));
                }
            }
        }

        if self.connect_timeout_secs == 0 || self.connect_timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "connect_timeout_secs must be between 1 and 300 seconds".to_string(),
            ));
        }

        if self.server_selection_timeout_secs == 0 || self.server_selection_timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "server_selection_timeout_secs must be between 1 and 300 seconds".to_string(),
            ));
        }

        if self.socket_timeout_secs == 0 || self.socket_timeout_secs > 600 {
            return Err(CommonError::CommonError(
                "socket_timeout_secs must be between 1 and 600 seconds".to_string(),
            ));
        }

        if self.batch_size == 0 || self.batch_size > 10000 {
            return Err(CommonError::CommonError(
                "batch_size must be between 1 and 10000".to_string(),
            ));
        }

        let valid_w_values = ["0", "1", "majority"];
        if !valid_w_values.contains(&self.w.as_str()) {
            let w_num: Result<u32, _> = self.w.parse();
            if w_num.is_err() || w_num.unwrap() > 10 {
                return Err(CommonError::CommonError(
                    "w must be '0', '1', 'majority', or a number between 2 and 10".to_string(),
                ));
            }
        }

        Ok(())
    }
}
