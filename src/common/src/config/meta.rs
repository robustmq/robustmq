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
    pub node_id: i32,
    pub addr: String,
    pub port: Option<u16>,
    pub runtime_work_threads: usize,
    pub data_path: String,
    pub meta_nodes: Vec<String>,
    pub rocksdb: Rocksdb,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rocksdb {
    pub max_open_files: Option<i32>,
}

// Default basic configuration of meta cluster
const DEFAULT_META_ADDRESS: &str = "127.0.0.1";
const DEFAULT_META_PORT: Option<u16> = Some(1227);
const DEFAULT_DATA_PATH: &str = "./data";

// Default Settings for storage tier rocksdb
const DEFAULT_ROCKSDB_MAX_OPEN_FILES: Option<i32> = Some(10000);

impl Default for MetaConfig {
    fn default() -> Self {
        MetaConfig {
            addr: String::from(DEFAULT_META_ADDRESS),
            data_path: String::from(DEFAULT_DATA_PATH),
            port: DEFAULT_META_PORT,
            ..Default::default()
        }
    }
}

impl Default for Rocksdb {
    fn default() -> Self {
        Rocksdb {
            max_open_files: DEFAULT_ROCKSDB_MAX_OPEN_FILES,
        }
    }
}
