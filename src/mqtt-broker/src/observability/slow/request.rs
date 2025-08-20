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

use std::sync::Arc;

use common_base::tools::now_mills;
use network_server::common::packet::RequestPackage;
use protocol::mqtt::codec::parse_mqtt_packet_to_name;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::handler::cache::CacheManager;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SlowRequestMs {
    command: String,
    time_ms: u128,
}

// total processing time of the request packet was recorded
pub fn try_record_total_request_ms(cache_manager: Arc<CacheManager>, package: RequestPackage) {
    let cluster_config = cache_manager.get_cluster_config();
    if !cluster_config.mqtt_slow_sub.enable {
        return;
    }

    let whole = cluster_config.mqtt_slow_sub.whole_ms;
    let time_ms = now_mills() - package.receive_ms;
    if time_ms < whole as u128 {
        return;
    }

    let command = parse_mqtt_packet_to_name(package.packet.get_mqtt_packet().unwrap());
    let slow = SlowRequestMs { command, time_ms };

    match serde_json::to_string(&slow) {
        Ok(data) => info!("{}", data),
        Err(e) => error!(
            "Failed to serialize slow subscribe message with error message :{}",
            e.to_string()
        ),
    }
}
