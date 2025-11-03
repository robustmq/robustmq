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
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RedisMode {
    #[default]
    Single,
    Cluster,
    Sentinel,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct RedisConnectorConfig {
    pub server: String,
    #[serde(default)]
    pub mode: RedisMode,
    #[serde(default = "default_database")]
    pub database: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sentinel_master_name: Option<String>,
    pub command_template: String,
    #[serde(default)]
    pub tls_enabled: bool,
    #[serde(default = "default_connect_timeout_ms")]
    pub connect_timeout_ms: u64,
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default = "default_retry_interval_ms")]
    pub retry_interval_ms: u64,
}

fn default_database() -> u8 {
    0
}

fn default_connect_timeout_ms() -> u64 {
    5000
}

fn default_pool_size() -> u32 {
    10
}

fn default_max_retries() -> u32 {
    3
}

fn default_retry_interval_ms() -> u64 {
    1000
}

impl RedisConnectorConfig {
    pub fn validate(&self) -> Result<(), CommonError> {
        if self.server.is_empty() {
            return Err(CommonError::CommonError(
                "server cannot be empty".to_string(),
            ));
        }

        if self.server.len() > 1024 {
            return Err(CommonError::CommonError(
                "server length cannot exceed 1024 characters".to_string(),
            ));
        }

        self.validate_server_format()?;
        self.validate_server_connectivity()?;

        if self.command_template.is_empty() {
            return Err(CommonError::CommonError(
                "command_template cannot be empty".to_string(),
            ));
        }

        if self.command_template.len() > 4096 {
            return Err(CommonError::CommonError(
                "command_template length cannot exceed 4096 characters".to_string(),
            ));
        }

        if self.mode == RedisMode::Sentinel && self.sentinel_master_name.is_none() {
            return Err(CommonError::CommonError(
                "sentinel_master_name is required for sentinel mode".to_string(),
            ));
        }

        if self.database > 15 {
            return Err(CommonError::CommonError(
                "database must be between 0 and 15".to_string(),
            ));
        }

        if self.pool_size == 0 {
            return Err(CommonError::CommonError(
                "pool_size must be greater than 0".to_string(),
            ));
        }

        if self.pool_size > 100 {
            return Err(CommonError::CommonError(
                "pool_size cannot exceed 100".to_string(),
            ));
        }

        if self.connect_timeout_ms == 0 {
            return Err(CommonError::CommonError(
                "connect_timeout_ms must be greater than 0".to_string(),
            ));
        }

        if self.retry_interval_ms == 0 {
            return Err(CommonError::CommonError(
                "retry_interval_ms must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    fn validate_server_format(&self) -> Result<(), CommonError> {
        let servers: Vec<&str> = self.server.split(',').collect();

        for (idx, server) in servers.iter().enumerate() {
            let server = server.trim();
            if server.is_empty() {
                return Err(CommonError::CommonError(format!(
                    "server contains empty address at position {}",
                    idx
                )));
            }

            if server.parse::<SocketAddr>().is_err() {
                let parts: Vec<&str> = server.split(':').collect();
                if parts.len() != 2 {
                    return Err(CommonError::CommonError(format!(
                        "Invalid server address format: {}. Expected format: host:port",
                        server
                    )));
                }

                if parts[0].is_empty() {
                    return Err(CommonError::CommonError(format!(
                        "Invalid server host: {}",
                        server
                    )));
                }

                if parts[1].parse::<u16>().is_err() {
                    return Err(CommonError::CommonError(format!(
                        "Invalid server port: {}",
                        server
                    )));
                }
            }
        }

        Ok(())
    }

    fn validate_server_connectivity(&self) -> Result<(), CommonError> {
        let servers: Vec<&str> = self.server.split(',').collect();
        let timeout = Duration::from_millis(self.connect_timeout_ms);

        let mut all_failed = true;

        for server in &servers {
            let server = server.trim();

            let addr = if let Ok(socket_addr) = server.parse::<SocketAddr>() {
                socket_addr
            } else {
                let parts: Vec<&str> = server.split(':').collect();
                if parts.len() != 2 {
                    continue;
                }

                let port = match parts[1].parse::<u16>() {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                match format!("{}:{}", parts[0], port).parse::<SocketAddr>() {
                    Ok(addr) => addr,
                    Err(_) => {
                        continue;
                    }
                }
            };

            if TcpStream::connect_timeout(&addr, timeout).is_ok() {
                all_failed = false;
                break;
            }
        }

        if all_failed && !servers.is_empty() {
            return Err(CommonError::CommonError(format!(
                "Unable to connect to any Redis server: {}. Please check the server addresses and network connectivity.",
                self.server
            )));
        }

        Ok(())
    }

    pub fn get_placeholders(&self) -> Vec<String> {
        let mut placeholders = Vec::new();
        let mut chars = self.command_template.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '$' && chars.peek() == Some(&'{') {
                chars.next();
                let mut placeholder = String::new();
                while let Some(&ch) = chars.peek() {
                    if ch == '}' {
                        chars.next();
                        if !placeholder.is_empty() {
                            placeholders.push(placeholder);
                        }
                        break;
                    }
                    placeholder.push(chars.next().unwrap());
                }
            }
        }

        placeholders
    }
}
