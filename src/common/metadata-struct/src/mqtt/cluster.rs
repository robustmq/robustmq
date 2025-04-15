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

pub const DEFAULT_DYNAMIC_CONFIG_SLOW_SUB: &str = "slow_sub";
pub const DEFAULT_DYNAMIC_CONFIG_FLAPPING_DETECT: &str = "flapping_detect";
pub const DEFAULT_DYNAMIC_CONFIG_PROTOCOL: &str = "protocol";
pub const DEFAULT_DYNAMIC_CONFIG_OFFLINE_MESSAGE: &str = "offline_message";
pub const DEFAULT_DYNAMIC_CONFIG_FEATURE: &str = "feature";
pub const DEFAULT_DYNAMIC_CONFIG_SECURITY: &str = "security";
pub const DEFAULT_DYNAMIC_CONFIG_NETWORK: &str = "network";

// Dynamic configuration of MQTT cluster latitude
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfig {
    pub protocol: MqttClusterDynamicConfigProtocol,
    pub feature: MqttClusterDynamicConfigFeature,
    pub security: MqttClusterDynamicConfigSecurity,
    pub network: MqttClusterDynamicConfigNetwork,
    pub slow: MqttClusterDynamicSlowSub,
    pub flapping_detect: MqttClusterDynamicFlappingDetect,
    pub offline_message: MqttClusterDynamicOfflineMessage,
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

impl MqttClusterDynamicConfigProtocol {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

// MQTT cluster security related dynamic configuration
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigSecurity {
    pub is_self_protection_status: bool,
    pub secret_free_login: bool,
}

impl MqttClusterDynamicConfigSecurity {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
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
    pub whole_ms: u64,
    pub internal_ms: u32,
    pub response_ms: u32,
}
impl MqttClusterDynamicSlowSub {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicFlappingDetect {
    pub enable: bool,
    pub window_time: u32,
    pub max_client_connections: u64,
    pub ban_time: u32,
}

impl MqttClusterDynamicFlappingDetect {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicOfflineMessage {
    pub enable: bool,
}

impl MqttClusterDynamicOfflineMessage {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

impl MqttClusterDynamicConfig {
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
