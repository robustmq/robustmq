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

use protocol::broker_mqtt::broker_mqtt_admin::MqttTopicRewriteRuleRaw;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct MqttTopicRewriteRule {
    pub cluster: String,
    pub action: String,
    pub source_topic: String,
    pub dest_topic: String,
    pub regex: String,
    pub timestamp: u128,
}

impl MqttTopicRewriteRule {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

impl From<MqttTopicRewriteRule> for MqttTopicRewriteRuleRaw {
    fn from(value: MqttTopicRewriteRule) -> Self {
        MqttTopicRewriteRuleRaw {
            source_topic: value.source_topic,
            cluster_name: value.cluster,
            dest_topic: value.dest_topic,
            action: value.action,
            regex: value.regex,
        }
    }
}
