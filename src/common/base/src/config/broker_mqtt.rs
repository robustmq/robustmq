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

use super::read_file;
use crate::tools::create_fold;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BrokerMQTTConfig {
    pub cluster_name: String,
    pub broker_id: u64,
    pub grpc_port: u32,
    pub http_port: usize,
    pub placement_center: Vec<String>,
    pub network: Network,
    pub tcp_thread: TcpThread,
    pub system: System,
    pub storage: Storage,
    pub log: Log,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Network {
    pub tcp_port: u32,
    pub tcps_port: u32,
    pub websocket_port: u32,
    pub websockets_port: u32,
    pub quic_port: u32,
    pub tls_cert: String,
    pub tls_key: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TcpThread {
    pub accept_thread_num: usize,
    pub handler_thread_num: usize,
    pub response_thread_num: usize,
    pub max_connection_num: usize,
    pub request_queue_size: usize,
    pub response_queue_size: usize,
    pub lock_max_try_mut_times: u64,
    pub lock_try_mut_sleep_time_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Storage {
    pub storage_type: String,
    pub journal_addr: String,
    pub mysql_addr: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct System {
    pub runtime_worker_threads: usize,
    pub default_user: String,
    pub default_password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Log {
    pub log_path: String,
    pub log_segment_size: u64,
    pub log_file_num: u32,
}

static BROKER_MQTT_CONF: OnceLock<BrokerMQTTConfig> = OnceLock::new();

pub fn init_broker_mqtt_conf_by_path(config_path: &String) -> &'static BrokerMQTTConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    BROKER_MQTT_CONF.get_or_init(|| {
        let content = read_file(config_path);
        let config: BrokerMQTTConfig = match toml::from_str(&content) {
            Ok(da) => da,
            Err(e) => {
                panic!("{}", e)
            }
        };
        create_fold(config.log.log_path.clone());
        return config;
    })
}

pub fn init_broker_mqtt_conf_by_config(config: BrokerMQTTConfig) -> &'static BrokerMQTTConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    BROKER_MQTT_CONF.get_or_init(|| {
        return config;
    })
}

pub fn broker_mqtt_conf() -> &'static BrokerMQTTConfig {
    match BROKER_MQTT_CONF.get() {
        Some(config) => {
            return config;
        }
        None => {
            panic!(
                "Placement center configuration is not initialized, check the configuration file."
            );
        }
    }
}
