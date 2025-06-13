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

use crate::{common::override_default_by_env, mqtt::config::BrokerMqttConfig};
use common_base::tools::{read_file, try_create_fold};
use std::sync::OnceLock;

pub mod config;
pub mod default;

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
    use crate::common::find_exist_env_for_config;
    use common_base::tools::read_file;

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

        assert_eq!(config.tcp_thread.lock_max_try_mut_times, 30);
        assert_eq!(config.tcp_thread.lock_try_mut_sleep_time_ms, 50);

        assert_eq!(config.system.runtime_worker_threads, 4);
        assert_eq!(config.system.default_user, "admin".to_string());
        assert_eq!(config.system.default_password, "pwd123".to_string());

        assert!(config.system_monitor.enable);
        assert_eq!(config.system_monitor.os_cpu_check_interval_ms, 60);
        assert_eq!(config.system_monitor.os_cpu_high_watermark, 70.0);
        assert_eq!(config.system_monitor.os_cpu_low_watermark, 50.0);
        assert_eq!(config.system_monitor.os_memory_check_interval_ms, 60);
        assert_eq!(config.system_monitor.os_memory_high_watermark, 80.0);

        assert_eq!(config.storage.storage_type, "memory".to_string());
        assert_eq!(config.storage.journal_addr, "".to_string());
        assert_eq!(config.storage.mysql_addr, "".to_string());

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

                assert!(config.system_monitor.enable);
                assert_eq!(config.system_monitor.os_cpu_check_interval_ms, 60);
                assert_eq!(config.system_monitor.os_cpu_high_watermark, 70.0);
                assert_eq!(config.system_monitor.os_cpu_low_watermark, 50.0);
                assert_eq!(config.system_monitor.os_memory_check_interval_ms, 60);
                assert_eq!(config.system_monitor.os_memory_high_watermark, 80.0);

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

            assert_eq!(config.tcp_thread.lock_max_try_mut_times, 30);
            assert_eq!(config.tcp_thread.lock_try_mut_sleep_time_ms, 50);

            assert_eq!(config.system.runtime_worker_threads, 4);
            assert_eq!(config.system.default_user, "admin".to_string());
            assert_eq!(config.system.default_password, "pwd123".to_string());

            assert!(config.system_monitor.enable);
            assert_eq!(config.system_monitor.os_cpu_check_interval_ms, 60);
            assert_eq!(config.system_monitor.os_cpu_high_watermark, 70.0);
            assert_eq!(config.system_monitor.os_cpu_low_watermark, 50.0);
            assert_eq!(config.system_monitor.os_memory_check_interval_ms, 60);
            assert_eq!(config.system_monitor.os_memory_high_watermark, 80.0);

            assert_eq!(config.storage.storage_type, "memory".to_string());
            assert_eq!(config.storage.journal_addr, "".to_string());
            assert_eq!(config.storage.mysql_addr, "".to_string());

            assert_eq!(config.auth.storage_type, "placement".to_string());
            assert_eq!(config.auth.journal_addr, "".to_string());
            assert_eq!(config.auth.mysql_addr, "".to_string());
        });
    }
}
