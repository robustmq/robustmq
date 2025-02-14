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
use log::{error, info};
use protocol::mqtt::codec::parse_mqtt_packet_to_name;
use serde::{Deserialize, Serialize};

use crate::handler::cache::CacheManager;
use crate::server::packet::RequestPackage;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SlowRequestMs {
    command: String,
    time_ms: u128,
}

// total processing time of the request packet was recorded
pub fn try_record_total_request_ms(cache_manager: Arc<CacheManager>, package: RequestPackage) {
    let cluster_config = cache_manager.get_cluster_info();
    if !cluster_config.slow.as_ref().unwrap().enable {
        return;
    }

    let whole = cluster_config.slow.as_ref().unwrap().whole_ms;
    let time_ms = now_mills() - package.receive_ms;
    if time_ms < whole.into() {
        return;
    }

    let command = parse_mqtt_packet_to_name(package.packet);
    let slow = SlowRequestMs { command, time_ms };

    match serde_json::to_string(&slow) {
        Ok(data) => info!("{}", data),
        Err(e) => error!(
            "Failed to serialize slow subscribe message with error message :{}",
            e.to_string()
        ),
    }
}
