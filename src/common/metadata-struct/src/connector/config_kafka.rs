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

use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

use common_base::error::common::CommonError;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KafkaConnectorConfig {
    pub bootstrap_servers: String,
    pub topic: String,

    #[serde(default)]
    pub key: String,

    #[serde(default = "default_compression_type")]
    pub compression_type: String,

    #[serde(default = "default_batch_size")]
    pub batch_size: u32,

    #[serde(default = "default_linger_ms")]
    pub linger_ms: u64,

    #[serde(default = "default_acks")]
    pub acks: String,

    #[serde(default = "default_retries")]
    pub retries: u32,

    #[serde(default = "default_message_timeout_ms")]
    pub message_timeout_ms: u64,

    #[serde(default = "default_cleanup_timeout_secs")]
    pub cleanup_timeout_secs: u64,
}

fn default_compression_type() -> String {
    "none".to_string()
}

fn default_batch_size() -> u32 {
    16384
}

fn default_linger_ms() -> u64 {
    5
}

fn default_acks() -> String {
    "1".to_string()
}

fn default_retries() -> u32 {
    3
}

fn default_message_timeout_ms() -> u64 {
    30000
}

fn default_cleanup_timeout_secs() -> u64 {
    10
}

impl Default for KafkaConnectorConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: String::new(),
            topic: String::new(),
            key: String::new(),
            compression_type: default_compression_type(),
            batch_size: default_batch_size(),
            linger_ms: default_linger_ms(),
            acks: default_acks(),
            retries: default_retries(),
            message_timeout_ms: default_message_timeout_ms(),
            cleanup_timeout_secs: default_cleanup_timeout_secs(),
        }
    }
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

        self.validate_bootstrap_servers_format()?;
        self.validate_bootstrap_servers_connectivity()?;

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

        let valid_compressions = ["none", "gzip", "snappy", "lz4", "zstd"];
        if !valid_compressions.contains(&self.compression_type.as_str()) {
            return Err(CommonError::CommonError(format!(
                "Invalid compression_type '{}', must be one of: none, gzip, snappy, lz4, zstd",
                self.compression_type
            )));
        }

        let valid_acks = ["0", "1", "all", "-1"];
        if !valid_acks.contains(&self.acks.as_str()) {
            return Err(CommonError::CommonError(format!(
                "Invalid acks '{}', must be one of: 0, 1, all, -1",
                self.acks
            )));
        }

        if self.batch_size == 0 || self.batch_size > 1048576 {
            return Err(CommonError::CommonError(
                "batch_size must be between 1 and 1048576 bytes".to_string(),
            ));
        }

        if self.linger_ms > 60000 {
            return Err(CommonError::CommonError(
                "linger_ms cannot exceed 60000 (60 seconds)".to_string(),
            ));
        }

        if self.retries > 100 {
            return Err(CommonError::CommonError(
                "retries cannot exceed 100".to_string(),
            ));
        }

        if self.message_timeout_ms < 1000 || self.message_timeout_ms > 300000 {
            return Err(CommonError::CommonError(
                "message_timeout_ms must be between 1000 (1s) and 300000 (5min)".to_string(),
            ));
        }

        if self.cleanup_timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "cleanup_timeout_secs cannot exceed 300 seconds".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_bootstrap_servers_format(&self) -> Result<(), CommonError> {
        let servers: Vec<&str> = self.bootstrap_servers.split(',').collect();

        if servers.is_empty() {
            return Err(CommonError::CommonError(
                "bootstrap_servers must contain at least one server address".to_string(),
            ));
        }

        for (idx, server) in servers.iter().enumerate() {
            let server = server.trim();

            if server.is_empty() {
                return Err(CommonError::CommonError(format!(
                    "bootstrap_servers contains empty address at position {}",
                    idx
                )));
            }

            let parts: Vec<&str> = server.split(':').collect();
            if parts.len() != 2 {
                return Err(CommonError::CommonError(format!(
                    "Invalid bootstrap server format '{}' at position {}. Expected format: host:port",
                    server, idx
                )));
            }

            let host = parts[0];
            let port_str = parts[1];

            if host.is_empty() {
                return Err(CommonError::CommonError(format!(
                    "Empty host in bootstrap server '{}' at position {}",
                    server, idx
                )));
            }

            match port_str.parse::<u16>() {
                Ok(port) => {
                    if port == 0 {
                        return Err(CommonError::CommonError(format!(
                            "Invalid port 0 in bootstrap server '{}' at position {}",
                            server, idx
                        )));
                    }
                }
                Err(_) => {
                    return Err(CommonError::CommonError(format!(
                        "Invalid port '{}' in bootstrap server '{}' at position {}. Port must be between 1-65535",
                        port_str, server, idx
                    )));
                }
            }
        }

        Ok(())
    }

    fn validate_bootstrap_servers_connectivity(&self) -> Result<(), CommonError> {
        let servers: Vec<&str> = self.bootstrap_servers.split(',').collect();
        let mut reachable_count = 0;
        let mut last_error = String::new();

        for server in servers.iter() {
            let server = server.trim();

            match TcpStream::connect_timeout(
                &server.parse::<SocketAddr>().map_err(|e| {
                    CommonError::CommonError(format!("Failed to parse address '{}': {}", server, e))
                })?,
                Duration::from_secs(3),
            ) {
                Ok(_) => {
                    reachable_count += 1;
                }
                Err(e) => {
                    last_error = format!("Failed to connect to {}: {}", server, e);
                }
            }
        }

        if reachable_count == 0 {
            return Err(CommonError::CommonError(format!(
                "None of the bootstrap servers are reachable. Last error: {}",
                last_error
            )));
        }

        Ok(())
    }
}
