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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum RobustMQProtocol {
    MQTT3,
    MQTT4,
    MQTT5,
    KAFKA,
}

impl RobustMQProtocol {
    pub fn is_mqtt(&self) -> bool {
        *self == RobustMQProtocol::MQTT3
            || *self == RobustMQProtocol::MQTT4
            || *self == RobustMQProtocol::MQTT5
    }

    pub fn is_kafka(&self) -> bool {
        *self == RobustMQProtocol::KAFKA
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            RobustMQProtocol::MQTT3 => 3,
            RobustMQProtocol::MQTT4 => 4,
            RobustMQProtocol::MQTT5 => 5,
            RobustMQProtocol::KAFKA => 0,
        }
    }
}
