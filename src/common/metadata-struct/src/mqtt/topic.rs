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


use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct MQTTTopic {
    pub topic_id: String,
    pub topic_name: String,
    pub retain_message: Option<Vec<u8>>,
    pub retain_message_expired_at: Option<u64>,
}

impl MQTTTopic {
    pub fn new(topic_id: String, topic_name: String) -> Self {
        return MQTTTopic {
            topic_id,
            topic_name: topic_name,
            retain_message: None,
            retain_message_expired_at: None,
        };
    }

    pub fn encode(&self) -> Vec<u8> {
        return serde_json::to_vec(&self).unwrap();
    }
}
