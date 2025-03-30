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

use std::env::temp_dir;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use toml::map::Map;
use toml::{Table, Value};

use super::common::{override_default_by_env, Log};
use super::default_placement_center::{
    default_cluster_name, default_data_path, default_grpc_port, default_heartbeat,
    default_heartbeat_check_time_ms, default_heartbeat_timeout_ms, default_http_port,
    default_local_ip, default_log, default_max_open_files, default_network, default_node,
    default_node_id, default_nodes, default_rocksdb, default_runtime_work_threads, default_system,
};
use crate::tools::{read_file, try_create_fold};

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
    #[serde(default = "default_local_ip")]
    pub local_ip: String,
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

pub fn init_placement_center_conf_by_path(config_path: &str) -> &'static PlacementCenterConfig {
    // n.b. static items do not call [`Drop`] on program termination, so if
    // [`DeepThought`] impls Drop, that will not be used for this instance.
    PLACEMENT_CENTER_CONF.get_or_init(|| {
        let content = match read_file(config_path) {
            Ok(data) => data,
            Err(e) => {
                panic!("{}", e.to_string());
            }
        };
        let new_content = override_default_by_env(content, "PLACEMENT_CENTER");
        let pc_config: PlacementCenterConfig = toml::from_str(&new_content).unwrap();
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
        pc_config
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
    PLACEMENT_CENTER_CONF.get_or_init(|| config)
}

pub fn placement_center_conf() -> &'static PlacementCenterConfig {
    match PLACEMENT_CENTER_CONF.get() {
        Some(config) => config,
        None => {
            panic!(
                "Placement center configuration is not initialized, check the configuration file."
            );
        }
    }
}

pub fn placement_center_test_conf() -> PlacementCenterConfig {
    let rocksdb = Rocksdb {
        data_path: temp_dir().to_str().unwrap().to_string(),
        max_open_files: Some(10),
    };

    let mut nodes = Map::new();
    nodes.insert("1".to_string(), Value::from("127.0.0.1:9982".to_string()));
    let node = Node { node_id: 1, nodes };

    let config = PlacementCenterConfig {
        rocksdb,
        node,
        ..Default::default()
    };
    init_placement_center_conf_by_config(config.clone());
    config
}

#[cfg(test)]
mod tests {
    use toml::Table;

    use super::{placement_center_conf, Log, PlacementCenterConfig};
    use crate::config::placement_center::init_placement_center_conf_by_path;

    #[test]
    fn meta_default() {
        let path = format!(
            "{}/../../../config/placement-center.toml",
            env!("CARGO_MANIFEST_DIR")
        );
        std::env::set_var("PLACEMENT_CENTER_NODE_NODE_ID", "2");
        std::env::set_var(
            "PLACEMENT_CENTER_NODE_NODES",
            "{ 1 = \"127.0.0.1:1228\" , 2 = \"127.0.0.1:1227\" }",
        );
        init_placement_center_conf_by_path(&path);
        let config: &PlacementCenterConfig = placement_center_conf();
        println!("{:?}", config);
        assert_eq!(config.cluster_name, "placement-test");
        assert_eq!(config.node.node_id, 2);
        assert_eq!(config.node.nodes.len(), 2);
        assert_eq!(config.network.grpc_port, 1228);
        assert_eq!(config.network.http_port, 1227);
        assert_eq!(config.system.runtime_work_threads, 100);
        println!("{}", config.rocksdb.data_path);
        println!("./robust-data/placement-center/data");
        assert_eq!(
            config.rocksdb.data_path,
            "./robust-data/placement-center/data".to_string()
        );
        assert_eq!(
            config.log,
            Log {
                log_path: "./robust-data/placement-center/logs".to_string(),
                log_config: "./config/log-config/place-log4rs.yaml".to_string(),
            }
        );
        let mut nodes = Table::new();
        nodes.insert(
            "1".to_string(),
            toml::Value::String(format!("{}:{}", "127.0.0.1", "1228")),
        );
        assert_eq!(config.rocksdb.max_open_files, Some(10000_i32));
        assert_eq!(config.heartbeat.heartbeat_timeout_ms, 5000);
        assert_eq!(config.heartbeat.heartbeat_check_time_ms, 1000);
    }
}
