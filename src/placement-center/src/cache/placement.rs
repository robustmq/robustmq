use common_base::tools::now_mills;
use dashmap::DashMap;
use metadata_struct::placement::{broker_node::BrokerNode, cluster::ClusterInfo};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PlacementCacheManager {
    pub cluster_list: DashMap<String, ClusterInfo>,
    pub node_list: DashMap<String, BrokerNode>,
    pub node_heartbeat: DashMap<String, u128>,
}

impl PlacementCacheManager {
    pub fn new() -> PlacementCacheManager {
        return PlacementCacheManager {
            cluster_list: DashMap::with_capacity(2),
            node_heartbeat: DashMap::with_capacity(2),
            node_list: DashMap::with_capacity(2),
        };
    }

    pub fn add_cluster(&self, cluster: ClusterInfo) {
        self.cluster_list
            .insert(cluster.cluster_name.clone(), cluster);
    }

    pub fn add_node(&self, node: BrokerNode) {
        let key = node_key(node.cluster_name.clone(), node.node_id);
        self.node_list.insert(key.clone(), node.clone());
        self.heart_time(key, now_mills());
    }

    pub fn remove_node(&self, cluster_name: String, node_id: u64) {
        let key = node_key(cluster_name.clone(), node_id);
        self.node_list.remove(&key);
        self.node_heartbeat.remove(&key);
    }

    pub fn get_node(&self, cluster_name: String, node_id: u64) -> Option<BrokerNode> {
        let key = node_key(cluster_name.clone(), node_id);
        if let Some(value) = self.node_list.get(&key) {
            return Some(value.clone());
        }
        return None;
    }

    pub fn get_cluster_node_addr(&self, cluster_name: &String) -> Vec<String> {
        let mut results = Vec::new();
        for (_, node) in self.node_list.clone() {
            if node.cluster_name.eq(cluster_name) {
                results.push(node.node_inner_addr);
            }
        }
        return results;
    }

    pub fn heart_time(&self, node_id: String, time: u128) {
        self.node_heartbeat.insert(node_id, time);
    }
}

pub fn node_key(cluster_name: String, node_id: u64) -> String {
    return format!("{}_{}", cluster_name, node_id);
}
