// Copyright 2023 RobustMQ Team
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


#[derive(Debug, PartialEq, Eq)]
pub struct TMqttRecord {
    pub msgid: String,
    pub header: String,
    pub msg_key: String,
    pub payload: Vec<u8>,
    pub create_time: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TMqttKvMsg {
    pub key: String,
    pub value: Vec<u8>,
    pub create_time: u64,
    pub update_time: u64,
}
