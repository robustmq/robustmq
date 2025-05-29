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

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::common::{
    default_pprof, default_prometheus, override_default_by_env, Auth, Log, Pprof, Prometheus,
    Storage, Telemetry,
};
use super::default_mqtt::{
    default_auth, default_grpc_port, default_heartbeat_timeout, default_log,
    default_mqtt_cluster_dynamic_feature, default_mqtt_cluster_dynamic_flapping_detect,
    default_mqtt_cluster_dynamic_network, default_mqtt_cluster_dynamic_protocol,
    default_mqtt_cluster_dynamic_security, default_mqtt_cluster_dynamic_slow_sub, default_network,
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
    #[serde(default = "default_placement_center")]
    pub placement_center: Vec<String>,
    #[serde(default = "default_heartbeat_timeout")]
    pub heartbeat_timeout: String,
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
    #[serde(default = "default_prometheus")]
    pub prometheus: Prometheus,
    #[serde(default = "default_pprof")]
    pub pprof: Pprof,

    #[serde(default = "default_mqtt_cluster_dynamic_slow_sub")]
    pub cluster_dynamic_config_slow_sub: MqttClusterDynamicSlowSub,
    #[serde(default = "default_mqtt_cluster_dynamic_flapping_detect")]
    pub cluster_dynamic_config_flapping_detect: MqttClusterDynamicFlappingDetect,
    #[serde(default = "default_mqtt_cluster_dynamic_protocol")]
    pub cluster_dynamic_config_protocol: MqttClusterDynamicConfigProtocol,
    #[serde(default = "default_mqtt_cluster_dynamic_feature")]
    pub cluster_dynamic_config_feature: MqttClusterDynamicConfigFeature,
    #[serde(default = "default_mqtt_cluster_dynamic_security")]
    pub cluster_dynamic_config_security: MqttClusterDynamicConfigSecurity,
    #[serde(default = "default_mqtt_cluster_dynamic_network")]
    pub cluster_dynamic_config_network: MqttClusterDynamicConfigNetwork,
}

// MQTT cluster protocol related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigProtocol {
    pub session_expiry_interval: u32,
    pub topic_alias_max: u16,
    pub max_qos: u8,
    pub max_packet_size: u32,
    pub max_server_keep_alive: u16,
    pub default_server_keep_alive: u16,
    pub receive_max: u16,
    pub max_message_expiry_interval: u64,
    pub client_pkid_persistent: bool,
}

impl MqttClusterDynamicConfigProtocol {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

// MQTT cluster security related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigSecurity {
    pub is_self_protection_status: bool,
    pub secret_free_login: bool,
}

impl MqttClusterDynamicConfigSecurity {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

// MQTT cluster network related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigNetwork {
    pub tcp_max_connection_num: u64,
    pub tcps_max_connection_num: u64,
    pub websocket_max_connection_num: u64,
    pub websockets_max_connection_num: u64,
    pub response_max_try_mut_times: u64,
    pub response_try_mut_sleep_time_ms: u64,
}

// MQTT cluster Feature related dynamic configuration
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicConfigFeature {
    pub retain_available: ConfigAvailableFlag,
    pub wildcard_subscription_available: ConfigAvailableFlag,
    pub subscription_identifiers_available: ConfigAvailableFlag,
    pub shared_subscription_available: ConfigAvailableFlag,
    pub exclusive_subscription_available: ConfigAvailableFlag,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicSlowSub {
    pub enable: bool,
    pub whole_ms: u64,
    pub internal_ms: u32,
    pub response_ms: u32,
}
impl MqttClusterDynamicSlowSub {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicFlappingDetect {
    pub enable: bool,
    pub window_time: u32,
    pub max_client_connections: u64,
    pub ban_time: u32,
}

impl MqttClusterDynamicFlappingDetect {
    pub fn encode(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct MqttClusterDynamicOfflineMessage {
    pub enable: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
pub enum ConfigAvailableFlag {
    #[default]
    Disable,
    Enable,
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
    use crate::{config::common::find_exist_env_for_config, tools::read_file};

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
        assert_eq!(config.heartbeat_timeout, "10s".to_string());

        assert_eq!(config.network.tcp_port, 1883);
        assert_eq!(config.network.tcps_port, 8883);
        assert_eq!(config.network.websocket_port, 8093);
        assert_eq!(config.network.websockets_port, 8094);
        assert_eq!(config.network.quic_port, 9083);
        assert!(!config.network.tls_cert.is_empty());
        assert!(!config.network.tls_key.is_empty());

        assert_eq!(config.tcp_thread.accept_thread_num, 5);
        assert_eq!(config.tcp_thread.handler_thread_num, 32);
        assert_eq!(config.tcp_thread.response_thread_num, 5);
        assert_eq!(config.tcp_thread.max_connection_num, 1000);
        assert_eq!(config.tcp_thread.request_queue_size, 2000);
        assert_eq!(config.tcp_thread.response_queue_size, 2000);
        assert_eq!(config.tcp_thread.lock_max_try_mut_times, 30);
        assert_eq!(config.tcp_thread.lock_try_mut_sleep_time_ms, 50);

        assert_eq!(config.system.runtime_worker_threads, 4);
        assert_eq!(config.system.default_user, "admin".to_string());
        assert_eq!(config.system.default_password, "pwd123".to_string());

        assert_eq!(config.storage.storage_type, "memory".to_string());
        assert_eq!(config.storage.journal_addr, "".to_string());
        assert_eq!(config.storage.mysql_addr, "".to_string());

        assert_eq!(
            config.log.log_config,
            "./config/log-config/mqtt-tracing.toml"
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
        temp_env::with_vars(
            [
                ("MQTT_SERVER_BROKER_ID", Some("10")),
                ("MQTT_SERVER_CLUSTER_NAME", Some("\"mqtt-broker-env\"")),
                (
                    "MQTT_SERVER_PLACEMENT_CENTER",
                    Some("[\"127.0.0.1:1228\",\"127.0.0.1:1228\",\"127.0.0.1:1228\"]"),
                ),
                ("MQTT_SERVER_GRPC_PORT", Some("99810")),
                ("MQTT_SERVER_NETWORK_TCP_PORT", Some("18830")),
                ("MQTT_SERVER_NETWORK_TCPS_PORT", Some("88830")),
                ("MQTT_SERVER_NETWORK_WEBSOCKET_PORT", Some("80930")),
                ("MQTT_SERVER_NETWORK_WEBSOCKETS_PORT", Some("80940")),
                ("MQTT_SERVER_NETWORK_QUIC_PORT", Some("90830")),
                ("MQTT_SERVER_TLS_CERT", Some("\"./certs/server.crt-env\"")),
                ("MQTT_SERVER_TLS_KEY", Some("\"./certs/server.key-env\"")),
                ("MQTT_SERVER_TCP_THREAD_ACCEPT_THREAD_NUM", Some("10")),
                ("MQTT_SERVER_TCP_THREAD_HANDLER_THREAD_NUM", Some("100")),
                ("MQTT_SERVER_TCP_THREAD_RESPONSE_THREAD_NUM", Some("10")),
                ("MQTT_SERVER_TCP_THREAD_MAX_CONNECTION_NUM", Some("100000")),
                ("MQTT_SERVER_TCP_THREAD_REQUEST_QUEUE_SIZE", Some("20000")),
                ("MQTT_SERVER_TCP_THREAD_RESPONSE_QUEUE_SIZE", Some("20000")),
                ("MQTT_SERVER_TCP_THREAD_LOCK_MAX_TRY_MUT_TIMES", Some("300")),
                (
                    "MQTT_SERVER_TCP_THREAD_LOCK_TRY_MUT_SLEEP_TIME_MS",
                    Some("500"),
                ),
                ("MQTT_SERVER_SYSTEM_RUNTIME_WORKER_THREADS", Some("1280")),
                ("MQTT_SERVER_SYSTEM_DEFAULT_USER", Some("\"admin-env\"")),
                (
                    "MQTT_SERVER_SYSTEM_DEFAULT_PASSWORD",
                    Some("\"pwd123-env\""),
                ),
                ("MQTT_SERVER_STORAGE_STORAGE_TYPE", Some("\"memory-env\"")),
                (
                    "MQTT_SERVER_LOG_LOG_CONFIG",
                    Some("\"./config/log-config/mqtt-tracing.toml-env\""),
                ),
                (
                    "MQTT_SERVER_LOG_LOG_PATH",
                    Some("\"./robust-data/mqtt-broker/logs-env\""),
                ),
                ("MQTT_SERVER_AUTH_STORAGE_TYPE", Some("\"placement-env\"")),
            ],
            || {
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
                    "./config/log-config/mqtt-tracing.toml-env"
                );

                assert_eq!(
                    config.log.log_path,
                    "./robust-data/mqtt-broker/logs-env".to_string()
                );

                assert_eq!(config.auth.storage_type, "placement-env".to_string());
                assert_eq!(config.auth.journal_addr, "".to_string());
                assert_eq!(config.auth.mysql_addr, "".to_string());
            },
        );
    }

    #[test]
    fn config_init_test() {
        let path = format!(
            "{}/../../../config/mqtt-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );

        // Test methods are executed concurrently.
        // BROKER_MQTT_CONF captures environment variables when the initialization program is started,
        // so we need to avoid the impact between test methods and execute the following tests in a "clean" env
        let content = read_file(&path).unwrap();
        let env_map = find_exist_env_for_config(&content, "MQTT_SERVER");
        let env_k: Vec<&String> = env_map.keys().collect();

        temp_env::with_vars_unset(env_k, || {
            init_broker_mqtt_conf_by_path(&path);
            let config = broker_mqtt_conf();
            assert_eq!(config.broker_id, 1);
            assert_eq!(config.cluster_name, "mqtt-broker".to_string());
            assert_eq!(config.placement_center.len(), 1);
            assert_eq!(config.grpc_port, 9981);
            assert_eq!(config.heartbeat_timeout, "10s".to_string());

            assert_eq!(config.network.tcp_port, 1883);
            assert_eq!(config.network.tcps_port, 8883);
            assert_eq!(config.network.websocket_port, 8093);
            assert_eq!(config.network.websockets_port, 8094);
            assert_eq!(config.network.quic_port, 9083);
            assert!(!config.network.tls_cert.is_empty());
            assert!(!config.network.tls_key.is_empty());

            assert_eq!(config.tcp_thread.accept_thread_num, 5);
            assert_eq!(config.tcp_thread.handler_thread_num, 32);
            assert_eq!(config.tcp_thread.response_thread_num, 5);
            assert_eq!(config.tcp_thread.max_connection_num, 1000);
            assert_eq!(config.tcp_thread.request_queue_size, 2000);
            assert_eq!(config.tcp_thread.response_queue_size, 2000);
            assert_eq!(config.tcp_thread.lock_max_try_mut_times, 30);
            assert_eq!(config.tcp_thread.lock_try_mut_sleep_time_ms, 50);

            assert_eq!(config.system.runtime_worker_threads, 4);
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
                "./config/log-config/mqtt-tracing.toml"
            );

            assert_eq!(config.auth.storage_type, "placement".to_string());
            assert_eq!(config.auth.journal_addr, "".to_string());
            assert_eq!(config.auth.mysql_addr, "".to_string());
        });
    }
}
