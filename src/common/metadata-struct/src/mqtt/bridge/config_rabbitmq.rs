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

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
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

        Ok(())
    }
}

fn default_port() -> u16 {
    5672
}

fn default_virtual_host() -> String {
    "/".to_string()
}
