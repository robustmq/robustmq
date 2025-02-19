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

use common_base::{config::placement_center::placement_center_conf, tools::now_second};
use log::info;
use tokio::{select, sync::broadcast};

use crate::mqtt::cache::MqttCacheManager;

pub async fn start_connector_heartbeat_check(
    mqtt_cache: Arc<MqttCacheManager>,
    stop_send: broadcast::Sender<bool>,
) {
    let mut recv = stop_send.subscribe();
    loop {
        select! {
            val = recv.recv() =>{
                if let Ok(flag) = val {
                    if flag {
                        break;
                    }
                }
            }
            _ = connector_heartbeat_check(&mqtt_cache)=>{}
        }
    }
}

async fn connector_heartbeat_check(mqtt_cache: &Arc<MqttCacheManager>) {
    let config = placement_center_conf();
    for connector in mqtt_cache.get_all_connector() {
        if now_second() - connector.last_heartbeat > config.heartbeat.heartbeat_timeout_ms / 1000 {
            info!(
                "cluster:{},Connector {} heartbeat expired, rescheduled, new node: {}",
                connector.cluster_name, connector.connector_name, 1
            )
        }
    }
}
