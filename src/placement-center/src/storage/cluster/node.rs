use crate::{
    storage::{keys::key_node, rocksdb::RocksDBEngine},
    structs::node::Node,
};
use common_base::log::error_meta;
use std::sync::Arc;

use super::cluster::ClusterStorage;

pub struct NodeStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cluster_storage: ClusterStorage,
}

impl NodeStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let cluster_storage = ClusterStorage::new(rocksdb_engine_handler.clone());
        NodeStorage {
            rocksdb_engine_handler,
            cluster_storage,
        }
    }

    pub fn list(&self, cluster_name: String) -> Vec<Node> {
        let mut result = Vec::new();
        let cf = self.rocksdb_engine_handler.cf_cluster();

        let cluster_info = self.cluster_storage.get(&cluster_name);
        if cluster_info.is_none() {
            return result;
        }

        for node_id in cluster_info.unwrap().nodes {
            let key = key_node(&cluster_name, node_id);
            match self.rocksdb_engine_handler.read::<Node>(cf, &key) {
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

    pub fn save(&self, cluster_name: String, cluster_type: String, node: Node) {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let node_key = key_node(&cluster_name, node.node_id);
        match self.rocksdb_engine_handler.write(cf, &node_key, &node) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    pub fn delete(&self, cluster_name: &String, node_id: u64) {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let node_key = key_node(&cluster_name, node_id);
        match self.rocksdb_engine_handler.delete(cf, &node_key) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    pub fn get(&self, cluster_name: String, node_id: u64) -> Option<Node> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let cluster_key = key_node(&cluster_name, node_id);
        match self.rocksdb_engine_handler.read::<Node>(cf, &cluster_key) {
            Ok(cluster_info) => {
                return cluster_info;
            }
            Err(_) => {}
        }
        return None;
    }
}
