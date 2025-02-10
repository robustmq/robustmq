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

/*
 * Copyright (c) 2023 RobustMQ Team
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::common::{override_default_by_env, Auth, Log, Storage, Telemetry};
use super::default_mqtt::{
    default_auth, default_grpc_port, default_http_port, default_log, default_network,
    default_network_quic_port, default_network_tcp_port, default_network_tcps_port,
    default_network_websocket_port, default_network_websockets_port, default_offline_message,
    default_placement_center, default_storage, default_system, default_tcp_thread,
    default_telemetry,
};
use crate::tools::{read_file, try_create_fold};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BrokerMqttConfig {
    pub cluster_name: String,
    pub broker_id: u64,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u32,
    #[serde(default = "default_http_port")]
    pub http_port: usize,
    #[serde(default = "default_placement_center")]
    pub placement_center: Vec<String>,
    #[serde(default = "default_network")]
    pub network: Network,
    #[serde(default = "default_tcp_thread")]
    pub tcp_thread: TcpThread,
    #[serde(default = "default_system")]
    pub system: System,
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

static BROKER_MQTT_CONF: OnceLock<BrokerMqttConfig> = OnceLock::new();

pub fn init_broker_mqtt_conf_by_path(config_path: &str) -> &'static BrokerMqttConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    BROKER_MQTT_CONF.get_or_init(|| {
        let content = match read_file(config_path) {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string())
            }
        };
        let new_content = override_default_by_env(content, "MQTT_SERVER");
        let config: BrokerMqttConfig = match toml::from_str(&new_content) {
            Ok(da) => da,
            Err(e) => {
                panic!("{}", e)
            }
        };
        match try_create_fold(&config.log.log_path) {
            Ok(()) => {}
            Err(e) => {
                panic!("{}", e);
            }
        }
        config
    })
}

pub fn init_broker_mqtt_conf_by_config(config: BrokerMqttConfig) -> &'static BrokerMqttConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    BROKER_MQTT_CONF.get_or_init(|| config)
}

pub fn broker_mqtt_conf() -> &'static BrokerMqttConfig {
    match BROKER_MQTT_CONF.get() {
        Some(config) => config,
        None => {
            panic!("MQTT Broker configuration is not initialized, check the configuration file.");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        broker_mqtt_conf, init_broker_mqtt_conf_by_path, override_default_by_env, BrokerMqttConfig,
    };
    use crate::tools::read_file;

    #[test]
    fn config_default_test() {
        let path = format!(
            "{}/../../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );

        let content = read_file(&path).unwrap();
        println!("{}", content);
        let config: BrokerMqttConfig = match toml::from_str(&content) {
            Ok(da) => da,
            Err(e) => {
                panic!("{}", e)
            }
        };
        assert_eq!(config.broker_id, 1);
        assert_eq!(config.cluster_name, "mqtt-broker".to_string());
        assert_eq!(config.placement_center.len(), 1);
        assert_eq!(config.grpc_port, 9981);
        assert_eq!(config.http_port, 9982);

        assert_eq!(config.network.tcp_port, 1883);
        assert_eq!(config.network.tcps_port, 8883);
        assert_eq!(config.network.websocket_port, 8093);
        assert_eq!(config.network.websockets_port, 8094);
        assert_eq!(config.network.quic_port, 9083);
        assert!(!config.network.tls_cert.is_empty());
        assert!(!config.network.tls_key.is_empty());

        assert_eq!(config.tcp_thread.accept_thread_num, 1);
        assert_eq!(config.tcp_thread.handler_thread_num, 10);
        assert_eq!(config.tcp_thread.response_thread_num, 1);
        assert_eq!(config.tcp_thread.max_connection_num, 1000);
        assert_eq!(config.tcp_thread.request_queue_size, 2000);
        assert_eq!(config.tcp_thread.response_queue_size, 2000);
        assert_eq!(config.tcp_thread.lock_max_try_mut_times, 30);
        assert_eq!(config.tcp_thread.lock_try_mut_sleep_time_ms, 50);

        assert_eq!(config.system.runtime_worker_threads, 128);
        assert_eq!(config.system.default_user, "admin".to_string());
        assert_eq!(config.system.default_password, "pwd123".to_string());

        assert_eq!(config.storage.storage_type, "memory".to_string());
        assert_eq!(config.storage.journal_addr, "".to_string());
        assert_eq!(config.storage.mysql_addr, "".to_string());

        assert_eq!(
            config.log.log_config,
            "./config/log-config/mqtt-log4rs.yaml"
        );

        assert_eq!(
            config.log.log_path,
            "./robust-data/mqtt-broker/logs".to_string()
        );

        assert_eq!(config.auth.storage_type, "placement".to_string());
        assert_eq!(config.auth.journal_addr, "".to_string());
        assert_eq!(config.auth.mysql_addr, "".to_string());
    }

    #[test]
    fn env_config_default_test() {
        std::env::set_var("MQTT_SERVER_BROKER_ID", "10");
        std::env::set_var("MQTT_SERVER_CLUSTER_NAME", "\"mqtt-broker-env\"");
        std::env::set_var(
            "MQTT_SERVER_PLACEMENT_CENTER",
            "[\"127.0.0.1:1228\",\"127.0.0.1:1228\",\"127.0.0.1:1228\"]",
        );
        std::env::set_var("MQTT_SERVER_GRPC_PORT", "99810");
        std::env::set_var("MQTT_SERVER_HTTP_PORT", "99820");
        std::env::set_var("MQTT_SERVER_NETWORK_TCP_PORT", "18830");
        std::env::set_var("MQTT_SERVER_NETWORK_TCPS_PORT", "88830");
        std::env::set_var("MQTT_SERVER_NETWORK_WEBSOCKET_PORT", "80930");
        std::env::set_var("MQTT_SERVER_NETWORK_WEBSOCKETS_PORT", "80940");
        std::env::set_var("MQTT_SERVER_NETWORK_QUIC_PORT", "90830");
        std::env::set_var("MQTT_SERVER_TLS_CERT", "\"./certs/server.crt-env\"");
        std::env::set_var("MQTT_SERVER_TLS_KEY", "\"./certs/server.key-env\"");
        std::env::set_var("MQTT_SERVER_TCP_THREAD_ACCEPT_THREAD_NUM", "10");
        std::env::set_var("MQTT_SERVER_TCP_THREAD_HANDLER_THREAD_NUM", "100");
        std::env::set_var("MQTT_SERVER_TCP_THREAD_RESPONSE_THREAD_NUM", "10");
        std::env::set_var("MQTT_SERVER_TCP_THREAD_MAX_CONNECTION_NUM", "100000");
        std::env::set_var("MQTT_SERVER_TCP_THREAD_REQUEST_QUEUE_SIZE", "20000");
        std::env::set_var("MQTT_SERVER_TCP_THREAD_RESPONSE_QUEUE_SIZE", "20000");
        std::env::set_var("MQTT_SERVER_TCP_THREAD_LOCK_MAX_TRY_MUT_TIMES", "300");
        std::env::set_var("MQTT_SERVER_TCP_THREAD_LOCK_TRY_MUT_SLEEP_TIME_MS", "500");
        std::env::set_var("MQTT_SERVER_SYSTEM_RUNTIME_WORKER_THREADS", "1280");
        std::env::set_var("MQTT_SERVER_SYSTEM_DEFAULT_USER", "\"admin-env\"");
        std::env::set_var("MQTT_SERVER_SYSTEM_DEFAULT_PASSWORD", "\"pwd123-env\"");
        std::env::set_var("MQTT_SERVER_STORAGE_STORAGE_TYPE", "\"memory-env\"");
        std::env::set_var(
            "MQTT_SERVER_LOG_LOG_CONFIG",
            "\"./config/log-config/mqtt-log4rs.yaml-env\"",
        );
        std::env::set_var(
            "MQTT_SERVER_LOG_LOG_PATH",
            "\"./robust-data/mqtt-broker/logs-env\"",
        );
        std::env::set_var("MQTT_SERVER_AUTH_STORAGE_TYPE", "\"placement-env\"");

        let path = format!(
            "{}/../../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );

        let content = read_file(&path).unwrap();

        let new_content = override_default_by_env(content, "MQTT_SERVER");

        println!("update content:{}", new_content);

        let config: BrokerMqttConfig = match toml::from_str(&new_content) {
            Ok(da) => da,
            Err(e) => {
                panic!("{}", e)
            }
        };
        assert_eq!(config.broker_id, 10);
        assert_eq!(config.cluster_name, "mqtt-broker-env".to_string());
        assert_eq!(config.placement_center.len(), 3);
        assert_eq!(config.grpc_port, 99810);
        assert_eq!(config.http_port, 99820);

        assert_eq!(config.network.tcp_port, 18830);
        assert_eq!(config.network.tcps_port, 88830);
        assert_eq!(config.network.websocket_port, 80930);
        assert_eq!(config.network.websockets_port, 80940);
        assert_eq!(config.network.quic_port, 90830);
        assert!(!config.network.tls_cert.is_empty());
        assert!(!config.network.tls_key.is_empty());

        assert_eq!(config.tcp_thread.accept_thread_num, 10);
        assert_eq!(config.tcp_thread.handler_thread_num, 100);
        assert_eq!(config.tcp_thread.response_thread_num, 10);
        assert_eq!(config.tcp_thread.max_connection_num, 100000);
        assert_eq!(config.tcp_thread.request_queue_size, 20000);
        assert_eq!(config.tcp_thread.response_queue_size, 20000);
        assert_eq!(config.tcp_thread.lock_max_try_mut_times, 300);
        assert_eq!(config.tcp_thread.lock_try_mut_sleep_time_ms, 500);

        assert_eq!(config.system.runtime_worker_threads, 1280);
        assert_eq!(config.system.default_user, "admin-env".to_string());
        assert_eq!(config.system.default_password, "pwd123-env".to_string());

        assert_eq!(config.storage.storage_type, "memory-env".to_string());
        assert_eq!(config.storage.journal_addr, "".to_string());
        assert_eq!(config.storage.mysql_addr, "".to_string());

        assert_eq!(
            config.log.log_config,
            "./config/log-config/mqtt-log4rs.yaml-env"
        );

        assert_eq!(
            config.log.log_path,
            "./robust-data/mqtt-broker/logs-env".to_string()
        );

        assert_eq!(config.auth.storage_type, "placement-env".to_string());
        assert_eq!(config.auth.journal_addr, "".to_string());
        assert_eq!(config.auth.mysql_addr, "".to_string());
    }

    #[test]
    fn config_init_test() {
        let path = format!(
            "{}/../../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );

        init_broker_mqtt_conf_by_path(&path);
        let config = broker_mqtt_conf();
        assert_eq!(config.broker_id, 1);
        assert_eq!(config.cluster_name, "mqtt-broker".to_string());
        assert_eq!(config.placement_center.len(), 1);
        assert_eq!(config.grpc_port, 9981);
        assert_eq!(config.http_port, 9982);

        assert_eq!(config.network.tcp_port, 1883);
        assert_eq!(config.network.tcps_port, 8883);
        assert_eq!(config.network.websocket_port, 8093);
        assert_eq!(config.network.websockets_port, 8094);
        assert_eq!(config.network.quic_port, 9083);
        assert!(!config.network.tls_cert.is_empty());
        assert!(!config.network.tls_key.is_empty());

        assert_eq!(config.tcp_thread.accept_thread_num, 1);
        assert_eq!(config.tcp_thread.handler_thread_num, 10);
        assert_eq!(config.tcp_thread.response_thread_num, 1);
        assert_eq!(config.tcp_thread.max_connection_num, 1000);
        assert_eq!(config.tcp_thread.request_queue_size, 2000);
        assert_eq!(config.tcp_thread.response_queue_size, 2000);
        assert_eq!(config.tcp_thread.lock_max_try_mut_times, 30);
        assert_eq!(config.tcp_thread.lock_try_mut_sleep_time_ms, 50);

        assert_eq!(config.system.runtime_worker_threads, 128);
        assert_eq!(config.system.default_user, "admin".to_string());
        assert_eq!(config.system.default_password, "pwd123".to_string());

        assert_eq!(config.storage.storage_type, "memory".to_string());
        assert_eq!(config.storage.journal_addr, "".to_string());
        assert_eq!(config.storage.mysql_addr, "".to_string());

        assert_eq!(
            config.log.log_path,
            "./robust-data/mqtt-broker/logs".to_string()
        );
        assert_eq!(
            config.log.log_config,
            "./config/log-config/mqtt-log4rs.yaml"
        );

        assert_eq!(config.auth.storage_type, "placement".to_string());
        assert_eq!(config.auth.journal_addr, "".to_string());
        assert_eq!(config.auth.mysql_addr, "".to_string());
    }
}
