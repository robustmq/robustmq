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

use serde::Deserialize;
use toml::Table;

#[derive(Debug, Deserialize, Clone)]
pub struct StorageEngineConfig {
    pub cluster_name: String,
    pub node_id: u64,
    pub addr: String,
    pub grpc_port: u32,
    pub prometheus_port: u16,
    pub runtime_work_threads: usize,
    pub data_path: String,
    pub log_path: String,
    pub log_segment_size: u64,
    pub log_file_num: u32,
    pub placement_center: Vec<String>,
    pub nodes: Table,
    pub rocksdb: Rocksdb,
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
            addr: "127.0.0.1".to_string(),
            grpc_port: 1227,
            placement_center: Vec::new(),
            prometheus_port: 1226,
            runtime_work_threads: 10,
            log_segment_size: 1024 * 1024 * 1024 * 1024 * 1024,
            log_file_num: 50,
            data_path: "/tmp/data".to_string(),
            log_path: "/tmp/logs".to_string(),
            nodes: Table::new(),
            rocksdb: Rocksdb {
                max_open_files: Some(100),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::parse_storage_engine;

    use super::StorageEngineConfig;

    #[test]
    fn meta_default() {
        let conf: StorageEngineConfig =
            parse_storage_engine(&"../../config/raft/node-1.toml".to_string());
        StorageEngineConfig::default();
        //todo meta test case
    }
}
