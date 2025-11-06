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

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub enum DeliveryMode {
    #[default]
    NonPersistent,
    Persistent,
}

fn default_port() -> u16 {
    5672
}

fn default_virtual_host() -> String {
    "/".to_string()
}

fn default_connection_timeout_secs() -> u64 {
    30
}

fn default_heartbeat_secs() -> u16 {
    60
}

fn default_batch_size() -> usize {
    100
}

fn default_channel_max() -> u16 {
    2047
}

fn default_frame_max() -> u32 {
    131072
}

fn default_confirm_timeout_secs() -> u64 {
    30
}

impl Default for RabbitMQConnectorConfig {
    fn default() -> Self {
        Self {
            server: String::new(),
            port: default_port(),
            username: String::new(),
            password: String::new(),
            virtual_host: default_virtual_host(),
            exchange: String::new(),
            routing_key: String::new(),
            delivery_mode: DeliveryMode::default(),
            enable_tls: false,
            connection_timeout_secs: default_connection_timeout_secs(),
            heartbeat_secs: default_heartbeat_secs(),
            batch_size: default_batch_size(),
            channel_max: default_channel_max(),
            frame_max: default_frame_max(),
            confirm_timeout_secs: default_confirm_timeout_secs(),
            publisher_confirms: true,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RabbitMQConnectorConfig {
    pub server: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub username: String,
    pub password: String,
    #[serde(default = "default_virtual_host")]
    pub virtual_host: String,
    pub exchange: String,
    pub routing_key: String,
    #[serde(default)]
    pub delivery_mode: DeliveryMode,
    #[serde(default)]
    pub enable_tls: bool,

    #[serde(default = "default_connection_timeout_secs")]
    pub connection_timeout_secs: u64,
    #[serde(default = "default_heartbeat_secs")]
    pub heartbeat_secs: u16,

    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_channel_max")]
    pub channel_max: u16,
    #[serde(default = "default_frame_max")]
    pub frame_max: u32,
    #[serde(default = "default_confirm_timeout_secs")]
    pub confirm_timeout_secs: u64,
    #[serde(default = "default_publisher_confirms")]
    pub publisher_confirms: bool,
}

fn default_publisher_confirms() -> bool {
    true
}

impl DeliveryMode {
    pub fn to_amqp_value(&self) -> u8 {
        match self {
            DeliveryMode::NonPersistent => 1,
            DeliveryMode::Persistent => 2,
        }
    }
}

impl RabbitMQConnectorConfig {
    pub fn validate(&self) -> Result<(), CommonError> {
        if self.server.is_empty() {
            return Err(CommonError::CommonError(
                "server cannot be empty".to_string(),
            ));
        }

        if self.server.len() > 512 {
            return Err(CommonError::CommonError(
                "server length cannot exceed 512 characters".to_string(),
            ));
        }

        if self.port == 0 {
            return Err(CommonError::CommonError(
                "port must be greater than 0".to_string(),
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

        if self.virtual_host.len() > 256 {
            return Err(CommonError::CommonError(
                "virtual_host length cannot exceed 256 characters".to_string(),
            ));
        }

        if self.exchange.is_empty() {
            return Err(CommonError::CommonError(
                "exchange cannot be empty".to_string(),
            ));
        }

        if self.exchange.len() > 256 {
            return Err(CommonError::CommonError(
                "exchange length cannot exceed 256 characters".to_string(),
            ));
        }

        if self.routing_key.len() > 256 {
            return Err(CommonError::CommonError(
                "routing_key length cannot exceed 256 characters".to_string(),
            ));
        }

        if self.connection_timeout_secs == 0 || self.connection_timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "connection_timeout_secs must be between 1 and 300 seconds".to_string(),
            ));
        }

        if self.heartbeat_secs > 300 {
            return Err(CommonError::CommonError(
                "heartbeat_secs cannot exceed 300 seconds".to_string(),
            ));
        }

        if self.batch_size == 0 || self.batch_size > 10000 {
            return Err(CommonError::CommonError(
                "batch_size must be between 1 and 10000".to_string(),
            ));
        }

        if self.channel_max == 0 {
            return Err(CommonError::CommonError(
                "channel_max must be greater than 0".to_string(),
            ));
        }

        if self.frame_max < 4096 || self.frame_max > 1048576 {
            return Err(CommonError::CommonError(
                "frame_max must be between 4096 and 1048576 bytes".to_string(),
            ));
        }

        if self.confirm_timeout_secs == 0 || self.confirm_timeout_secs > 300 {
            return Err(CommonError::CommonError(
                "confirm_timeout_secs must be between 1 and 300 seconds".to_string(),
            ));
        }

        Ok(())
    }
}
