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

use super::default_placement_center::{
    default_cluster_name, default_data_path, default_grpc_port, default_heartbeat,
    default_heartbeat_check_time_ms, default_heartbeat_timeout_ms, default_http_port, default_log,
    default_max_open_files, default_network, default_node, default_node_id, default_nodes,
    default_rocksdb, default_runtime_work_threads, default_system,
};
use crate::tools::{try_create_fold, read_file, unique_id};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use toml::{map::Map, Table, Value};

use super::common::Log;

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PlacementCenterConfig {
    #[serde(default = "default_cluster_name")]
    pub cluster_name: String,
    #[serde(default = "default_node")]
    pub node: Node,
    #[serde(default = "default_network")]
    pub network: Network,
    #[serde(default = "default_system")]
    pub system: System,
    #[serde(default = "default_heartbeat")]
    pub heartbeat: Heartbeat,
    #[serde(default = "default_rocksdb")]
    pub rocksdb: Rocksdb,
    #[serde(default = "default_log")]
    pub log: Log,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Node {
    #[serde(default = "default_node_id")]
    pub node_id: u64,
    #[serde(default = "default_nodes")]
    pub nodes: Table,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Network {
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u32,
    #[serde(default = "default_http_port")]
    pub http_port: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct System {
    #[serde(default = "default_runtime_work_threads")]
    pub runtime_work_threads: usize,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Heartbeat {
    #[serde(default = "default_heartbeat_timeout_ms")]
    pub heartbeat_timeout_ms: u64,
    #[serde(default = "default_heartbeat_check_time_ms")]
    pub heartbeat_check_time_ms: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq, Eq)]
pub struct Rocksdb {
    #[serde(default = "default_data_path")]
    pub data_path: String,
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
                panic!("{}", e.to_string());
            }
        };

        let pc_config: PlacementCenterConfig = toml::from_str(&content).unwrap();
        match try_create_fold(&pc_config.rocksdb.data_path) {
            Ok(()) => {}
            Err(e) => {
                panic!("{}", e);
            }
        }
        match try_create_fold(&pc_config.log.log_path) {
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
    match try_create_fold(&config.rocksdb.data_path) {
        Ok(()) => {}
        Err(e) => {
            panic!("{}", e);
        }
    }
    match try_create_fold(&config.log.log_path) {
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

pub fn placement_center_test_conf() -> PlacementCenterConfig {
    let mut config = PlacementCenterConfig::default();
    config.rocksdb.data_path = format!("/tmp/{}", unique_id());
    config.node.node_id = 1;
    let mut nodes = Map::new();
    nodes.insert("1".to_string(), Value::from("127.0.0.1:9982".to_string()));
    config.rocksdb.max_open_files = Some(10);
    config.node.nodes = nodes;
    init_placement_center_conf_by_config(config.clone());
    return config;
}

#[cfg(test)]
mod tests {
    use super::{placement_center_conf, Log, PlacementCenterConfig};
    use crate::config::placement_center::init_placement_center_conf_by_path;
    use toml::Table;

    #[test]
    fn meta_default() {
        let path = format!(
            "{}/src/config/test/placement-center.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        init_placement_center_conf_by_path(&path);
        let config: &PlacementCenterConfig = placement_center_conf();
        println!("{:?}", config);
        assert_eq!(config.cluster_name, "placement-test");
        assert_eq!(config.node.node_id, 1);
        assert_eq!(config.network.grpc_port, 1228);
        assert_eq!(config.network.http_port, 1227);
        assert_eq!(config.system.runtime_work_threads, 100);
        println!("{}", config.rocksdb.data_path);
        println!("{}", "/tmp/robust/placement-center/data".to_string());
        assert_eq!(
            config.rocksdb.data_path,
            "/tmp/robust/placement-center/data".to_string()
        );
        assert_eq!(
            config.log,
            Log {
                log_path: format!("./logs/placement-center"),
                log_config: format!("./config/log4rs.yaml"),
            }
        );
        let mut nodes = Table::new();
        nodes.insert(
            "1".to_string(),
            toml::Value::String(format!("{}:{}", "127.0.0.1", "1228")),
        );
        assert_eq!(config.rocksdb.max_open_files, Some(10000 as i32));
        assert_eq!(config.heartbeat.heartbeat_timeout_ms, 30000);
        assert_eq!(config.heartbeat.heartbeat_check_time_ms, 1000);
    }
}
