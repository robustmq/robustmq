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
use serde::Deserialize;
use std::sync::OnceLock;
use toml::Table;

#[derive(Debug, Deserialize, Clone)]
pub struct StorageEngineConfig {
    pub cluster_name: String,
    pub node_id: u64,
    pub grpc_port: u32,
    pub prometheus_port: u16,
    pub runtime_work_threads: usize,
    pub data_path: Vec<String>,
    pub log_path: String,
    pub log_segment_size: u64,
    pub log_file_num: u32,
    pub placement_center: Vec<String>,
    pub nodes: Table,
    pub rocksdb: Rocksdb,
    pub network: Network,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Network {
    pub accept_thread_num: usize,
    pub handler_thread_num: usize,
    pub response_thread_num: usize,
    pub max_connection_num: usize,
    pub request_queue_size: usize,
    pub response_queue_size: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rocksdb {
    pub max_open_files: Option<i32>,
}

impl Default for StorageEngineConfig {
    fn default() -> Self {
        StorageEngineConfig {
            cluster_name: "default".to_string(),
            node_id: 1,
            grpc_port: 1227,
            placement_center: Vec::new(),
            prometheus_port: 1226,
            runtime_work_threads: 10,
            log_segment_size: 1024 * 1024 * 1024 * 1024 * 1024,
            log_file_num: 50,
            data_path: Vec::new(),
            log_path: "/tmp/logs".to_string(),
            nodes: Table::new(),
            rocksdb: Rocksdb {
                max_open_files: Some(100),
            },
            network: Network {
                accept_thread_num: 1,
                handler_thread_num: 1,
                response_thread_num: 1,
                max_connection_num: 1,
                request_queue_size: 100,
                response_queue_size: 100,
            },
        }
    }
}

static STORAGE_ENGINE_CONFIG: OnceLock<StorageEngineConfig> = OnceLock::new();

pub fn init_storage_engine_conf_by_path(config_path: &String) -> &'static StorageEngineConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    STORAGE_ENGINE_CONFIG.get_or_init(|| {
        let content = read_file(config_path);
        let pc_config: StorageEngineConfig = toml::from_str(&content).unwrap();
        for fold in pc_config.data_path.clone() {
            create_fold(fold);
        }
        create_fold(pc_config.log_path.clone());
        return pc_config;
    })
}

pub fn init_storage_engine_conf_by_config(
    config: StorageEngineConfig,
) -> &'static StorageEngineConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    STORAGE_ENGINE_CONFIG.get_or_init(|| {
        return config;
    })
}

pub fn storage_engine_conf() -> &'static StorageEngineConfig {
    match STORAGE_ENGINE_CONFIG.get() {
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

#[cfg(test)]
mod tests {
    use crate::config::storage_engine::storage_engine_conf;

    use super::init_storage_engine_conf_by_path;
    #[test]
    fn meta_default() {
        init_storage_engine_conf_by_path(&"../../config/storage-engine.toml".to_string());

        let conf = storage_engine_conf();
        assert_eq!(conf.grpc_port, 2228);
        //todo meta test case
    }
}
