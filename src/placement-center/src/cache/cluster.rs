use crate::storage::{cluster::ClusterInfo, node::NodeInfo};
use common_base::tools::now_mills;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ClusterCache {
    pub cluster_list: DashMap<String, ClusterInfo>,
    pub node_list: DashMap<String, NodeInfo>,
    pub node_heartbeat: DashMap<String, u128>,
}

impl ClusterCache {
    pub fn new() -> ClusterCache {
        return ClusterCache {
            cluster_list: DashMap::with_capacity(2),
            node_heartbeat: DashMap::with_capacity(2),
            node_list: DashMap::with_capacity(2),
        };
    }

    pub fn add_cluster(&self, cluster: ClusterInfo) {
        self.cluster_list
            .insert(cluster.cluster_name.clone(), cluster);
    }

    pub fn add_node(& self, node: NodeInfo) {
        let key = node_key(node.cluster_name.clone(), node.node_id);
        self.node_list.insert(key.clone(), node.clone());
        self.add_cluster_node(node.cluster_name, node.node_id);
        self.heart_time(key, now_mills());
    }

    pub fn remove_node(& self, cluster_name: String, node_id: u64) {
        let key = node_key(cluster_name, node_id);
        self.node_list.remove(&key);
        self.node_heartbeat.remove(&key);
        self.remove_cluster_node(cluster_name, node_id);
    }

    pub fn heart_time(& self, node_id: String, time: u128) {
        self.node_heartbeat.insert(node_id, time);
    }

    fn add_cluster_node(&self, cluster_name: String, node_id: u64) {
        if !self.cluster_list.contains_key(&cluster_name) {
            return;
        }
        if let Some(mut cluster) = self.cluster_list.get_mut(&cluster_name) {
            cluster.nodes.push(node_id);
        }
    }

    fn remove_cluster_node(& self, cluster_name: String, node_id: u64) {
        if !self.cluster_list.contains_key(&cluster_name) {
            return;
        }
        if let Some(mut cluster) = self.cluster_list.get_mut(&cluster_name) {
            match cluster.nodes.binary_search(&node_id) {
                Ok(index) => {
                    cluster.nodes.remove(index);
                }
                Err(_) => {}
            }
        }
    }
}

pub fn node_key(cluster_name: String, node_id: u64) -> String {
    return format!("{}_{}", cluster_name, node_id);
}
