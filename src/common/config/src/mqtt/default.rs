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

use super::config::{
    Feature, FlappingDetect, MqttProtocolConfig, NetworkPort, NetworkThread, OfflineMessage,
    Security, SlowSub, System, SystemMonitor,
};
use crate::{
    common::{AvailableFlag, Log, Telemetry},
    mqtt::config::{
        AuthStorage, MessageDataStorage, Schema, SchemaFailedOperation, SchemaStrategy,
    },
};

pub fn default_grpc_port() -> u32 {
    9981
}

pub fn default_heartbeat_timeout() -> String {
    String::from("15s")
}

pub fn default_placement_center() -> Vec<String> {
    vec!["127.0.0.1:1228".to_string()]
}

pub fn default_network_port() -> NetworkPort {
    NetworkPort {
        tcp_port: default_network_tcp_port(),
        tcps_port: default_network_tcps_port(),
        websocket_port: default_network_websocket_port(),
        websockets_port: default_network_websockets_port(),
        quic_port: default_network_quic_port(),
        tls_cert: "".to_string(),
        tls_key: "".to_string(),
    }
}
pub fn default_network_tcp_port() -> u32 {
    1883
}
pub fn default_network_tcps_port() -> u32 {
    1884
}
pub fn default_network_websocket_port() -> u32 {
    8083
}
pub fn default_network_websockets_port() -> u32 {
    8084
}
pub fn default_network_quic_port() -> u32 {
    9083
}

pub fn default_network_thread() -> NetworkThread {
    NetworkThread {
        accept_thread_num: 1,
        handler_thread_num: 1,
        response_thread_num: 1,
        max_connection_num: 5000000,
        request_queue_size: 2000,
        response_queue_size: 2000,
        lock_max_try_mut_times: 30,
        lock_try_mut_sleep_time_ms: 50,
    }
}

pub fn default_system() -> System {
    System {
        runtime_worker_threads: 16,
        default_user: "admin".to_string(),
        default_password: "robustmq".to_string(),
        heartbeat_timeout: "10s".to_string(),
    }
}

pub fn default_system_monitor() -> SystemMonitor {
    SystemMonitor {
        enable: true,
        os_cpu_check_interval_ms: 60000,
        os_cpu_high_watermark: 70.0,
        os_cpu_low_watermark: 50.0,
        os_memory_check_interval_ms: 60,
        os_memory_high_watermark: 80.0,
    }
}

pub fn default_message_storage() -> MessageDataStorage {
    MessageDataStorage {
        storage_type: "memory".to_string(),
        journal_addr: "".to_string(),
        mysql_addr: "".to_string(),
        rocksdb_data_path: "".to_string(),
        rocksdb_max_open_files: None,
    }
}

pub fn default_log() -> Log {
    Log {
        log_path: "./logs".to_string(),
        log_config: "./config/log4rs.yaml".to_string(),
    }
}

pub fn default_offline_message() -> OfflineMessage {
    OfflineMessage {
        enable: false,
        expire_ms: 0,
        max_messages_num: 0,
    }
}

pub fn default_auth_storage() -> AuthStorage {
    AuthStorage {
        storage_type: "placement".to_string(),
        journal_addr: "".to_string(),
        mysql_addr: "".to_string(),
    }
}

pub fn default_telemetry() -> Telemetry {
    Telemetry {
        enable: false,
        exporter_endpoint: "grpc://127.0.0.1:4317".to_string(),
        exporter_type: "otlp".to_string(),
    }
}

pub fn default_feature() -> Feature {
    Feature {
        retain_available: AvailableFlag::Enable,
        wildcard_subscription_available: AvailableFlag::Enable,
        subscription_identifiers_available: AvailableFlag::Enable,
        shared_subscription_available: AvailableFlag::Enable,
        exclusive_subscription_available: AvailableFlag::Enable,
    }
}

pub fn default_security() -> Security {
    Security {
        secret_free_login: false,
        is_self_protection_status: false,
    }
}

pub fn default_protocol() -> MqttProtocolConfig {
    MqttProtocolConfig {
        max_session_expiry_interval: 1800,
        default_session_expiry_interval: 30,
        topic_alias_max: 65535,
        max_qos: 2,
        max_packet_size: 1024 * 1024 * 10,
        max_server_keep_alive: 3600,
        default_server_keep_alive: 60,
        receive_max: 65535,
        client_pkid_persistent: false,
        max_message_expiry_interval: 3600,
    }
}

pub fn default_slow_sub() -> SlowSub {
    SlowSub {
        enable: false,
        whole_ms: 0,
        internal_ms: 0,
        response_ms: 0,
    }
}

pub fn default_flapping_detect() -> FlappingDetect {
    FlappingDetect {
        enable: false,
        window_time: 1,
        max_client_connections: 15,
        ban_time: 5,
    }
}

pub fn default_schema() -> Schema {
    Schema {
        enable: true,
        strategy: SchemaStrategy::ALL,
        failed_operation: SchemaFailedOperation::Discard,
        echo_log: true,
        log_level: "info".to_string(),
    }
}
