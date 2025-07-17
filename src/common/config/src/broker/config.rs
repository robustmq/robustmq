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
    default_broker_id, default_cluster_name, default_flapping_detect, default_grpc_port,
    default_mqtt_auth_storage, default_mqtt_message_storage, default_mqtt_offline_message,
    default_mqtt_protocol_config, default_mqtt_runtime, default_mqtt_schema, default_mqtt_security,
    default_mqtt_server, default_mqtt_slow_sub, default_mqtt_system_monitor, default_network,
    default_place_runtime, default_placement_center, default_rocksdb, default_roles,
    default_runtime,
};
use crate::common::Log;
use crate::common::Prometheus;
use crate::common::{default_log, default_pprof, default_prometheus};
use serde::{Deserialize, Serialize};
use toml::Table;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BrokerConfig {
    #[serde(default = "default_cluster_name")]
    pub cluster_name: String,

    #[serde(default = "default_broker_id")]
    pub broker_id: u64,

    #[serde(default = "default_roles")]
    pub roles: Vec<String>,

    #[serde(default = "default_grpc_port")]
    pub grpc_port: u32,

    #[serde(default = "default_placement_center")]
    pub placement_center: Table,

    #[serde(default = "default_prometheus")]
    pub prometheus: Prometheus,

    #[serde(default = "default_log")]
    pub log: Log,

    #[serde(default = "default_runtime")]
    pub runtime: Runtime,

    #[serde(default = "default_network")]
    pub network: Network,

    #[serde(default = "default_rocksdb")]
    pub rocksdb: Rocksdb,

    #[serde(default = "default_place_runtime")]
    pub place_runtime: PlaceRuntime,

    #[serde(default = "default_pprof")]
    pub p_prof: PProf,

    #[serde(default = "default_mqtt_server")]
    pub mqtt_server: MqttServer,

    #[serde(default = "default_mqtt_auth_storage")]
    pub mqtt_auth_storage: MqttAuthStorage,

    #[serde(default = "default_mqtt_message_storage")]
    pub mqtt_message_storage: MqttMessageStorage,

    #[serde(default = "default_mqtt_runtime")]
    pub mqtt_runtime: MqttRuntime,

    #[serde(default = "default_mqtt_offline_message")]
    pub mqtt_offline_message: MqttOfflineMessage,

    #[serde(default = "default_mqtt_slow_sub")]
    pub mqtt_slow_sub: MqttSlowSub,

    #[serde(default = "default_flapping_detect")]
    pub mqtt_flapping_detect: MqttFlappingDetect,

    #[serde(default = "default_mqtt_protocol_config")]
    pub mqtt_protocol_config: MqttProtocolConfig,

    #[serde(default = "default_mqtt_security")]
    pub mqtt_security: MqttSecurity,

    #[serde(default = "default_mqtt_schema")]
    pub mqtt_schema: MqttSchema,

    #[serde(default = "default_mqtt_system_monitor")]
    pub mqtt_system_monitor: MqttSystemMonitor,
}

impl BrokerConfig {
    pub fn get_placement_center_addr(&self) -> Vec<String> {
        self.placement_center
            .values()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Runtime {
    pub runtime_worker_threads: usize,

    pub tls_cert: String,

    pub tls_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Network {
    pub accept_thread_num: usize,

    pub handler_thread_num: usize,

    pub response_thread_num: usize,

    pub queue_size: usize,

    pub lock_max_try_mut_times: u64,

    pub lock_try_mut_sleep_time_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Rocksdb {
    pub data_path: String,
    pub max_open_files: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PlaceRuntime {
    pub heartbeat_timeout_ms: u64,
    pub heartbeat_check_time_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MqttServer {
    pub tcp_port: u32,
    pub tls_port: u32,
    pub websocket_port: u32,
    pub websockets_port: u32,
    pub quic_port: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MqttAuthStorage {
    pub storage_type: String,

    pub journal_addr: String,

    pub mysql_addr: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MqttMessageStorage {
    pub storage_type: String,

    pub journal_addr: String,

    pub mysql_addr: String,

    pub rocksdb_data_path: String,
    pub rocksdb_max_open_files: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MqttRuntime {
    pub default_user: String,

    pub default_password: String,

    pub max_connection_num: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MqttSystemMonitor {
    pub enable: bool,

    pub os_cpu_check_interval_ms: u64,

    pub os_cpu_high_watermark: f32,

    pub os_cpu_low_watermark: f32,

    pub os_memory_check_interval_ms: u64,

    pub os_memory_high_watermark: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MqttOfflineMessage {
    pub enable: bool,

    pub expire_ms: u32,

    pub max_messages_num: u32,
}

impl MqttOfflineMessage {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct MqttSchema {
    pub enable: bool,
    pub strategy: SchemaStrategy,
    pub failed_operation: SchemaFailedOperation,
    pub echo_log: bool,
    pub log_level: String,
}

// MQTT cluster security related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttSecurity {
    pub is_self_protection_status: bool,
    pub secret_free_login: bool,
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

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttFlappingDetect {
    pub enable: bool,
    pub window_time: u32,
    pub max_client_connections: u64,
    pub ban_time: u32,
}

impl MqttFlappingDetect {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttSlowSub {
    pub enable: bool,
    pub whole_ms: u64,
    pub internal_ms: u32,
    pub response_ms: u32,
}
impl MqttSlowSub {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PProf {
    pub enable: bool,
    pub port: u16,
    pub frequency: i32,
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

impl MqttSchema {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}
