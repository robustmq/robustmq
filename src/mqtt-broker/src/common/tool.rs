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

use broker_core::cache::BrokerCacheManager;
use grpc_clients::pool::ClientPool;
use protocol::mqtt::common::MqttPacket;

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

pub fn test_build_mqtt_cache_manager() -> Arc<MQTTCacheManager> {
    let client_pool: Arc<ClientPool> = Arc::new(ClientPool::new(100));
    let broker_cache = Arc::new(BrokerCacheManager::new("test".to_string()));
    Arc::new(MQTTCacheManager::new(client_pool, broker_cache))
}
