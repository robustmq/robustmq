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

use crate::tools::{create_fold, read_file};
use serde::Deserialize;
use std::sync::OnceLock;
use toml::Table;

use super::common::Log;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct PlacementCenterConfig {
    pub cluster_name: String,
    pub node_id: u64,
    pub addr: String,
    pub grpc_port: u16,
    pub http_port: u16,
    pub runtime_work_threads: usize,
    pub data_path: String,
    pub log: Log,
    pub nodes: Table,
    pub rocksdb: Rocksdb,
    pub heartbeat_timeout_ms: u64,
    pub heartbeat_check_time_ms: u64,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct Rocksdb {
    pub max_open_files: Option<i32>,
}

static PLACEMENT_CENTER_CONF: OnceLock<PlacementCenterConfig> = OnceLock::new();

pub fn init_placement_center_conf_by_path(config_path: &String) -> &'static PlacementCenterConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    PLACEMENT_CENTER_CONF.get_or_init(|| {
        let content = match read_file(config_path) {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        let pc_config: PlacementCenterConfig = toml::from_str(&content).unwrap();
        match create_fold(&pc_config.data_path) {
            Ok(()) => {}
            Err(e) => {
                panic!("{}", e);
            }
        }
        match create_fold(&pc_config.log.log_path) {
            Ok(()) => {}
            Err(e) => {
                panic!("{}", e);
            }
        }
        return pc_config;
    })
}

pub fn init_placement_center_conf_by_config(
    config: PlacementCenterConfig,
) -> &'static PlacementCenterConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    match create_fold(&config.data_path) {
        Ok(()) => {}
        Err(e) => {
            panic!("{}", e);
        }
    }
    match create_fold(&config.log.log_path) {
        Ok(()) => {}
        Err(e) => {
            panic!("{}", e);
        }
    }
    PLACEMENT_CENTER_CONF.get_or_init(|| {
        return config;
    })
}

pub fn placement_center_conf() -> &'static PlacementCenterConfig {
    match PLACEMENT_CENTER_CONF.get() {
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
