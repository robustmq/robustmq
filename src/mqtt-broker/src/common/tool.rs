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

use std::{sync::Arc, time::Duration};

use common_base::node_status::NodeStatus;
use protocol::mqtt::common::MqttPacket;
use tokio::time::sleep;

use crate::handler::cache::MQTTCacheManager;

pub fn is_ignore_print(packet: &MqttPacket) -> bool {
    if let MqttPacket::PingResp(_) = packet {
        return true;
    }
    if let MqttPacket::PingReq(_) = packet {
        return true;
    }
    false
}

pub async fn wait_cluster_running(cache_manager: &Arc<MQTTCacheManager>) {
    loop {
        if cache_manager.get_status() == NodeStatus::Running {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }
}
