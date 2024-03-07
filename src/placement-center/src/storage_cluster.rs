use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::storage::cluster_storage::{ClusterInfo, NodeInfo, ShardInfo};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct StorageCluster {
    pub cluster_list: HashMap<String, ClusterInfo>,
    pub node_list: HashMap<u64, NodeInfo>,
    pub shard_list: HashMap<String, ShardInfo>,
    pub node_heartbeat: HashMap<u64, u128>,
}

impl StorageCluster {
    pub fn new() -> StorageCluster {
        let bc = StorageCluster::default();
        return bc;
    }

    pub fn add_cluster(&mut self, cluster: ClusterInfo) {
        self.cluster_list
            .insert(cluster.cluster_name.clone(), cluster);
    }

    pub fn add_node(&mut self, node: NodeInfo) {
        self.node_list.insert(node.node_id, node);
    }

    pub fn remove_node(&mut self, node_id: u64) {
        self.node_list.remove(&node_id);
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
}
