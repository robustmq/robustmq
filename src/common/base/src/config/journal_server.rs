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
use crate::tools::{read_file, try_create_fold};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct JournalServerConfig {
    pub cluster_name: String,
    pub node_id: u64,
    pub placement_center: Vec<String>,
    pub network: Network,
    pub system: System,
    pub storage: Storage,
    pub tcp_thread: TcpThread,
    pub prometheus: Prometheus,
    pub log: Log,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Network {
    pub grpc_port: u32,
    pub tcp_port: u32,
    pub tcps_port: u32,
    pub tls_cert: String,
    pub tls_key: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct System {
    pub runtime_work_threads: usize,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Storage {
    pub data_path: Vec<String>,
    pub rocksdb_max_open_files: Option<i32>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TcpThread {
    pub accept_thread_num: usize,
    pub handler_thread_num: usize,
    pub response_thread_num: usize,
    pub max_connection_num: usize,
    pub request_queue_size: usize,
    pub response_queue_size: usize,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Prometheus {
    pub enable: bool,
    pub model: String,
    pub port: u32,
    pub push_gateway_server: String,
    pub interval: u32,
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
        assert_eq!(conf.network.grpc_port, 2228);
    }
}
