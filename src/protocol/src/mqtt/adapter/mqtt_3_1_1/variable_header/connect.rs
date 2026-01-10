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

use crate::mqtt::adapter::common::protocol_level::ProtocolLevel;
use crate::mqtt::adapter::common::qos::QoSLevel;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ConnectVariableHeader {
    protocol_name: String,
    protocol_level: ProtocolLevel,
    connect_flags: ConnectVariableFlags,
    keep_alive: u16,
}

#[allow(dead_code)]
impl ConnectVariableHeader {
    pub fn new(connect_flags: ConnectVariableFlags, keep_alive: u16) -> Self {
        ConnectVariableHeader {
            protocol_name: "MQTT".to_string(),
            protocol_level: ProtocolLevel::Mqtt3_1_1,
            connect_flags,
            keep_alive,
        }
    }

    pub fn protocol_name(&self) -> &str {
        &self.protocol_name
    }

    pub fn protocol_level(&self) -> &ProtocolLevel {
        &self.protocol_level
    }

    pub fn connect_flags(&self) -> &ConnectVariableFlags {
        &self.connect_flags
    }

    pub fn keep_alive(&self) -> u16 {
        self.keep_alive
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnectVariableFlags {
    username_flag: bool,
    password_flag: bool,
    will_retain: bool,
    will_qos: QoSLevel,
    will_flag: bool,
    clean_session: bool,
}

#[allow(dead_code)]
impl ConnectVariableFlags {
    pub fn new(
        username_flag: bool,
        password_flag: bool,
        will_retain: bool,
        will_qos: QoSLevel,
        will_flag: bool,
        clean_session: bool,
    ) -> Self {
        ConnectVariableFlags {
            username_flag,
            password_flag,
            will_retain,
            will_qos,
            will_flag,
            clean_session,
        }
    }

    pub fn username_flag(&self) -> bool {
        self.username_flag
    }
    pub fn password_flag(&self) -> bool {
        self.password_flag
    }
    pub fn will_retain(&self) -> bool {
        self.will_retain
    }
    pub fn will_qos(&self) -> &QoSLevel {
        &self.will_qos
    }
    pub fn will_flag(&self) -> bool {
        self.will_flag
    }
    pub fn clean_session(&self) -> bool {
        self.clean_session
    }
}
