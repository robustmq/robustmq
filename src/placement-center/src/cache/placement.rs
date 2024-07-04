use common_base::tools::now_second;
use dashmap::DashMap;
use metadata_struct::placement::{broker_node::BrokerNode, cluster::ClusterInfo};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::storage::{
    placement::{cluster::ClusterStorage, node::NodeStorage},
    rocksdb::RocksDBEngine,
};

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct PlacementCacheManager {
    pub cluster_list: DashMap<String, ClusterInfo>,
    pub node_list: DashMap<String, DashMap<u64, BrokerNode>>,
    pub node_heartbeat: DashMap<String, DashMap<u64, u64>>,
}

impl PlacementCacheManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> PlacementCacheManager {
        let cache = PlacementCacheManager {
            cluster_list: DashMap::with_capacity(2),
            node_heartbeat: DashMap::with_capacity(2),
            node_list: DashMap::with_capacity(2),
        };
        cache.load_cache(rocksdb_engine_handler);
        return cache;
    }

    pub fn add_cluster(&self, cluster: ClusterInfo) {
        self.cluster_list
            .insert(cluster.cluster_name.clone(), cluster);
    }

    pub fn add_node(&self, node: BrokerNode) {
        self.heart_time(&node.cluster_name, node.node_id, now_second());
        
        if let Some(data) = self.node_list.get_mut(&node.cluster_name) {
            data.insert(node.node_id, node);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(node.node_id, node.clone());
            self.node_list.insert(node.cluster_name.clone(), data);
        }
    }

    pub fn remove_node(&self, cluster_name: &String, node_id: u64) {
        if let Some(data) = self.node_list.get_mut(cluster_name) {
            data.remove(&node_id);
        }
        if let Some(data) = self.node_heartbeat.get_mut(cluster_name) {
            data.remove(&node_id);
        }
    }

    pub fn get_node(&self, cluster_name: &String, node_id: u64) -> Option<BrokerNode> {
        if let Some(data) = self.node_list.get_mut(cluster_name) {
            if let Some(value) = data.get(&node_id) {
                return Some(value.clone());
            }
        }
        return None;
    }

    pub fn get_cluster_node_addr(&self, cluster_name: &String) -> Vec<String> {
        let mut results = Vec::new();
        if let Some(data) = self.node_list.get_mut(cluster_name) {
            for (_, node) in data.clone() {
                if node.cluster_name.eq(cluster_name) {
                    results.push(node.node_inner_addr);
                }
            }
        }
        return results;
    }

    pub fn heart_time(&self, cluster_name: &String, node_id: u64, time: u64) {
        if let Some(data) = self.node_heartbeat.get_mut(cluster_name) {
            data.insert(node_id, time);
        } else {
            let data = DashMap::with_capacity(2);
            data.insert(node_id, time);
            self.node_heartbeat.insert(cluster_name.clone(), data);
        }
    }

    pub fn load_cache(&self, rocksdb_engine_handler: Arc<RocksDBEngine>) {
        let cluster = ClusterStorage::new(rocksdb_engine_handler.clone());
        match cluster.list(None) {
            Ok(result) => {
                for cluster in result {
                    self.add_cluster(cluster);
                }
            }
            Err(_) => {}
        }

        let node = NodeStorage::new(rocksdb_engine_handler.clone());
        match node.list(None) {
            Ok(result) => {
                for bn in result {
                    self.add_node(bn);
                }
            }
            Err(_) => {}
        }
    }
}
