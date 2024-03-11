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

use crate::{config::read_file, tools::create_fold};

#[derive(Debug, Deserialize, Clone)]
pub struct PlacementCenterConfig {
    pub node_id: u64,
    pub addr: String,
    pub grpc_port: u16,
    pub http_port: u16,
    pub runtime_work_threads: usize,
    pub data_path: String,
    pub log_path: String,
    pub log_segment_size: u64,
    pub log_file_num: u32,
    pub nodes: Table,
    pub rocksdb: Rocksdb,
    pub heartbeat_timeout_ms: u64,
    pub heartbeat_check_time_ms: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Rocksdb {
    pub max_open_files: Option<i32>,
}

impl Default for PlacementCenterConfig {
    fn default() -> Self {
        PlacementCenterConfig {
            node_id: 1,
            addr: "127.0.0.1".to_string(),
            grpc_port: 1227,
            http_port: 1226,
            runtime_work_threads: 10,
            log_segment_size: 1024 * 1024 * 1024 * 1024 * 1024,
            log_file_num: 50,
            data_path: "/tmp/data".to_string(),
            log_path: "/tmp/logs".to_string(),
            nodes: Table::new(),
            heartbeat_timeout_ms: 30000,
            heartbeat_check_time_ms: 1000,
            rocksdb: Rocksdb {
                max_open_files: Some(100),
            },
        }
    }
}

static COMPUTATION: OnceLock<PlacementCenterConfig> = OnceLock::new();

pub fn init_placement_center_conf_by_path(config_path: &String) -> &'static PlacementCenterConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    COMPUTATION.get_or_init(|| {
        let content = read_file(config_path);
        let pc_config: PlacementCenterConfig = toml::from_str(&content).unwrap();
        create_fold(pc_config.data_path.clone());
        create_fold(pc_config.log_path.clone());
        return pc_config;
    })
}

pub fn init_placement_center_conf_by_config(
    config: PlacementCenterConfig,
) -> &'static PlacementCenterConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    COMPUTATION.get_or_init(|| {
        return config;
    })
}

pub fn placement_center_conf() -> &'static PlacementCenterConfig {
    match COMPUTATION.get() {
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
    use crate::config::placement_center::init_placement_center_conf_by_path;

    use super::{placement_center_conf, PlacementCenterConfig};

    #[test]
    fn meta_default() {
        let path =
            &"/Users/bytedance/Desktop/code/robustmq-project/robustmq/config/placement-center.toml"
                .to_string();
        init_placement_center_conf_by_path(path);
        let conf: &PlacementCenterConfig = placement_center_conf();
        assert_eq!(conf.grpc_port, 1228);
    }
}
