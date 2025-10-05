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

fn default_port() -> u16 {
    5672
}

fn default_virtual_host() -> String {
    "/".to_string()
}
