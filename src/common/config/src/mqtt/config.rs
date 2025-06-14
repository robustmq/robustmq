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

use super::default::{
    default_auth_storage, default_feature, default_flapping_detect, default_grpc_port,
    default_heartbeat_timeout, default_log, default_message_storage, default_network_port,
    default_network_quic_port, default_network_tcp_port, default_network_tcps_port,
    default_network_thread, default_network_websocket_port, default_network_websockets_port,
    default_offline_message, default_placement_center, default_protocol, default_schema,
    default_security, default_slow_sub, default_system, default_system_monitor, default_telemetry,
};
use crate::common::{
    default_pprof, default_prometheus, AvailableFlag, Log, Pprof, Prometheus, Telemetry,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BrokerMqttConfig {
    pub cluster_name: String,

    pub broker_id: u64,

    // grpc port
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u32,

    // placement center
    #[serde(default = "default_placement_center")]
    pub placement_center: Vec<String>,

    // network port
    #[serde(default = "default_network_port")]
    pub network_port: NetworkPort,

    // network thread
    #[serde(default = "default_network_thread")]
    pub network_thread: NetworkThread,

    // system config
    #[serde(default = "default_system")]
    pub system: System,

    // message storage
    #[serde(default = "default_message_storage")]
    pub storage: MessageDataStorage,

    // auth storage
    #[serde(default = "default_auth_storage")]
    pub auth_storage: AuthStorage,

    // log
    #[serde(default = "default_log")]
    pub log: Log,

    // offline message
    #[serde(default = "default_offline_message")]
    pub offline_messages: OfflineMessage,

    // telemetry
    #[serde(default = "default_telemetry")]
    pub telemetry: Telemetry,

    // prometheus
    #[serde(default = "default_prometheus")]
    pub prometheus: Prometheus,

    // pprof
    #[serde(default = "default_pprof")]
    pub pprof: Pprof,

    // slow sub
    #[serde(default = "default_slow_sub")]
    pub slow_sub: SlowSub,

    // flapping detect
    #[serde(default = "default_flapping_detect")]
    pub flapping_detect: FlappingDetect,

    // mqtt protocol related configuration
    #[serde(default = "default_protocol")]
    pub mqtt_protocol_config: MqttProtocolConfig,

    // mqtt support feature
    #[serde(default = "default_feature")]
    pub feature: Feature,

    // security
    #[serde(default = "default_security")]
    pub security: Security,

    // security
    #[serde(default = "default_schema")]
    pub schema: Schema,

    // system monitor
    #[serde(default = "default_system_monitor")]
    pub system_monitor: SystemMonitor,
}

// MQTT cluster protocol related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttProtocolConfig {
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

impl MqttProtocolConfig {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

// MQTT cluster security related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Security {
    pub is_self_protection_status: bool,
    pub secret_free_login: bool,
}

impl Security {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct AuthStorage {
    pub storage_type: String,
    #[serde(default)]
    pub journal_addr: String,
    #[serde(default)]
    pub mysql_addr: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct MessageDataStorage {
    pub storage_type: String,
    #[serde(default)]
    pub journal_addr: String,
    #[serde(default)]
    pub mysql_addr: String,
    #[serde(default)]
    pub rocksdb_data_path: String,
    pub rocksdb_max_open_files: Option<i32>,
}

// MQTT cluster Feature related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Feature {
    pub retain_available: AvailableFlag,
    pub wildcard_subscription_available: AvailableFlag,
    pub subscription_identifiers_available: AvailableFlag,
    pub shared_subscription_available: AvailableFlag,
    pub exclusive_subscription_available: AvailableFlag,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct SlowSub {
    pub enable: bool,
    pub whole_ms: u64,
    pub internal_ms: u32,
    pub response_ms: u32,
}
impl SlowSub {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct FlappingDetect {
    pub enable: bool,
    pub window_time: u32,
    pub max_client_connections: u64,
    pub ban_time: u32,
}

impl FlappingDetect {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
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

impl SystemMonitor {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicOfflineMessage {
    pub enable: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct NetworkPort {
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
pub struct NetworkThread {
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
    #[serde(default = "default_heartbeat_timeout")]
    pub heartbeat_timeout: String,
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

impl OfflineMessage {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

// Schema
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Schema {
    pub enable: bool,
    pub strategy: SchemaStrategy,
    pub failed_operation: SchemaFailedOperation,
    pub echo_log: bool,
    pub log_level: String,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub enum SchemaStrategy {
    #[default]
    ALL,
    Any,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub enum SchemaFailedOperation {
    #[default]
    Discard,
    DisconnectAndDiscard,
    Ignore,
}

impl Schema {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}
