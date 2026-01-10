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

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct SubAckPayload {
    return_codes: Vec<SubAckReturnCode>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubAckReturnCode {
    SuccessQoS0,
    SuccessQoS1,
    SuccessQoS2,
    Failure,
}

#[allow(dead_code)]
impl SubAckPayload {
    pub fn new(return_codes: Vec<SubAckReturnCode>) -> Self {
        SubAckPayload { return_codes }
    }

    pub fn return_codes(&self) -> &Vec<SubAckReturnCode> {
        &self.return_codes
    }
}
