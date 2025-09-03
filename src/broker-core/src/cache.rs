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

use common_base::{node_status::NodeStatus, tools::now_second};
use common_config::config::BrokerConfig;
use dashmap::DashMap;
use metadata_struct::placement::node::BrokerNode;

pub struct BrokerCacheManager {
    pub start_time: u64,

    // node list
    pub node_lists: DashMap<u64, BrokerNode>,

    // cluster_name
    pub cluster_name: String,

    // (cluster_name, Cluster)
    pub cluster_info: DashMap<String, BrokerConfig>,

    // (cluster_name, Status)
    pub status: DashMap<String, NodeStatus>,
}
impl BrokerCacheManager {
    pub fn new(cluster_name: String) -> Self {
        BrokerCacheManager {
            cluster_name,
            start_time: now_second(),
            node_lists: DashMap::with_capacity(2),
            cluster_info: DashMap::with_capacity(1),
            status: DashMap::with_capacity(2),
        }
    }

    // node
    pub fn add_node(&self, node: BrokerNode) {
        self.node_lists.insert(node.node_id, node);
    }

    pub fn remove_node(&self, node: BrokerNode) {
        self.node_lists.remove(&node.node_id);
    }

    pub fn node_list(&self) -> Vec<BrokerNode> {
        self.node_lists
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    // get start time
    pub fn get_start_time(&self) -> u64 {
        self.start_time
    }
}
