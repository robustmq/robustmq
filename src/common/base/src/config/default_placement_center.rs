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

use toml::Table;

use super::common::Log;
use super::placement_center::{Heartbeat, Network, Node, Rocksdb, System};

pub fn default_cluster_name() -> String {
    "placement-center".to_string()
}

pub fn default_node() -> Node {
    Node {
        node_id: default_node_id(),
        nodes: default_nodes(),
    }
}

pub fn default_node_id() -> u64 {
    1
}

pub fn default_network() -> Network {
    Network {
        grpc_port: default_grpc_port(),
        http_port: default_http_port(),
    }
}

pub fn default_grpc_port() -> u32 {
    1228
}

pub fn default_http_port() -> u32 {
    1227
}

pub fn default_system() -> System {
    System {
        runtime_work_threads: default_runtime_work_threads(),
    }
}

pub fn default_runtime_work_threads() -> usize {
    100
}

pub fn default_data_path() -> String {
    "/tmp/robust/placement-center/data".to_string()
}

pub fn default_log() -> Log {
    Log {
        log_path: "./logs/placement-center".to_string(),
        log_config: "./config/log4rs.yaml".to_string(),
    }
}

pub fn default_nodes() -> Table {
    let mut nodes = Table::new();
    nodes.insert(
        default_node_id().to_string(),
        toml::Value::String(format!("127.0.0.1:{}", default_grpc_port())),
    );
    nodes
}

pub fn default_max_open_files() -> Option<i32> {
    Some(10000_i32)
}

pub fn default_rocksdb() -> Rocksdb {
    Rocksdb {
        max_open_files: default_max_open_files(),
        data_path: default_data_path(),
        ..Default::default()
    }
}

pub fn default_heartbeat() -> Heartbeat {
    Heartbeat {
        heartbeat_check_time_ms: default_heartbeat_check_time_ms(),
        heartbeat_timeout_ms: default_heartbeat_timeout_ms(),
    }
}

pub fn default_heartbeat_timeout_ms() -> u64 {
    30000
}

pub fn default_heartbeat_check_time_ms() -> u64 {
    1000
}
