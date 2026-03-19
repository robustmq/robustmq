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
    default_grpc_port, default_http_port, default_meta_addrs, default_meta_runtime,
    default_mqtt_flapping_detect, default_mqtt_keep_alive, default_mqtt_offline_message,
    default_mqtt_protocol, default_mqtt_runtime, default_mqtt_schema, default_mqtt_server,
    default_mqtt_slow_subscribe, default_mqtt_system_monitor, default_network, default_rocksdb,
    default_roles, default_runtime, default_runtime_worker_threads,
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

    #[serde(default)]
    pub channels_per_address: usize,

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

    pub queue_size: usize,
}

impl Default for Network {
    fn default() -> Self {
        default_network()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ClusterLimit {
    pub max_network_connection: u64,
    pub max_network_connection_rate: u32,
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
    pub max_connections_per_node: u64,
    pub max_connection_rate: u32,
    pub max_topics: u64,
    pub max_sessions: u64,
    pub max_mqtt_qos1_num: u64,
    pub max_mqtt_qos2_num: u64,
    pub max_publish_rate: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MQTTLimit {
    pub cluster: LimitQuota,
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
                max_mqtt_qos1_num: 1000,
                max_mqtt_qos2_num: 1000,
                max_publish_rate: 10000,
            },
            tenant: LimitQuota {
                max_connections_per_node: 1000000,
                max_connection_rate: 10000,
                max_topics: 500000,
                max_sessions: 5000000,
                max_mqtt_qos1_num: 1000,
                max_mqtt_qos2_num: 1000,
                max_publish_rate: 10000,
            },
        }
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

    pub durable_sessions_enable: bool,

    pub secret_free_login: bool,

    pub is_self_protection_status: bool,
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

    pub system_topic_interval_ms: u64,
}

impl Default for MqttSystemMonitor {
    fn default() -> Self {
        default_mqtt_system_monitor()
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
    pub offset_enable_cache: bool,
}

impl Default for StorageRuntime {
    fn default() -> Self {
        default_engine_runtime()
    }
}
