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

use core::str;

use super::default::{
    default_broker_id, default_cluster_name, default_flapping_detect, default_grpc_port,
    default_http_port, default_journal_runtime, default_journal_server, default_journal_storage,
    default_message_storage, default_meta_addrs, default_mqtt_auth_config, default_mqtt_keep_alive,
    default_mqtt_offline_message, default_mqtt_protocol_config, default_mqtt_runtime,
    default_mqtt_schema, default_mqtt_security, default_mqtt_server,
    default_mqtt_slow_subscribe_config, default_mqtt_system_monitor, default_network,
    default_place_runtime, default_rocksdb, default_roles, default_runtime, default_storage_offset,
};
use super::security::{AuthnConfig, AuthzConfig};
use crate::common::Log;
use crate::common::Prometheus;
use crate::common::{default_log, default_pprof, default_prometheus};
use crate::storage::StorageAdapterConfig;
use common_base::enum_type::delay_type::DelayType;
use serde::{Deserialize, Serialize};
use toml::Table;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BrokerConfig {
    // Base
    #[serde(default = "default_cluster_name")]
    pub cluster_name: String,

    #[serde(default = "default_broker_id")]
    pub broker_id: u64,

    #[serde(default)]
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
    #[serde(default = "default_place_runtime")]
    pub meta_runtime: MetaRuntime,

    #[serde(default = "default_rocksdb")]
    pub rocksdb: Rocksdb,

    // Journal Engine
    #[serde(default = "default_journal_server")]
    pub journal_server: JournalServer,

    #[serde(default = "default_journal_runtime")]
    pub journal_runtime: JournalRuntime,

    #[serde(default = "default_journal_storage")]
    pub journal_storage: JournalStorage,

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

    #[serde(default = "default_storage_offset")]
    pub storage_offset: StorageOffset,
}

impl BrokerConfig {
    pub fn get_meta_service_addr(&self) -> Vec<String> {
        self.meta_addrs
            .values()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    }

    pub fn is_start_meta(&self) -> bool {
        self.roles.contains(&"meta".to_string())
    }

    pub fn is_start_journal(&self) -> bool {
        self.roles.contains(&"journal".to_string())
    }

    pub fn is_start_broker(&self) -> bool {
        self.roles.contains(&"broker".to_string())
    }

    pub fn is_enable_slow_subscribe_record(&self) -> bool {
        self.mqtt_slow_subscribe_config.enable
    }

    pub fn get_slow_subscribe_delay_type(&self) -> DelayType {
        self.mqtt_slow_subscribe_config.delay_type
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
pub struct MetaRuntime {
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
pub struct MqttAuthConfig {
    pub authn_config: AuthnConfig,
    pub authz_config: AuthzConfig,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct MqttKeepAlive {
    pub enable: bool,
    pub default_time: u16,
    pub max_time: u16,
    pub default_timeout: u16,
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

    pub os_cpu_high_watermark: f32,

    pub os_memory_high_watermark: f32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct StorageOffset {
    pub enable_cache: bool,
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
pub struct MqttSlowSubscribeConfig {
    pub enable: bool,
    pub record_time: u64,
    pub delay_type: DelayType,
}

impl MqttSlowSubscribeConfig {
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

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct JournalRuntime {
    pub enable_auto_create_shard: bool,
    pub shard_replica_num: u32,
    pub max_segment_size: u32,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct JournalStorage {
    pub data_path: Vec<String>,
    pub rocksdb_max_open_files: i32,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct JournalServer {
    pub tcp_port: u32,
}
