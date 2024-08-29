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

 use super::default_placement_center::{
    default_cluster_name, default_node_id, default_addr, default_grpc_port, 
    default_http_port, default_runtime_work_threads, default_data_path, 
    default_log, default_nodes, default_rocksdb, default_max_open_files, 
    default_heartbeat_timeout_ms, default_heartbeat_check_time_ms,
};
use crate::tools::{create_fold, read_file};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use toml::Table;

use super::common::Log;

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct PlacementCenterConfig {
    #[serde(default = "default_cluster_name")]
    pub cluster_name: String,
    #[serde(default = "default_node_id")]
    pub node_id: u64,
    #[serde(default = "default_addr")]
    pub addr: String,
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u32,
    #[serde(default = "default_http_port")]
    pub http_port: u32,
    #[serde(default = "default_runtime_work_threads")]
    pub runtime_work_threads: usize,
    #[serde(default = "default_data_path")]
    pub data_path: String,
    #[serde(default = "default_log")]
    pub log: Log,
    #[serde(default = "default_nodes")]
    pub nodes: Table,
    #[serde(default = "default_rocksdb")]
    pub rocksdb: Rocksdb,
    #[serde(default = "default_heartbeat_timeout_ms")]
    pub heartbeat_timeout_ms: u64,
    #[serde(default = "default_heartbeat_check_time_ms")]
    pub heartbeat_check_time_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Rocksdb {
    #[serde(default = "default_max_open_files")]
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
                eprintln!("{}", e.to_string());
                "".to_string()
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
    use crate::config::{placement_center::init_placement_center_conf_by_path};
    use super::{placement_center_conf, PlacementCenterConfig, Log, Rocksdb};
    use toml::Table;

    #[test]
    fn meta_default() {
        let path = format!(
            "{}/src/config/test/placement-center.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_placement_center_conf_by_path(&path);
        let config: &PlacementCenterConfig = placement_center_conf();
        assert_eq!(config.cluster_name, "placement-test");
        assert_eq!(config.node_id, 1);
        assert_eq!(config.addr, "127.0.0.1");
        assert_eq!(config.grpc_port, 1228);
        assert_eq!(config.http_port, 1227);
        assert_eq!(config.runtime_work_threads, 100);
        assert_eq!(config.data_path, "/tmp/robust/placement-center/data");
        assert_eq!(config.log, Log {
            log_path: format!("./logs/placement-center"),
            log_config: format!("./config/log4rs.yaml"),
        });
        let mut nodes = Table::new();
        nodes.insert(
            "1".to_string(),
            toml::Value::String(format!("{}:{}", "127.0.0.1", "1228"))
        );
        assert_eq!(config.nodes, nodes);
        assert_eq!(config.rocksdb, Rocksdb {
            max_open_files: Some(10000 as i32)
        });
        assert_eq!(config.heartbeat_timeout_ms, 30000);
        assert_eq!(config.heartbeat_check_time_ms, 1000);
    }
}
