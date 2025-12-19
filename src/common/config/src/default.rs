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

use super::security::{AuthnConfig, AuthzConfig};
use crate::config::{
    MetaRuntime, MqttAuthConfig, MqttFlappingDetect, MqttKeepAlive, MqttOfflineMessage,
    MqttProtocolConfig, MqttRuntime, MqttSchema, MqttSecurity, MqttServer, MqttSlowSubscribeConfig,
    MqttSystemMonitor, Network, Rocksdb, Runtime, SchemaFailedOperation, SchemaStrategy,
    StorageOffset, StorageRuntime,
};
use crate::storage::{StorageAdapterConfig, StorageAdapterType};
use common_base::enum_type::delay_type::DelayType;
use common_base::runtime::get_runtime_worker_threads;
use common_base::tools::get_local_ip;
use toml::Table;

pub fn default_roles() -> Vec<String> {
    vec!["meta".to_string(), "broker".to_string()]
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
        runtime_worker_threads: get_runtime_worker_threads(),
        tls_cert: "./config/certs/cert.pem".to_string(),
        tls_key: "./config/certs/key.pem".to_string(),
    }
}

pub fn default_network() -> Network {
    Network {
        accept_thread_num: 8,
        handler_thread_num: 32,
        response_thread_num: 8,
        queue_size: 1000,
        lock_max_try_mut_times: 30,
        lock_try_mut_sleep_time_ms: 50,
    }
}

pub fn default_rocksdb() -> Rocksdb {
    Rocksdb {
        max_open_files: 10000,
        data_path: "./data".to_string(),
    }
}

pub fn default_place_runtime() -> MetaRuntime {
    MetaRuntime {
        heartbeat_check_time_ms: 1000,
        heartbeat_timeout_ms: 30000,
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

pub fn default_mqtt_auth_config() -> MqttAuthConfig {
    MqttAuthConfig {
        authn_config: AuthnConfig::default(),
        authz_config: AuthzConfig::default(),
    }
}

pub fn default_message_storage() -> StorageAdapterConfig {
    StorageAdapterConfig {
        storage_type: StorageAdapterType::Memory,
        ..Default::default()
    }
}

pub fn default_mqtt_runtime() -> MqttRuntime {
    MqttRuntime {
        default_user: "admin".to_string(),
        default_password: "robustmq".to_string(),
        max_connection_num: 1000000,
    }
}

pub fn default_mqtt_offline_message() -> MqttOfflineMessage {
    MqttOfflineMessage {
        enable: true,
        expire_ms: 0,
        max_messages_num: 0,
    }
}

pub fn default_mqtt_slow_subscribe_config() -> MqttSlowSubscribeConfig {
    MqttSlowSubscribeConfig {
        enable: false,
        record_time: 1000,
        delay_type: DelayType::Whole,
    }
}

pub fn default_flapping_detect() -> MqttFlappingDetect {
    MqttFlappingDetect {
        enable: false,
        window_time: 1,
        max_client_connections: 15,
        ban_time: 5,
    }
}

pub fn default_mqtt_protocol_config() -> MqttProtocolConfig {
    MqttProtocolConfig {
        max_session_expiry_interval: 1800,
        default_session_expiry_interval: 30,
        topic_alias_max: 65535,
        max_qos: 2,
        max_packet_size: 1024 * 1024 * 10,
        receive_max: 65535,
        client_pkid_persistent: false,
        max_message_expiry_interval: 3600,
    }
}

pub fn default_mqtt_security() -> MqttSecurity {
    MqttSecurity {
        secret_free_login: false,
        is_self_protection_status: false,
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
    }
}

pub fn default_storage_offset() -> StorageOffset {
    StorageOffset { enable_cache: true }
}

pub fn default_journal_runtime() -> StorageRuntime {
    StorageRuntime {
        tcp_port: 1778,
        max_segment_size: 1073741824,
        data_path: vec![],
    }
}
