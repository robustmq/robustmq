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

use std::sync::Arc;

use common_base::{node_status::NodeStatus, tools::now_second};
use common_config::config::BrokerConfig;
use dashmap::DashMap;
use metadata_struct::meta::node::BrokerNode;
use tokio::sync::RwLock;

pub struct BrokerCacheManager {
    // start_time
    pub start_time: u64,

    // node list
    pub node_lists: DashMap<u64, BrokerNode>,

    // cluster_name
    pub cluster_name: String,

    // (cluster_name, Cluster)
    pub cluster_config: Arc<RwLock<BrokerConfig>>,

    // (cluster_name, Status)
    pub status: Arc<RwLock<NodeStatus>>,
}
impl BrokerCacheManager {
    pub fn new(cluster: BrokerConfig) -> Self {
        BrokerCacheManager {
            cluster_name: cluster.cluster_name.clone(),
            start_time: now_second(),
            node_lists: DashMap::with_capacity(2),
            cluster_config: Arc::new(RwLock::new(cluster.clone())),
            status: Arc::new(RwLock::new(NodeStatus::Starting)),
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

    // status
    pub async fn set_status(&self, status: NodeStatus) {
        let mut data = self.status.write().await;
        *data = status;
    }

    pub async fn get_status(&self) -> NodeStatus {
        self.status.read().await.clone()
    }

    pub async fn is_stop(&self) -> bool {
        self.get_status().await == NodeStatus::Stopping
    }

    // cluster config
    pub async fn set_cluster_config(&self, config: BrokerConfig) {
        let mut data = self.cluster_config.write().await;
        *data = config;
    }

    pub async fn get_cluster_config(&self) -> BrokerConfig {
        self.cluster_config.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::cache::BrokerCacheManager;
    use common_base::tools::now_second;
    use common_config::broker::default_broker_config;
    use metadata_struct::meta::node::BrokerNode;

    #[tokio::test]
    async fn start_time_operations() {
        let cache_manager = BrokerCacheManager::new(default_broker_config());
        let start_time = cache_manager.get_start_time();
        assert!(start_time > 0);
        assert!(start_time <= now_second());
    }

    #[tokio::test]
    async fn node_operations() {
        let cache_manager = BrokerCacheManager::new(default_broker_config());
        let node = BrokerNode {
            node_id: 1,
            node_ip: "127.0.0.1".to_string(),
            ..Default::default()
        };

        // add
        cache_manager.add_node(node.clone());

        // get
        let nodes = cache_manager.node_list();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].node_id, node.node_id);
        assert_eq!(nodes[0].node_ip, node.node_ip);

        // remove
        cache_manager.remove_node(node.clone());

        // get again
        let nodes = cache_manager.node_list();
        assert!(nodes.is_empty());
    }
}
