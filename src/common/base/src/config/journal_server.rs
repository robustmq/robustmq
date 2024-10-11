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
use toml::Table;

use super::common::Log;
use crate::tools::{read_file, try_create_fold};

#[derive(Debug, Deserialize, Clone, Default)]
pub struct JournalServerConfig {
    pub cluster_name: String,
    pub node_id: u64,
    pub grpc_port: u32,
    pub prometheus_port: u16,
    pub runtime_work_threads: usize,
    pub data_path: Vec<String>,
    pub placement_center: Vec<String>,
    pub nodes: Table,
    pub rocksdb: Rocksdb,
    pub network: Network,
    pub log: Log,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Network {
    pub accept_thread_num: usize,
    pub handler_thread_num: usize,
    pub response_thread_num: usize,
    pub max_connection_num: usize,
    pub request_queue_size: usize,
    pub response_queue_size: usize,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Rocksdb {
    pub max_open_files: Option<i32>,
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
        for fold in pc_config.data_path.clone() {
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
    #[ignore]
    fn meta_default() {
        init_journal_server_conf_by_path("../../config/storage-engine.toml");

        let conf = journal_server_conf();
        assert_eq!(conf.grpc_port, 2228);
        //todo meta test case
    }
}
