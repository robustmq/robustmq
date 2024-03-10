use crate::rocksdb::{cluster::ClusterInfo, node::NodeInfo, shard::ShardInfo};
use common::tools::now_mills;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct EngineClusterCache {
    pub cluster_list: HashMap<String, ClusterInfo>,
    pub node_list: HashMap<String, NodeInfo>,
    pub shard_list: HashMap<String, ShardInfo>,
    pub node_heartbeat: HashMap<u64, u128>,
}

impl EngineClusterCache {
    pub fn new() -> EngineClusterCache {
        let bc = EngineClusterCache::default();
        return bc;
    }

    pub fn add_cluster(&mut self, cluster: ClusterInfo) {
        self.cluster_list
            .insert(cluster.cluster_name.clone(), cluster);
    }

    pub fn add_cluster_node(&mut self, cluster_name: String, node_id: u64) {
        if !self.cluster_list.contains_key(&cluster_name) {
            return;
        }
        let mut cluster = self.cluster_list.remove(&cluster_name).unwrap();
        if !cluster.nodes.contains(&node_id) {
            cluster.nodes.push(node_id);
        }
        self.add_cluster(cluster);
    }

    pub fn remove_cluster_node(&mut self, cluster_name: String, node_id: u64) {
        if !self.cluster_list.contains_key(&cluster_name) {
            return;
        }
        let mut cluster = self.cluster_list.remove(&cluster_name).unwrap();
        match cluster.nodes.binary_search(&node_id) {
            Ok(index) => {
                cluster.nodes.remove(index);
                self.add_cluster(cluster);
            }
            Err(_) => {
                self.add_cluster(cluster);
            }
        }
    }

    pub fn add_node(&mut self, node: NodeInfo) {
        self.node_list.insert(
            self.node_key(node.cluster_name.clone(), node.node_id),
            node.clone(),
        );

        self.heart_time(node.node_id, now_mills());
    }

    pub fn remove_node(&mut self, cluster_name: String, node_id: u64) {
        self.node_list.remove(&self.node_key(cluster_name, node_id));
        self.node_heartbeat.remove(&node_id);
    }

    pub fn add_shard(&mut self, shard: ShardInfo) {
        self.shard_list.insert(shard.shard_name.clone(), shard);
    }

    pub fn remove_shard(&mut self, shard_name: String) {
        self.shard_list.remove(&shard_name);
    }

    pub fn heart_time(&mut self, node_id: u64, time: u128) {
        self.node_heartbeat.insert(node_id, time);
    }

    fn node_key(&self, cluster_name: String, node_id: u64) -> String {
        return format!("{}_{}", cluster_name, node_id);
    }
}
