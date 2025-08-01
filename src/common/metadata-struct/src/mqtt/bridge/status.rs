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

use std::fmt::Display;

use protocol::broker_mqtt::broker_mqtt_admin::MqttStatus as ProtoMQTTStatus;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub enum MQTTStatus {
    #[default]
    Idle,
    Running,
}

impl Display for MQTTStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl From<MQTTStatus> for ProtoMQTTStatus {
    fn from(status: MQTTStatus) -> Self {
        match status {
            MQTTStatus::Idle => ProtoMQTTStatus::Idle,
            MQTTStatus::Running => ProtoMQTTStatus::Running,
        }
    }
}
