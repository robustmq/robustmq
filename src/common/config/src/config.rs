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
    default_accept_thread_num, default_broker_id, default_broker_ip, default_channels_per_address,
    default_cluster_name, default_engine_runtime, default_flapping_ban_time,
    default_flapping_max_connections, default_flapping_window_time, default_grpc_port,
    default_handler_thread_num, default_heartbeat_check_time_ms, default_heartbeat_timeout_ms,
    default_http_port, default_keep_alive_default_time, default_keep_alive_default_timeout,
    default_keep_alive_enable, default_keep_alive_max_time, default_limit_max_connection_rate,
    default_limit_max_connections_per_node, default_limit_max_publish_rate,
    default_limit_max_sessions, default_limit_max_topics, default_max_admin_http_uri_rate,
    default_max_message_expiry_interval, default_max_network_connection,
    default_max_network_connection_rate, default_max_packet_size,
    default_max_session_expiry_interval, default_meta_addrs, default_meta_runtime,
    default_mqtt_flapping_detect, default_mqtt_keep_alive, default_mqtt_limit_cluster,
    default_mqtt_limit_tenant, default_mqtt_offline_message, default_mqtt_protocol,
    default_mqtt_quic_port, default_mqtt_runtime, default_mqtt_runtime_password,
    default_mqtt_runtime_user, default_mqtt_schema, default_mqtt_server,
    default_mqtt_slow_subscribe, default_mqtt_system_monitor, default_mqtt_tcp_port,
    default_mqtt_tls_port, default_mqtt_websocket_port, default_mqtt_websockets_port,
    default_network, default_offline_message_enable, default_offline_message_expire_ms,
    default_offline_message_max_num, default_pprof_frequency, default_pprof_port,
    default_queue_size, default_raft_write_timeout_sec, default_receive_max, default_rocksdb,
    default_rocksdb_data_path, default_rocksdb_max_open_files, default_roles, default_runtime,
    default_runtime_worker_threads, default_schema_echo_log, default_schema_enable,
    default_schema_failed_operation, default_schema_log_level, default_schema_strategy,
    default_session_expiry_interval, default_slow_subscribe_delay_type,
    default_slow_subscribe_record_time, default_storage_io_thread_num,
    default_storage_max_segment_size, default_storage_offset_enable_cache,
    default_storage_tcp_port, default_system_monitor_cpu_watermark,
    default_system_monitor_memory_watermark, default_system_monitor_topic_interval_ms,
    default_tls_cert, default_tls_key, default_topic_alias_max,
};
use crate::common::Log;
use crate::common::Prometheus;
use crate::common::{default_log, default_pprof, default_prometheus};
use common_base::enum_type::delay_type::DelayType;
use serde::{Deserialize, Serialize};
use toml::Table;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LLMPlatform {
    OpenAI,
    OpenAIResp,
    Gemini,
    Anthropic,
    Fireworks,
    Together,
    Groq,
    Mimo,
    Nebius,
    Xai,
    DeepSeek,
    Zai,
    BigModel,
    Cohere,
    Ollama,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LLMClientConfig {
    pub platform: LLMPlatform,
    pub model: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_url: Option<String>,
}

impl LLMClientConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.model.trim().is_empty() {
            return Err("model cannot be empty".to_string());
        }

        if let Some(base_url) = &self.base_url {
            if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
                return Err("base_url must start with http:// or https://".to_string());
            }
        }

        if self.platform != LLMPlatform::Ollama {
            let token = self.token.as_deref().unwrap_or_default().trim();
            if token.is_empty() {
                return Err("token is required for non-ollama platforms".to_string());
            }
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BrokerConfig {
    // Global
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
    pub pprof: PProf,

    #[serde(default = "default_rocksdb")]
    pub rocksdb: Rocksdb,

    #[serde(default)]
    pub llm_client: Option<LLMClientConfig>,

    #[serde(default)]
    pub cluster_limit: ClusterLimit,

    // meta
    #[serde(default = "default_meta_runtime")]
    pub meta_runtime: MetaRuntime,

    // Storage Engine
    #[serde(default = "default_engine_runtime")]
    pub storage_runtime: StorageRuntime,

    // MQTT
    #[serde(default = "default_mqtt_server")]
    pub mqtt_server: MqttServer,

    #[serde(default = "default_mqtt_keep_alive")]
    pub mqtt_keep_alive: MqttKeepAlive,

    #[serde(default = "default_mqtt_runtime")]
    pub mqtt_runtime: MqttRuntime,

    #[serde(default = "default_mqtt_offline_message")]
    pub mqtt_offline_message: MqttOfflineMessage,

    #[serde(default = "default_mqtt_slow_subscribe")]
    pub mqtt_slow_subscribe: MqttSlowSubscribeConfig,

    #[serde(default = "default_mqtt_flapping_detect")]
    pub mqtt_flapping_detect: MqttFlappingDetect,

    #[serde(default = "default_mqtt_protocol")]
    pub mqtt_protocol: MqttProtocolConfig,

    #[serde(default = "default_mqtt_schema")]
    pub mqtt_schema: MqttSchema,

    #[serde(default = "default_mqtt_system_monitor")]
    pub mqtt_system_monitor: MqttSystemMonitor,

    #[serde(default)]
    pub mqtt_limit: MQTTLimit,
}

impl Default for BrokerConfig {
    fn default() -> Self {
        Self {
            // Global
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
            pprof: default_pprof(),
            rocksdb: default_rocksdb(),
            llm_client: None,
            cluster_limit: ClusterLimit::default(),

            // Meta Service
            meta_runtime: default_meta_runtime(),

            // Storage Engine
            storage_runtime: default_engine_runtime(),

            // MQTT Broker
            mqtt_runtime: default_mqtt_runtime(),
            mqtt_server: default_mqtt_server(),
            mqtt_keep_alive: default_mqtt_keep_alive(),
            mqtt_offline_message: default_mqtt_offline_message(),
            mqtt_slow_subscribe: default_mqtt_slow_subscribe(),
            mqtt_flapping_detect: default_mqtt_flapping_detect(),
            mqtt_protocol: default_mqtt_protocol(),
            mqtt_schema: default_mqtt_schema(),
            mqtt_system_monitor: default_mqtt_system_monitor(),
            mqtt_limit: MQTTLimit::default(),
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
        self.mqtt_slow_subscribe.enable
    }

    pub fn get_slow_subscribe_delay_type(&self) -> DelayType {
        self.mqtt_slow_subscribe.delay_type
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Runtime {
    /// Legacy single multiplier kept for backward-compatibility.
    /// When the per-runtime fields below are 0 (auto), this value × num_cpus
    /// is used as a fallback.  Default: 1 (= num_cpus threads per runtime).
    #[serde(default = "default_runtime_worker_threads")]
    pub runtime_worker_threads: usize,

    /// Worker threads for the server runtime (gRPC, HTTP admin, Prometheus).
    /// 0 = auto: num_cpus.
    #[serde(default)]
    pub server_worker_threads: usize,

    /// Worker threads for the meta runtime (Raft state machines, RocksDB).
    /// 0 = auto: num_cpus.
    #[serde(default)]
    pub meta_worker_threads: usize,

    /// Worker threads for the broker runtime (MQTT handler pool, push manager).
    /// 0 = auto: num_cpus.  This is the hot-path runtime.
    #[serde(default)]
    pub broker_worker_threads: usize,

    #[serde(default = "default_channels_per_address")]
    pub channels_per_address: usize,

    #[serde(default = "default_tls_cert")]
    pub tls_cert: String,

    #[serde(default = "default_tls_key")]
    pub tls_key: String,
}

impl Default for Runtime {
    fn default() -> Self {
        default_runtime()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Network {
    #[serde(default = "default_accept_thread_num")]
    pub accept_thread_num: usize,

    #[serde(default = "default_handler_thread_num")]
    pub handler_thread_num: usize,

    #[serde(default = "default_queue_size")]
    pub queue_size: usize,
}

impl Default for Network {
    fn default() -> Self {
        default_network()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClusterLimit {
    #[serde(default = "default_max_network_connection")]
    pub max_network_connection: u64,
    #[serde(default = "default_max_network_connection_rate")]
    pub max_network_connection_rate: u32,
    #[serde(default = "default_max_admin_http_uri_rate")]
    pub max_admin_http_uri_rate: u32,
}

impl Default for ClusterLimit {
    fn default() -> Self {
        ClusterLimit {
            max_network_connection: 100000000,
            max_network_connection_rate: 10000,
            max_admin_http_uri_rate: 50,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LimitQuota {
    #[serde(default = "default_limit_max_connections_per_node")]
    pub max_connections_per_node: u64,
    #[serde(default = "default_limit_max_connection_rate")]
    pub max_connection_rate: u32,
    #[serde(default = "default_limit_max_topics")]
    pub max_topics: u64,
    #[serde(default = "default_limit_max_sessions")]
    pub max_sessions: u64,
    #[serde(default = "default_limit_max_publish_rate")]
    pub max_publish_rate: u32,
}

impl Default for LimitQuota {
    fn default() -> Self {
        LimitQuota {
            max_connections_per_node: 1000000,
            max_connection_rate: 10000,
            max_topics: 500000,
            max_sessions: 5000000,
            max_publish_rate: 10000,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MQTTLimit {
    #[serde(default = "default_mqtt_limit_cluster")]
    pub cluster: LimitQuota,
    #[serde(default = "default_mqtt_limit_tenant")]
    pub tenant: LimitQuota,
}

impl Default for MQTTLimit {
    fn default() -> Self {
        MQTTLimit {
            cluster: LimitQuota {
                max_connections_per_node: 10000000,
                max_connection_rate: 100000,
                max_topics: 5000000,
                max_sessions: 50000000,
                max_publish_rate: 10000,
            },
            tenant: LimitQuota {
                max_connections_per_node: 1000000,
                max_connection_rate: 10000,
                max_topics: 500000,
                max_sessions: 5000000,
                max_publish_rate: 10000,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Rocksdb {
    #[serde(default = "default_rocksdb_data_path")]
    pub data_path: String,
    #[serde(default = "default_rocksdb_max_open_files")]
    pub max_open_files: i32,
}

impl Default for Rocksdb {
    fn default() -> Self {
        default_rocksdb()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetaRuntime {
    #[serde(default = "default_heartbeat_timeout_ms")]
    pub heartbeat_timeout_ms: u64,
    #[serde(default = "default_heartbeat_check_time_ms")]
    pub heartbeat_check_time_ms: u64,
    #[serde(default = "default_raft_write_timeout_sec")]
    pub raft_write_timeout_sec: u64,
    #[serde(default = "default_raft_sharded_group_num")]
    pub offset_raft_group_num: u32,
    #[serde(default = "default_raft_sharded_group_num")]
    pub data_raft_group_num: u32,
}

fn default_raft_sharded_group_num() -> u32 {
    1
}

impl Default for MetaRuntime {
    fn default() -> Self {
        default_meta_runtime()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttServer {
    #[serde(default = "default_mqtt_tcp_port")]
    pub tcp_port: u32,
    #[serde(default = "default_mqtt_tls_port")]
    pub tls_port: u32,
    #[serde(default = "default_mqtt_websocket_port")]
    pub websocket_port: u32,
    #[serde(default = "default_mqtt_websockets_port")]
    pub websockets_port: u32,
    #[serde(default = "default_mqtt_quic_port")]
    pub quic_port: u32,
}

impl Default for MqttServer {
    fn default() -> Self {
        default_mqtt_server()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttKeepAlive {
    #[serde(default = "default_keep_alive_enable")]
    pub enable: bool,
    #[serde(default = "default_keep_alive_default_time")]
    pub default_time: u16,
    #[serde(default = "default_keep_alive_max_time")]
    pub max_time: u16,
    #[serde(default = "default_keep_alive_default_timeout")]
    pub default_timeout: u16,
}

impl Default for MqttKeepAlive {
    fn default() -> Self {
        default_mqtt_keep_alive()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttRuntime {
    #[serde(default = "default_mqtt_runtime_user")]
    pub default_user: String,

    #[serde(default = "default_mqtt_runtime_password")]
    pub default_password: String,

    #[serde(default)]
    pub durable_sessions_enable: bool,

    #[serde(default)]
    pub secret_free_login: bool,

    #[serde(default)]
    pub is_self_protection_status: bool,
}

impl Default for MqttRuntime {
    fn default() -> Self {
        default_mqtt_runtime()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttSystemMonitor {
    #[serde(default)]
    pub enable: bool,

    #[serde(default = "default_system_monitor_cpu_watermark")]
    pub os_cpu_high_watermark: f32,

    #[serde(default = "default_system_monitor_memory_watermark")]
    pub os_memory_high_watermark: f32,

    #[serde(default = "default_system_monitor_topic_interval_ms")]
    pub system_topic_interval_ms: u64,
}

impl Default for MqttSystemMonitor {
    fn default() -> Self {
        default_mqtt_system_monitor()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MqttOfflineMessage {
    #[serde(default = "default_offline_message_enable")]
    pub enable: bool,

    #[serde(default = "default_offline_message_expire_ms")]
    pub expire_ms: u32,

    #[serde(default = "default_offline_message_max_num")]
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
    #[serde(default = "default_schema_enable")]
    pub enable: bool,
    #[serde(default = "default_schema_strategy")]
    pub strategy: SchemaStrategy,
    #[serde(default = "default_schema_failed_operation")]
    pub failed_operation: SchemaFailedOperation,
    #[serde(default = "default_schema_echo_log")]
    pub echo_log: bool,
    #[serde(default = "default_schema_log_level")]
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

// MQTT cluster protocol related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttProtocolConfig {
    #[serde(default = "default_max_session_expiry_interval")]
    pub max_session_expiry_interval: u32,
    #[serde(default = "default_session_expiry_interval")]
    pub default_session_expiry_interval: u32,
    #[serde(default = "default_topic_alias_max")]
    pub topic_alias_max: u16,
    #[serde(default = "default_max_packet_size")]
    pub max_packet_size: u32,
    #[serde(default = "default_receive_max")]
    pub receive_max: u16,
    #[serde(default = "default_max_message_expiry_interval")]
    pub max_message_expiry_interval: u64,
    #[serde(default)]
    pub client_pkid_persistent: bool,
}

impl Default for MqttProtocolConfig {
    fn default() -> Self {
        default_mqtt_protocol()
    }
}

impl MqttProtocolConfig {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).expect("Failed to serialize MqttProtocolConfig")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MqttFlappingDetect {
    #[serde(default)]
    pub enable: bool,
    #[serde(default = "default_flapping_window_time")]
    pub window_time: u32,
    #[serde(default = "default_flapping_max_connections")]
    pub max_client_connections: u64,
    #[serde(default = "default_flapping_ban_time")]
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
    #[serde(default)]
    pub enable: bool,
    #[serde(default = "default_slow_subscribe_record_time")]
    pub record_time: u64,
    #[serde(default = "default_slow_subscribe_delay_type")]
    pub delay_type: DelayType,
}

impl Default for MqttSlowSubscribeConfig {
    fn default() -> Self {
        default_mqtt_slow_subscribe()
    }
}

impl MqttSlowSubscribeConfig {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).expect("Failed to serialize MqttSlowSubscribeConfig")
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PProf {
    #[serde(default)]
    pub enable: bool,
    #[serde(default = "default_pprof_port")]
    pub port: u16,
    #[serde(default = "default_pprof_frequency")]
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
    #[serde(default = "default_storage_tcp_port")]
    pub tcp_port: u32,
    #[serde(default = "default_storage_max_segment_size")]
    pub max_segment_size: u32,
    #[serde(default = "default_storage_io_thread_num")]
    pub io_thread_num: u32,
    #[serde(default)]
    pub data_path: Vec<String>,
    #[serde(default = "default_storage_offset_enable_cache")]
    pub offset_enable_cache: bool,
}

impl Default for StorageRuntime {
    fn default() -> Self {
        default_engine_runtime()
    }
}
