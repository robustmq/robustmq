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

use crate::handler::error::MqttBrokerError;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SlowSubData {
    sub_name: String,
    client_id: String,
    topic: String,
    time_ms: u128,
    node_info: String,
    create_time: u64,
}

impl SlowSubData {
    pub fn build(sub_name: String, client_id: String, topic_name: String, time_ms: u128) -> Self {
        let ip = get_local_ip();
        let node_info = format!("RobustMQ-MQTT@{}", ip);
        SlowSubData {
            sub_name,
            client_id,
            topic: topic_name,
            time_ms,
            node_info,
            create_time: now_second(),
        }
    }
}

pub fn record_slow_sub_data(slow_data: SlowSubData) -> Result<(), MqttBrokerError> {
    let _ = serde_json::to_string(&slow_data)?;
    Ok(())
}
