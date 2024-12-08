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

use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::placement::cluster::ClusterInfo;
use metadata_struct::placement::node::BrokerNode;
use serde::{Deserialize, Serialize};

use super::heartbeat::NodeHeartbeatData;
use crate::storage::placement::cluster::ClusterStorage;
use crate::storage::placement::node::NodeStorage;
use crate::storage::rocksdb::RocksDBEngine;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PlacementCacheManager {
    // broker cluster & node
    pub cluster_list: DashMap<String, ClusterInfo>,
    pub node_list: DashMap<String, DashMap<u64, BrokerNode>>,
    pub node_heartbeat: DashMap<String, NodeHeartbeatData>,
}

impl PlacementCacheManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> PlacementCacheManager {
        let mut cache = PlacementCacheManager {
            cluster_list: DashMap::with_capacity(2),
            node_heartbeat: DashMap::with_capacity(2),
            node_list: DashMap::with_capacity(2),
        };
        cache.load_cache(rocksdb_engine_handler);
        cache
    }

    pub fn add_broker_cluster(&self, cluster: &ClusterInfo) {
        self.cluster_list
            .insert(cluster.cluster_name.clone(), cluster.clone());
    }

    pub fn add_broker_node(&self, node: BrokerNode) {
        if let Some(data) = self.node_list.get_mut(&node.cluster_name) {
            data.insert(node.node_id, node);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(node.node_id, node.clone());
            self.node_list.insert(node.cluster_name.clone(), data);
        }
    }

    pub fn remove_broker_node(
        &self,
        cluster_name: &str,
        node_id: u64,
    ) -> Option<(u64, BrokerNode)> {
        if let Some(data) = self.node_list.get_mut(cluster_name) {
            return data.remove(&node_id);
        }
        None
    }

    pub fn get_broker_num(&self, cluster_name: &str) -> usize {
        if let Some(data) = self.node_list.get(cluster_name) {
            return data.len();
        }
        0
    }

    pub fn get_broker_node(&self, cluster_name: &str, node_id: u64) -> Option<BrokerNode> {
        if let Some(data) = self.node_list.get(cluster_name) {
            if let Some(value) = data.get(&node_id) {
                return Some(value.clone());
            }
        }
        None
    }

    pub fn get_broker_node_addr_by_cluster(&self, cluster_name: &str) -> Vec<String> {
        let mut results = Vec::new();
        if let Some(data) = self.node_list.get(cluster_name) {
            for (_, node) in data.clone() {
                if node.cluster_name.eq(cluster_name) {
                    results.push(node.node_inner_addr);
                }
            }
        }
        results
    }

    pub fn get_broker_node_id_by_cluster(&self, cluster_name: &str) -> Vec<u64> {
        let mut results = Vec::new();
        if let Some(data) = self.node_list.get(cluster_name) {
            for (_, node) in data.clone() {
                if node.cluster_name.eq(cluster_name) {
                    results.push(node.node_id);
                }
            }
        }
        results
    }

    pub fn report_broker_heart(&self, cluster_name: &str, node_id: u64) {
        let key = self.node_key(cluster_name, node_id);
        let data = NodeHeartbeatData {
            cluster_name: cluster_name.to_string(),
            node_id,
            time: now_second(),
        };
        self.node_heartbeat.insert(key, data);
    }

    pub fn remove_broker_heart(&self, cluster_name: &str, node_id: u64) {
        let key = self.node_key(cluster_name, node_id);
        self.node_heartbeat.remove(&key);
    }

    pub fn get_broker_heart(&self, cluster_name: &str, node_id: u64) -> Option<NodeHeartbeatData> {
        let key = self.node_key(cluster_name, node_id);
        if let Some(heart) = self.node_heartbeat.get(&key) {
            return Some(heart.clone());
        }
        None
    }

    pub fn load_cache(&mut self, rocksdb_engine_handler: Arc<RocksDBEngine>) {
        let cluster = ClusterStorage::new(rocksdb_engine_handler.clone());
        if let Ok(result) = cluster.list(None) {
            for cluster in result {
                self.add_broker_cluster(&cluster);
            }
        }

        let node = NodeStorage::new(rocksdb_engine_handler.clone());
        if let Ok(result) = node.list(None) {
            for bn in result {
                self.add_broker_node(bn);
            }
        }
    }

    fn node_key(&self, cluster_name: &str, node_id: u64) -> String {
        format!("{}_{}", cluster_name, node_id)
    }
}
