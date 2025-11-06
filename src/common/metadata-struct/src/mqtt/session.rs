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

use common_base::{error::common::CommonError, tools::now_second, utils::serialize};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct MqttSession {
    pub client_id: String,
    pub session_expiry: u64,
    pub is_contain_last_will: bool,
    pub last_will_delay_interval: Option<u64>,
    pub create_time: u64,

    pub connection_id: Option<u64>,
    pub broker_id: Option<u64>,
    pub reconnect_time: Option<u64>,
    pub distinct_time: Option<u64>,
}

impl MqttSession {
    pub fn new(
        client_id: String,
        session_expiry: u64,
        is_contain_last_will: bool,
        last_will_delay_interval: Option<u64>,
    ) -> MqttSession {
        MqttSession {
            client_id,
            session_expiry,
            is_contain_last_will,
            last_will_delay_interval,
            create_time: now_second(),
            ..Default::default()
        }
    }

    pub fn update_connnction_id(&mut self, connection_id: Option<u64>) {
        self.connection_id = connection_id;
    }

    pub fn update_broker_id(&mut self, broker_id: Option<u64>) {
        self.broker_id = broker_id;
    }

    pub fn update_update_time(&mut self) {
        self.reconnect_time = Some(now_second());
    }

    pub fn update_reconnect_time(&mut self) {
        self.reconnect_time = Some(now_second());
    }

    pub fn update_distinct_time(&mut self) {
        self.distinct_time = Some(now_second());
    }

    pub fn encode(&self) -> Result<Vec<u8>, CommonError> {
        serialize::serialize(self)
    }

    pub fn decode(data: &[u8]) -> Result<Self, CommonError> {
        serialize::deserialize(data)
    }
}
