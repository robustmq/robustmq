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

use serde::Deserialize;

use super::common::Log;
use super::default_journal_server::{
    default_grpc_port, default_network, default_network_tcp_port,default_network_tcps_port, default_system, default_storage, default_tcp_thread, default_log, default_prometheus, default_prometheus_port
};
use crate::tools::{read_file, try_create_fold};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct JournalServerConfig {
    pub cluster_name: String,
    pub node_id: u64,
    #[serde(default)]
    pub placement_center: Vec<String>,
    #[serde(default = "default_network")]
    pub network: Network,
    #[serde(default = "default_system")]
    pub system: System,
    #[serde(default = "default_storage")]
    pub storage: Storage,
    #[serde(default = "default_tcp_thread")]
    pub tcp_thread: TcpThread,
    #[serde(default = "default_prometheus")]
    pub prometheus: Prometheus,
    #[serde(default = "default_log")]
    pub log: Log,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Network {
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u32,
    #[serde(default = "default_network_tcp_port")]
    pub tcp_port: u32,
    #[serde(default = "default_network_tcps_port")]
    pub tcps_port: u32,
    #[serde(default)]
    pub tls_cert: String,
    #[serde(default)]
    pub tls_key: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct System {
    #[serde(default)]
    pub runtime_work_threads: usize,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Storage {
    #[serde(default)]
    pub data_path: Vec<String>,
    pub rocksdb_max_open_files: Option<i32>,
}

#[derive(Debug, Deserialize, Clone, Default)]
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
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Prometheus {
    #[serde(default)]
    pub enable: bool,
    #[serde(default)]
    pub model: String,
    #[serde(default = "default_prometheus_port")]
    pub port: u32,
    #[serde(default)]
    pub push_gateway_server: String,
    #[serde(default)]
    pub interval: u32,
    #[serde(default)]
    pub header: String,
}

static STORAGE_ENGINE_CONFIG: OnceLock<JournalServerConfig> = OnceLock::new();

pub fn init_journal_server_conf_by_path(config_path: &str) -> &'static JournalServerConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    STORAGE_ENGINE_CONFIG.get_or_init(|| {
        let content = match read_file(config_path) {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        let pc_config: JournalServerConfig = toml::from_str(&content).unwrap();
        for fold in pc_config.storage.data_path.clone() {
            match try_create_fold(&fold) {
                Ok(()) => {}
                Err(e) => {
                    panic!("{}", e);
                }
            }
        }
        match try_create_fold(&pc_config.log.log_path) {
            Ok(()) => {}
            Err(e) => {
                panic!("{}", e);
            }
        }
        pc_config
    })
}

pub fn init_journal_server_conf_by_config(
    config: JournalServerConfig,
) -> &'static JournalServerConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    STORAGE_ENGINE_CONFIG.get_or_init(|| config)
}

pub fn journal_server_conf() -> &'static JournalServerConfig {
    match STORAGE_ENGINE_CONFIG.get() {
        Some(config) => config,
        None => {
            panic!(
                "Placement center configuration is not initialized, check the configuration file."
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::init_journal_server_conf_by_path;
    use crate::config::journal_server::journal_server_conf;
    #[test]
    fn journal_server_toml_test() {
        let path = format!(
            "{}/../../../config/journal-server.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_journal_server_conf_by_path(&path);

        let conf = journal_server_conf();
        assert_eq!(conf.cluster_name, "JournalCluster1".to_string());
        assert_eq!(conf.node_id, 1);
        assert_eq!(conf.placement_center, vec![String::from("127.0.0.1:1228")]);
        assert_eq!(conf.network.grpc_port, 2228);
        assert_eq!(conf.network.tcp_port, 3110);
        assert_eq!(conf.network.tcps_port, 3111);

        assert_eq!(conf.system.runtime_work_threads, 100);

        assert_eq!(conf.tcp_thread.accept_thread_num, 1);
        assert_eq!(conf.tcp_thread.handler_thread_num, 20);
        assert_eq!(conf.tcp_thread.response_thread_num, 2);
        assert_eq!(conf.tcp_thread.max_connection_num, 1000);
        assert_eq!(conf.tcp_thread.request_queue_size, 2000);
        assert_eq!(conf.tcp_thread.response_queue_size, 2000);

        assert_eq!(conf.prometheus.enable, false);
        assert_eq!(conf.prometheus.model, "pull".to_string());
        assert_eq!(conf.prometheus.port, 3306);
        assert_eq!(conf.prometheus.interval, 10);
    }
}
