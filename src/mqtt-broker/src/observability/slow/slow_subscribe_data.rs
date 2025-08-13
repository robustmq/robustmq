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

#[derive(Debug, PartialEq, Serialize, Deserialize, Default, Clone)]
pub struct SlowSubscribeData {
    pub(crate) subscribe_name: String,
    pub(crate) client_id: String,
    pub(crate) topic_name: String,
    pub(crate) node_info: String,
    pub(crate) time_span: u64,
    pub(crate) create_time: u64,
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
