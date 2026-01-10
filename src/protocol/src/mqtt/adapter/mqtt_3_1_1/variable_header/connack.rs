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

use crate::mqtt::adapter::mqtt_3_1_1::variable_header::return_code::ReturnCode;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ConnAckVariableHeader {
    session_present: bool,
    return_code: ReturnCode,
}

#[allow(dead_code)]
impl ConnAckVariableHeader {
    pub fn new(session_present: bool, return_code: ReturnCode) -> Self {
        ConnAckVariableHeader {
            session_present,
            return_code,
        }
    }

    pub fn session_present(&self) -> bool {
        self.session_present
    }

    pub fn return_code(&self) -> &ReturnCode {
        &self.return_code
    }
}
