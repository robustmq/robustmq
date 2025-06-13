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

use serde::{Deserialize, Serialize};

use super::default::{
    default_auth, default_grpc_port, default_heartbeat_timeout, default_log,
    default_mqtt_cluster_dynamic_feature, default_mqtt_cluster_dynamic_flapping_detect,
    default_mqtt_cluster_dynamic_network, default_mqtt_cluster_dynamic_protocol,
    default_mqtt_cluster_dynamic_security, default_mqtt_cluster_dynamic_slow_sub,
    default_mqtt_cluster_dynamic_system_monitor, default_network, default_network_quic_port,
    default_network_tcp_port, default_network_tcps_port, default_network_websocket_port,
    default_network_websockets_port, default_offline_message, default_placement_center,
    default_storage, default_system, default_system_monitor, default_tcp_thread, default_telemetry,
};
use crate::common::{
    default_pprof, default_prometheus, Auth, Log, Pprof, Prometheus, Storage, Telemetry,
};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BrokerMqttConfig {
    pub cluster_name: String,
    pub broker_id: u64,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u32,
    #[serde(default = "default_placement_center")]
    pub placement_center: Vec<String>,
    #[serde(default = "default_heartbeat_timeout")]
    pub heartbeat_timeout: String,
    #[serde(default = "default_network")]
    pub network: Network,
    #[serde(default = "default_tcp_thread")]
    pub tcp_thread: TcpThread,
    #[serde(default = "default_system")]
    pub system: System,
    #[serde(default = "default_system_monitor")]
    pub system_monitor: SystemMonitor,
    #[serde(default = "default_storage")]
    pub storage: Storage,
    #[serde(default = "default_auth")]
    pub auth: Auth,
    #[serde(default = "default_log")]
    pub log: Log,
    #[serde(default = "default_offline_message")]
    pub offline_messages: OfflineMessage,
    #[serde(default = "default_telemetry")]
    pub telemetry: Telemetry,
    #[serde(default = "default_prometheus")]
    pub prometheus: Prometheus,
    #[serde(default = "default_pprof")]
    pub pprof: Pprof,

    #[serde(default = "default_mqtt_cluster_dynamic_slow_sub")]
    pub cluster_dynamic_config_slow_sub: MqttClusterDynamicSlowSub,
    #[serde(default = "default_mqtt_cluster_dynamic_flapping_detect")]
    pub cluster_dynamic_config_flapping_detect: MqttClusterDynamicFlappingDetect,
    #[serde(default = "default_mqtt_cluster_dynamic_system_monitor")]
    pub cluster_dynamic_system_monitor: MqttClusterDynamicSystemMonitor,
    #[serde(default = "default_mqtt_cluster_dynamic_protocol")]
    pub cluster_dynamic_config_protocol: MqttClusterDynamicConfigProtocol,
    #[serde(default = "default_mqtt_cluster_dynamic_feature")]
    pub cluster_dynamic_config_feature: MqttClusterDynamicConfigFeature,
    #[serde(default = "default_mqtt_cluster_dynamic_security")]
    pub cluster_dynamic_config_security: MqttClusterDynamicConfigSecurity,
    #[serde(default = "default_mqtt_cluster_dynamic_network")]
    pub cluster_dynamic_config_network: MqttClusterDynamicConfigNetwork,
}

// MQTT cluster protocol related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigProtocol {
    pub max_session_expiry_interval: u32,
    pub default_session_expiry_interval: u32,
    pub topic_alias_max: u16,
    pub max_qos: u8,
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
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
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
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigNetwork {
    pub tcp_max_connection_num: u64,
    pub tcps_max_connection_num: u64,
    pub websocket_max_connection_num: u64,
    pub websockets_max_connection_num: u64,
    pub response_max_try_mut_times: u64,
    pub response_try_mut_sleep_time_ms: u64,
}

// MQTT cluster Feature related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigFeature {
    pub retain_available: ConfigAvailableFlag,
    pub wildcard_subscription_available: ConfigAvailableFlag,
    pub subscription_identifiers_available: ConfigAvailableFlag,
    pub shared_subscription_available: ConfigAvailableFlag,
    pub exclusive_subscription_available: ConfigAvailableFlag,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
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

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicSystemMonitor {
    pub enable: bool,
    pub os_cpu_high_watermark: f32,
    pub os_cpu_low_watermark: f32,
    pub os_memory_high_watermark: f32,
}

impl MqttClusterDynamicSystemMonitor {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicOfflineMessage {
    pub enable: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
pub enum ConfigAvailableFlag {
    #[default]
    Disable,
    Enable,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Network {
    #[serde(default = "default_network_tcp_port")]
    pub tcp_port: u32,
    #[serde(default = "default_network_tcps_port")]
    pub tcps_port: u32,
    #[serde(default = "default_network_websocket_port")]
    pub websocket_port: u32,
    #[serde(default = "default_network_websockets_port")]
    pub websockets_port: u32,
    #[serde(default = "default_network_quic_port")]
    pub quic_port: u32,
    #[serde(default)]
    pub tls_cert: String,
    #[serde(default)]
    pub tls_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TcpThread {
    #[serde(default)]
    pub accept_thread_num: usize,
    #[serde(default)]
    pub handler_thread_num: usize,
    #[serde(default)]
    pub response_thread_num: usize,
    #[serde(default)]
    pub max_connection_num: usize,
    #[serde(default)]
    pub request_queue_size: usize,
    #[serde(default)]
    pub response_queue_size: usize,
    #[serde(default)]
    pub lock_max_try_mut_times: u64,
    #[serde(default)]
    pub lock_try_mut_sleep_time_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct System {
    #[serde(default)]
    pub runtime_worker_threads: usize,
    #[serde(default)]
    pub default_user: String,
    #[serde(default)]
    pub default_password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct OfflineMessage {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub expire_ms: u32,
    #[serde(default)]
    pub max_messages_num: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SystemMonitor {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub os_cpu_check_interval_ms: u64,
    #[serde(default)]
    pub os_cpu_high_watermark: f32,
    #[serde(default)]
    pub os_cpu_low_watermark: f32,
    #[serde(default)]
    pub os_memory_check_interval_ms: u64,
    #[serde(default)]
    pub os_memory_high_watermark: f32,
}
