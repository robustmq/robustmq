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

use common_base::tools::{get_local_ip, now_second};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct SlowSubscribeKey {
    pub time_span: u64,
    pub client_id: String,
    pub topic_name: String,
}

impl SlowSubscribeKey {
    pub fn build(time_span: u64, client_id: String, topic_name: String) -> Self {
        Self {
            time_span,
            client_id,
            topic_name,
        }
    }

    pub fn time_span(&self) -> u64 {
        self.time_span
    }

    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    pub fn topic_name(&self) -> &str {
        &self.topic_name
    }
}

impl PartialOrd for SlowSubscribeKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SlowSubscribeKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time_span
            .cmp(&other.time_span)
            .then_with(|| self.client_id.cmp(&other.client_id))
            .then_with(|| self.topic_name.cmp(&other.topic_name))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct SlowSubscribeData {
    pub subscribe_name: String,
    pub client_id: String,
    pub topic_name: String,
    pub node_info: String,
    pub time_span: u64,
    pub create_time: u64,
}

impl SlowSubscribeData {
    pub fn build(
        subscribe_name: String,
        client_id: String,
        topic_name: String,
        time_span: u64,
    ) -> Self {
        let ip = get_local_ip();
        let node_info = format!("RobustMQ-MQTT@{ip}");
        SlowSubscribeData {
            subscribe_name,
            client_id,
            topic_name,
            time_span,
            node_info,
            create_time: now_second(),
        }
    }
}
