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

#[derive(Debug, Deserialize, Clone)]
pub struct MetaConfig {
    pub node_id: u64,
    pub addr: String,
    pub port: u16,
    pub admin_port: u16,
    pub runtime_work_threads: usize,
    pub data_path: String,
    pub log_path: String,
    pub log_segment_size: u64,
    pub log_file_num: u32,
    pub meta_nodes: Vec<String>,
    pub rocksdb: Rocksdb,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rocksdb {
    pub max_open_files: Option<i32>,
}

impl Default for MetaConfig {
    fn default() -> Self {
        MetaConfig {
            node_id: 1,
            addr: "127.0.0.1".to_string(),
            port: 1227,
            admin_port: 1226,
            runtime_work_threads: 10,
            log_segment_size: 1024 * 1024 * 1024 * 1024 * 1024,
            log_file_num: 50,
            data_path: "/tmp/data".to_string(),
            log_path: "/tmp/logs".to_string(),
            meta_nodes: vec![],
            rocksdb: Rocksdb {
                max_open_files: Some(100),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MetaConfig;

    #[test]
    fn meta_default() {
        MetaConfig::default();
        //todo meta test case
    }
}
