use std::sync::Arc;

use super::{
    cluster::{ClusterInfo, ClusterStorage},
    keys::key_node,
    rocksdb::RocksDBEngine,
};
use common::{config::placement_center::placement_center_conf, log::error_meta};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_uid: String,
    pub node_id: u64,
    pub node_ip: String,
    pub node_port: u32,
    pub cluster_name: String,
    pub create_time: u128,
}

pub struct NodeStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cluster_storage: ClusterStorage,
}

impl NodeStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let config = placement_center_conf();
        let cluster_storage = ClusterStorage::new(rocksdb_engine_handler.clone());
        NodeStorage {
            rocksdb_engine_handler,
            cluster_storage,
        }
    }

    pub fn node_list(&self, cluster_name: String) -> Vec<NodeInfo> {
        let mut result = Vec::new();
        let cf = self.rocksdb_engine_handler.cf_cluster();

        let cluster_info = self.cluster_storage.get_cluster(&cluster_name);
        if cluster_info.is_none() {
            return result;
        }

        for node_id in cluster_info.unwrap().nodes {
            let key = key_node(&cluster_name, node_id);
            match self.rocksdb_engine_handler.read::<NodeInfo>(cf, &key) {
                Ok(node_info) => {
                    if let Some(ni) = node_info {
                        result.push(ni);
                    }
                }
                Err(_) => {}
            };
        }
        return result;
    }

    pub fn save_node(&self, cluster_name: String, cluster_type: String, node: NodeInfo) {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let node_key = key_node(&cluster_name, node.node_id);
        match self.rocksdb_engine_handler.write(cf, &node_key, &node) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    pub fn delete_node(&self, cluster_name: &String, node_id: u64) {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let node_key = key_node(&cluster_name, node_id);
        match self.rocksdb_engine_handler.delete(cf, &node_key) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    pub fn get_node(&self, cluster_name: String, node_id: u64) -> Option<NodeInfo> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let cluster_key = key_node(&cluster_name, node_id);
        match self
            .rocksdb_engine_handler
            .read::<NodeInfo>(cf, &cluster_key)
        {
            Ok(cluster_info) => {
                return cluster_info;
            }
            Err(_) => {}
        }
        return None;
    }
}
