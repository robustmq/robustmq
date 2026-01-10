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
pub struct PublishVariableHeader {
    topic_name: String,
    packet_identifier: Option<u16>,
}

#[allow(dead_code)]
impl PublishVariableHeader {
    pub fn new(topic_name: String, packet_identifier: Option<u16>) -> Self {
        PublishVariableHeader {
            topic_name,
            packet_identifier,
        }
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }

    pub fn packet_identifier(&self) -> Option<u16> {
        self.packet_identifier
    }
}
