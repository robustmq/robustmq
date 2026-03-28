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

use crate::config::{
    AmqpRuntime, DelayTask, KafkaRuntime, MetaRuntime, MqttFlappingDetect, MqttKeepAlive,
    MqttOfflineMessage, MqttProtocolConfig, MqttRuntime, MqttSchema, MqttServer,
    MqttSlowSubscribeConfig, MqttSystemMonitor, NatsRuntime, Network, Rocksdb, Runtime,
    SchemaFailedOperation, SchemaStrategy, StorageRuntime,
};
use crate::storage::{StorageAdapterConfig, StorageType};
use common_base::enum_type::delay_type::DelayType;
use common_base::role::{ROLE_BROKER, ROLE_META};
use common_base::runtime::get_default_runtime_worker_threads;
use common_base::tools::get_local_ip;
use toml::Table;

pub fn default_runtime_worker_threads() -> usize {
    get_default_runtime_worker_threads()
}

pub fn default_roles() -> Vec<String> {
    vec![ROLE_BROKER.to_string(), ROLE_META.to_string()]
}

pub fn default_cluster_name() -> String {
    "robust_mq_cluster_default".to_string()
}

pub fn default_broker_id() -> u64 {
    1
}

pub fn default_grpc_port() -> u32 {
    1228
}

pub fn default_broker_ip() -> Option<String> {
    Some(get_local_ip())
}

pub fn default_http_port() -> u32 {
    8080
}

pub fn default_meta_addrs() -> Table {
    let mut nodes = Table::new();
    nodes.insert(
        default_broker_id().to_string(),
        toml::Value::String(format!("127.0.0.1:{}", default_grpc_port())),
    );
    nodes
}

pub fn default_runtime() -> Runtime {
    Runtime {
        runtime_worker_threads: get_default_runtime_worker_threads(),
        server_worker_threads: 0,
        meta_worker_threads: 0,
        broker_worker_threads: 0,
        channels_per_address: 4,
        tls_cert: "./config/certs/cert.pem".to_string(),
        tls_key: "./config/certs/key.pem".to_string(),
    }
}

pub fn default_network() -> Network {
    Network {
        accept_thread_num: 1,
        handler_thread_num: 64,
        queue_size: 5000,
    }
}

pub fn default_rocksdb() -> Rocksdb {
    Rocksdb {
        max_open_files: 10000,
        data_path: "./data".to_string(),
    }
}

pub fn default_meta_runtime() -> MetaRuntime {
    MetaRuntime {
        heartbeat_check_time_ms: 1000,
        heartbeat_timeout_ms: 30000,
        raft_write_timeout_sec: 30,
        offset_raft_group_num: 1,
        data_raft_group_num: 1,
    }
}

pub fn default_mqtt_server() -> MqttServer {
    MqttServer {
        tcp_port: 1883,
        tls_port: 1885,
        websocket_port: 8083,
        websockets_port: 8085,
        quic_port: 9083,
    }
}

pub fn default_mqtt_keep_alive() -> MqttKeepAlive {
    MqttKeepAlive {
        enable: true,
        max_time: 3600,
        default_time: 180,
        default_timeout: 2,
    }
}

pub fn default_message_storage() -> StorageAdapterConfig {
    StorageAdapterConfig {
        storage_type: StorageType::EngineMemory,
        ..Default::default()
    }
}

pub fn default_mqtt_runtime_user() -> String {
    "admin".to_string()
}

pub fn default_mqtt_runtime_password() -> String {
    "robustmq".to_string()
}

pub fn default_mqtt_runtime() -> MqttRuntime {
    MqttRuntime {
        default_user: "admin".to_string(),
        default_password: "robustmq".to_string(),
        durable_sessions_enable: false, // Default: transient sessions (better performance)
        secret_free_login: false,
        is_self_protection_status: false,
        network: default_network(),
    }
}

pub fn default_mqtt_offline_message() -> MqttOfflineMessage {
    MqttOfflineMessage {
        enable: true,
        expire_ms: 0,
        max_messages_num: 0,
    }
}

pub fn default_mqtt_slow_subscribe() -> MqttSlowSubscribeConfig {
    MqttSlowSubscribeConfig {
        enable: false,
        record_time: 1000,
        delay_type: DelayType::Whole,
    }
}

pub fn default_mqtt_flapping_detect() -> MqttFlappingDetect {
    MqttFlappingDetect {
        enable: false,
        window_time: 1,
        max_client_connections: 15,
        ban_time: 5,
    }
}

pub fn default_mqtt_protocol() -> MqttProtocolConfig {
    MqttProtocolConfig {
        max_session_expiry_interval: 1800,
        default_session_expiry_interval: 30,
        topic_alias_max: 65535,
        max_packet_size: 1024 * 1024 * 10,
        receive_max: 65535,
        client_pkid_persistent: false,
        max_message_expiry_interval: 3600,
    }
}

pub fn default_mqtt_schema() -> MqttSchema {
    MqttSchema {
        enable: true,
        strategy: SchemaStrategy::ALL,
        failed_operation: SchemaFailedOperation::Discard,
        echo_log: true,
        log_level: "info".to_string(),
    }
}

pub fn default_mqtt_system_monitor() -> MqttSystemMonitor {
    MqttSystemMonitor {
        enable: false,
        os_cpu_high_watermark: 70.0,
        os_memory_high_watermark: 80.0,
        system_topic_interval_ms: 60000,
    }
}

pub fn default_engine_runtime() -> StorageRuntime {
    StorageRuntime {
        tcp_port: 1778,
        max_segment_size: 1073741824,
        data_path: vec![],
        io_thread_num: 8,
        offset_enable_cache: true,
        network: default_network(),
    }
}

pub fn default_kafka_runtime() -> KafkaRuntime {
    KafkaRuntime {
        network: default_network(),
    }
}

pub fn default_amqp_runtime() -> AmqpRuntime {
    AmqpRuntime {
        network: default_network(),
    }
}

pub fn default_nats_runtime() -> NatsRuntime {
    NatsRuntime {
        network: default_network(),
    }
}

// Runtime
pub fn default_tls_cert() -> String {
    "./config/certs/cert.pem".to_string()
}
pub fn default_tls_key() -> String {
    "./config/certs/key.pem".to_string()
}
pub fn default_channels_per_address() -> usize {
    4
}

// Network
pub fn default_accept_thread_num() -> usize {
    1
}
pub fn default_handler_thread_num() -> usize {
    64
}
pub fn default_queue_size() -> usize {
    2000
}

// Rocksdb
pub fn default_rocksdb_data_path() -> String {
    "./data".to_string()
}
pub fn default_rocksdb_max_open_files() -> i32 {
    10000
}

// MetaRuntime
pub fn default_heartbeat_timeout_ms() -> u64 {
    30000
}
pub fn default_heartbeat_check_time_ms() -> u64 {
    1000
}
pub fn default_raft_write_timeout_sec() -> u64 {
    30
}

// MqttServer
pub fn default_mqtt_tcp_port() -> u32 {
    1883
}
pub fn default_mqtt_tls_port() -> u32 {
    1885
}
pub fn default_mqtt_websocket_port() -> u32 {
    8083
}
pub fn default_mqtt_websockets_port() -> u32 {
    8085
}
pub fn default_mqtt_quic_port() -> u32 {
    9083
}

// MqttKeepAlive
pub fn default_keep_alive_enable() -> bool {
    true
}
pub fn default_keep_alive_default_time() -> u16 {
    180
}
pub fn default_keep_alive_max_time() -> u16 {
    3600
}
pub fn default_keep_alive_default_timeout() -> u16 {
    2
}

// MqttSystemMonitor
pub fn default_system_monitor_cpu_watermark() -> f32 {
    70.0
}
pub fn default_system_monitor_memory_watermark() -> f32 {
    80.0
}
pub fn default_system_monitor_topic_interval_ms() -> u64 {
    60000
}

// MqttOfflineMessage
pub fn default_offline_message_enable() -> bool {
    true
}
pub fn default_offline_message_expire_ms() -> u32 {
    3_600_000 // 1 hour; 0 = never expire
}
pub fn default_offline_message_max_num() -> u32 {
    100_000 // 0 = unlimited
}

// MqttSchema
pub fn default_schema_enable() -> bool {
    true
}
pub fn default_schema_strategy() -> SchemaStrategy {
    SchemaStrategy::ALL
}
pub fn default_schema_failed_operation() -> SchemaFailedOperation {
    SchemaFailedOperation::Discard
}
pub fn default_schema_echo_log() -> bool {
    true
}
pub fn default_schema_log_level() -> String {
    "info".to_string()
}

// MqttProtocolConfig
pub fn default_max_session_expiry_interval() -> u32 {
    1800
}
pub fn default_session_expiry_interval() -> u32 {
    30
}
pub fn default_topic_alias_max() -> u16 {
    65535
}

pub fn default_max_packet_size() -> u32 {
    1024 * 1024 * 10
}
pub fn default_receive_max() -> u16 {
    65535
}
pub fn default_max_message_expiry_interval() -> u64 {
    3600
}

// MqttFlappingDetect
pub fn default_flapping_window_time() -> u32 {
    1
}
pub fn default_flapping_max_connections() -> u64 {
    15
}
pub fn default_flapping_ban_time() -> u32 {
    5
}

// MqttSlowSubscribeConfig
pub fn default_slow_subscribe_record_time() -> u64 {
    1000
}
pub fn default_slow_subscribe_delay_type() -> DelayType {
    DelayType::Whole
}

// PProf
pub fn default_pprof_port() -> u16 {
    9090
}
pub fn default_pprof_frequency() -> i32 {
    100
}

// StorageRuntime
pub fn default_storage_tcp_port() -> u32 {
    1778
}
pub fn default_storage_max_segment_size() -> u32 {
    1073741824
}
pub fn default_storage_io_thread_num() -> u32 {
    8
}
pub fn default_storage_offset_enable_cache() -> bool {
    true
}

// ClusterLimit
pub fn default_max_network_connection() -> u64 {
    100000000
}
pub fn default_max_network_connection_rate() -> u32 {
    10000
}
pub fn default_max_admin_http_uri_rate() -> u32 {
    50
}

// LimitQuota
pub fn default_limit_max_connections_per_node() -> u64 {
    1000000
}
pub fn default_limit_max_connection_rate() -> u32 {
    10000
}
pub fn default_limit_max_topics() -> u64 {
    500000
}
pub fn default_limit_max_sessions() -> u64 {
    5000000
}
pub fn default_limit_max_publish_rate() -> u32 {
    10000
}

// MQTTLimit — cluster and tenant have different default quotas
pub fn default_mqtt_limit_cluster() -> crate::config::LimitQuota {
    crate::config::LimitQuota {
        max_connections_per_node: 10_000_000,
        max_connection_rate: 100_000,
        max_topics: 5_000_000,
        max_sessions: 50_000_000,
        max_publish_rate: 10_000,
    }
}
pub fn default_mqtt_limit_tenant() -> crate::config::LimitQuota {
    crate::config::LimitQuota {
        max_connections_per_node: 1_000_000,
        max_connection_rate: 10_000,
        max_topics: 500_000,
        max_sessions: 5_000_000,
        max_publish_rate: 10_000,
    }
}

// DelayTask
pub fn default_delay_task() -> DelayTask {
    DelayTask {
        delay_task_queue_num: default_delay_task_queue_num(),
        delay_task_handler_concurrency: default_delay_task_handler_concurrency(),
    }
}

pub fn default_delay_task_queue_num() -> usize {
    8
}

pub fn default_delay_task_handler_concurrency() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}
