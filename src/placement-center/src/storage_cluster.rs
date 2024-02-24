use std::collections::HashMap;

use crate::storage::cluster_storage::{ClusterInfo, NodeInfo, ShardInfo};

#[derive(Clone, Default, Debug)]
pub struct StorageCluster {
    pub cluster_list: HashMap<String, ClusterInfo>,
    pub node_list: HashMap<u64, NodeInfo>,
    pub shard_list: HashMap<String, ShardInfo>,
}

impl StorageCluster {
    pub fn new() -> StorageCluster {
        let bc = StorageCluster::default();
        return bc;
    }

    pub fn add_cluster(&mut self, cluster_info: ClusterInfo) {
        self.cluster_list
            .insert(cluster_info.cluster_name.clone(), cluster_info);
    }

    pub fn add_node(&self) {}

    pub fn remove_node(&self) {}

    pub fn add_shard(&self) {}

    pub fn remove_shard(&self) {}

    pub fn heart_time(&self) {}
}
