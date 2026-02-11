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
    default_broker_id, default_broker_ip, default_cluster_name, default_engine_runtime,
    default_grpc_port, default_http_port, default_message_storage, default_meta_addrs,
    default_meta_runtime, default_mqtt_auth_config, default_mqtt_flapping_detect,
    default_mqtt_keep_alive, default_mqtt_offline_message, default_mqtt_protocol_config,
    default_mqtt_runtime, default_mqtt_schema, default_mqtt_security, default_mqtt_server,
    default_mqtt_slow_subscribe_config, default_mqtt_system_monitor, default_network,
    default_rocksdb, default_roles, default_runtime, default_storage_offset,
};
use super::security::{AuthnConfig, AuthzConfig};
use crate::common::Log;
use crate::common::Prometheus;
use crate::common::{default_log, default_pprof, default_prometheus};
use crate::storage::StorageAdapterConfig;
use common_base::enum_type::delay_type::DelayType;
use serde::{Deserialize, Serialize};
use toml::Table;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BrokerConfig {
    // Base
    #[serde(default = "default_cluster_name")]
    pub cluster_name: String,

    #[serde(default = "default_broker_id")]
    pub broker_id: u64,

    #[serde(default = "default_broker_ip")]
    pub broker_ip: Option<String>,

    #[serde(default = "default_roles")]
    pub roles: Vec<String>,

    #[serde(default = "default_grpc_port")]
    pub grpc_port: u32,

    #[serde(default = "default_http_port")]
    pub http_port: u32,

    #[serde(default = "default_meta_addrs")]
    pub meta_addrs: Table,

    #[serde(default = "default_prometheus")]
    pub prometheus: Prometheus,

    #[serde(default = "default_log")]
    pub log: Log,

    #[serde(default = "default_runtime")]
    pub runtime: Runtime,

    #[serde(default = "default_network")]
    pub network: Network,

    #[serde(default = "default_pprof")]
    pub p_prof: PProf,

    #[serde(default = "default_message_storage")]
    pub message_storage: StorageAdapterConfig,

    // meta
    #[serde(default = "default_meta_runtime")]
    pub meta_runtime: MetaRuntime,

    #[serde(default = "default_rocksdb")]
    pub rocksdb: Rocksdb,

    #[serde(default = "default_engine_runtime")]
    pub storage_runtime: StorageRuntime,

    // MQTT
    #[serde(default = "default_mqtt_server")]
    pub mqtt_server: MqttServer,

    #[serde(default = "default_mqtt_keep_alive")]
    pub mqtt_keep_alive: MqttKeepAlive,

    #[serde(default = "default_mqtt_auth_config")]
    pub mqtt_auth_config: MqttAuthConfig,

    #[serde(default = "default_mqtt_runtime")]
    pub mqtt_runtime: MqttRuntime,

    #[serde(default = "default_mqtt_offline_message")]
    pub mqtt_offline_message: MqttOfflineMessage,

    #[serde(default = "default_mqtt_slow_subscribe_config")]
    pub mqtt_slow_subscribe_config: MqttSlowSubscribeConfig,

    #[serde(default = "default_mqtt_flapping_detect")]
    pub mqtt_flapping_detect: MqttFlappingDetect,

    #[serde(default = "default_mqtt_protocol_config")]
    pub mqtt_protocol_config: MqttProtocolConfig,

    #[serde(default = "default_mqtt_security")]
    pub mqtt_security: MqttSecurity,

    #[serde(default = "default_mqtt_schema")]
    pub mqtt_schema: MqttSchema,

    #[serde(default = "default_mqtt_system_monitor")]
    pub mqtt_system_monitor: MqttSystemMonitor,

    #[serde(default = "default_storage_offset")]
    pub storage_offset: StorageOffset,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            cluster_name: default_cluster_name(),
            broker_id: default_broker_id(),
            broker_ip: default_broker_ip(),
            roles: default_roles(),
            grpc_port: default_grpc_port(),
            http_port: default_http_port(),
            meta_addrs: default_meta_addrs(),
            prometheus: default_prometheus(),
            log: default_log(),
            runtime: default_runtime(),
            network: default_network(),
            p_prof: default_pprof(),
            message_storage: default_message_storage(),
            meta_runtime: default_meta_runtime(),
            rocksdb: default_rocksdb(),
            storage_runtime: default_engine_runtime(),
            mqtt_server: default_mqtt_server(),
            mqtt_keep_alive: default_mqtt_keep_alive(),
            mqtt_auth_config: default_mqtt_auth_config(),
            mqtt_runtime: default_mqtt_runtime(),
            mqtt_offline_message: default_mqtt_offline_message(),
            mqtt_slow_subscribe_config: default_mqtt_slow_subscribe_config(),
            mqtt_flapping_detect: default_mqtt_flapping_detect(),
            mqtt_protocol_config: default_mqtt_protocol_config(),
            mqtt_security: default_mqtt_security(),
            mqtt_schema: default_mqtt_schema(),
            mqtt_system_monitor: default_mqtt_system_monitor(),
            storage_offset: default_storage_offset(),
        }
    }
}

impl BrokerConfig {
    pub fn get_meta_service_addr(&self) -> Vec<String> {
        self.meta_addrs
            .values()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    }

    pub fn is_enable_slow_subscribe_record(&self) -> bool {
        self.mqtt_slow_subscribe_config.enable
    }

    pub fn get_slow_subscribe_delay_type(&self) -> DelayType {
        self.mqtt_slow_subscribe_config.delay_type
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Runtime {
    pub runtime_worker_threads: usize,

    pub tls_cert: String,

    pub tls_key: String,
}

impl Default for Runtime {
    fn default() -> Self {
        default_runtime()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Network {
    pub accept_thread_num: usize,

    pub handler_thread_num: usize,

    pub response_thread_num: usize,

    pub queue_size: usize,

    pub lock_max_try_mut_times: u64,

    pub lock_try_mut_sleep_time_ms: u64,
}

impl Default for Network {
    fn default() -> Self {
        default_network()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rocksdb {
    pub data_path: String,
    pub max_open_files: i32,
}

impl Default for Rocksdb {
    fn default() -> Self {
        default_rocksdb()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetaRuntime {
    pub heartbeat_timeout_ms: u64,
    pub heartbeat_check_time_ms: u64,
    pub raft_write_timeout_sec: u64,
}

impl Default for MetaRuntime {
    fn default() -> Self {
        default_meta_runtime()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttServer {
    pub tcp_port: u32,
    pub tls_port: u32,
    pub websocket_port: u32,
    pub websockets_port: u32,
    pub quic_port: u32,
}

impl Default for MqttServer {
    fn default() -> Self {
        default_mqtt_server()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttAuthConfig {
    pub authn_config: AuthnConfig,
    pub authz_config: AuthzConfig,
}

impl Default for MqttAuthConfig {
    fn default() -> Self {
        default_mqtt_auth_config()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttKeepAlive {
    pub enable: bool,
    pub default_time: u16,
    pub max_time: u16,
    pub default_timeout: u16,
}

impl Default for MqttKeepAlive {
    fn default() -> Self {
        default_mqtt_keep_alive()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttRuntime {
    pub default_user: String,

    pub default_password: String,

    pub max_connection_num: usize,

    pub durable_sessions_enable: bool,
}

impl Default for MqttRuntime {
    fn default() -> Self {
        default_mqtt_runtime()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttSystemMonitor {
    pub enable: bool,

    pub os_cpu_high_watermark: f32,

    pub os_memory_high_watermark: f32,
}

impl Default for MqttSystemMonitor {
    fn default() -> Self {
        default_mqtt_system_monitor()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct StorageOffset {
    pub enable_cache: bool,
}

impl Default for StorageOffset {
    fn default() -> Self {
        default_storage_offset()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttOfflineMessage {
    pub enable: bool,

    pub expire_ms: u32,

    pub max_messages_num: u32,
}

impl Default for MqttOfflineMessage {
    fn default() -> Self {
        default_mqtt_offline_message()
    }
}

impl MqttOfflineMessage {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).expect("Failed to serialize MqttOfflineMessage")
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MqttSchema {
    pub enable: bool,
    pub strategy: SchemaStrategy,
    pub failed_operation: SchemaFailedOperation,
    pub echo_log: bool,
    pub log_level: String,
}

impl Default for MqttSchema {
    fn default() -> Self {
        default_mqtt_schema()
    }
}

impl MqttSchema {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).expect("Failed to serialize MqttSchema")
    }
}

// MQTT cluster security related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttSecurity {
    pub is_self_protection_status: bool,
    pub secret_free_login: bool,
}

impl Default for MqttSecurity {
    fn default() -> Self {
        default_mqtt_security()
    }
}

// MQTT cluster protocol related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttProtocolConfig {
    pub max_session_expiry_interval: u32,
    pub default_session_expiry_interval: u32,
    pub topic_alias_max: u16,
    pub max_qos_flight_message: u8,
    pub max_packet_size: u32,
    pub receive_max: u16,
    pub max_message_expiry_interval: u64,
    pub client_pkid_persistent: bool,
}

impl Default for MqttProtocolConfig {
    fn default() -> Self {
        default_mqtt_protocol_config()
    }
}

impl MqttProtocolConfig {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).expect("Failed to serialize MqttProtocolConfig")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttFlappingDetect {
    pub enable: bool,
    pub window_time: u32,
    pub max_client_connections: u64,
    pub ban_time: u32,
}

impl Default for MqttFlappingDetect {
    fn default() -> Self {
        default_mqtt_flapping_detect()
    }
}

impl MqttFlappingDetect {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).expect("Failed to serialize MqttFlappingDetect")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttSlowSubscribeConfig {
    pub enable: bool,
    pub record_time: u64,
    pub delay_type: DelayType,
}

impl Default for MqttSlowSubscribeConfig {
    fn default() -> Self {
        default_mqtt_slow_subscribe_config()
    }
}

impl MqttSlowSubscribeConfig {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).expect("Failed to serialize MqttSlowSubscribeConfig")
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PProf {
    pub enable: bool,
    pub port: u16,
    pub frequency: i32,
}

impl Default for PProf {
    fn default() -> Self {
        default_pprof()
    }
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

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StorageRuntime {
    pub tcp_port: u32,
    pub max_segment_size: u32,
    pub io_thread_num: u32,
    pub data_path: Vec<String>,
}

impl Default for StorageRuntime {
    fn default() -> Self {
        default_engine_runtime()
    }
}
