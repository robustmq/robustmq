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

use protocol::mqtt::common::QoS;
use serde::{Deserialize, Serialize};

// Dynamic configuration of MQTT cluster latitude
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfig {
    pub protocol: MqttClusterDynamicConfigProtocol,
    pub feature: MqttClusterDynamicConfigFeature,
    pub security: MqttClusterDynamicConfigSecurity,
    pub network: MqttClusterDynamicConfigNetwork,
    pub slow: MqttClusterDynamicSlowSub,
}

// MQTT cluster protocol related dynamic configuration
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigProtocol {
    pub session_expiry_interval: u32,
    pub topic_alias_max: u16,
    pub max_qos: QoS,
    pub max_packet_size: u32,
    pub max_server_keep_alive: u16,
    pub default_server_keep_alive: u16,
    pub receive_max: u16,
    pub max_message_expiry_interval: u64,
    pub client_pkid_persistent: bool,
}

// MQTT cluster security related dynamic configuration
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigSecurity {
    pub is_self_protection_status: bool,
    pub secret_free_login: bool,
}

// MQTT cluster network related dynamic configuration
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigNetwork {
    pub tcp_max_connection_num: u64,
    pub tcps_max_connection_num: u64,
    pub websocket_max_connection_num: u64,
    pub websockets_max_connection_num: u64,
    pub response_max_try_mut_times: u64,
    pub response_try_mut_sleep_time_ms: u64,
}

// MQTT cluster Feature related dynamic configuration
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigFeature {
    pub retain_available: AvailableFlag,
    pub wildcard_subscription_available: AvailableFlag,
    pub subscription_identifiers_available: AvailableFlag,
    pub shared_subscription_available: AvailableFlag,
    pub exclusive_subscription_available: AvailableFlag,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicSlowSub {
    pub enable: bool,
    pub whole_ms: u128,
    pub internal_ms: u32,
    pub response_ms: u32,
}

impl MqttClusterDynamicConfig {
    pub fn new() -> Self {
        MqttClusterDynamicConfig {
            protocol: MqttClusterDynamicConfigProtocol {
                session_expiry_interval: 1800,
                topic_alias_max: 65535,
                max_qos: QoS::ExactlyOnce,
                max_packet_size: 1024 * 1024 * 10,
                max_server_keep_alive: 3600,
                default_server_keep_alive: 60,
                receive_max: 65535,
                client_pkid_persistent: false,
                max_message_expiry_interval: 3600,
            },
            feature: MqttClusterDynamicConfigFeature {
                retain_available: AvailableFlag::Enable,
                wildcard_subscription_available: AvailableFlag::Enable,
                subscription_identifiers_available: AvailableFlag::Enable,
                shared_subscription_available: AvailableFlag::Enable,
                exclusive_subscription_available: AvailableFlag::Enable,
            },
            security: MqttClusterDynamicConfigSecurity {
                secret_free_login: false,
                is_self_protection_status: false,
            },
            network: MqttClusterDynamicConfigNetwork {
                tcp_max_connection_num: 1000,
                tcps_max_connection_num: 1000,
                websocket_max_connection_num: 1000,
                websockets_max_connection_num: 1000,
                response_max_try_mut_times: 128,
                response_try_mut_sleep_time_ms: 100,
            },
            slow: MqttClusterDynamicSlowSub {
                enable: false,
                whole_ms: 0,
                internal_ms: 0,
                response_ms: 0,
            },
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Default, Clone)]
pub enum AvailableFlag {
    #[default]
    Disable,
    Enable,
}

#[cfg(test)]
mod tests {
    use crate::mqtt::cluster::AvailableFlag;

    #[test]
    fn client34_connect_test() {
        assert_eq!(AvailableFlag::Disable as u8, 0);
        assert_eq!(AvailableFlag::Enable as u8, 1);
    }
}
