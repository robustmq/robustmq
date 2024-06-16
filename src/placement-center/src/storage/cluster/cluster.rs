use crate::{
    storage::{keys::key_cluster, rocksdb::RocksDBEngine},
    structs::cluster::ClusterInfo,
};
use common_base::log::error_meta;
use std::sync::Arc;

pub struct ClusterStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ClusterStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ClusterStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, cluster_info: ClusterInfo) {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let cluster_key = key_cluster(&cluster_info.cluster_name);
        match self
            .rocksdb_engine_handler
            .write(cf, &cluster_key, &cluster_info)
        {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    pub fn get(&self, cluster_name: &String) -> Option<ClusterInfo> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let cluster_key = key_cluster(&cluster_name);
        match self
            .rocksdb_engine_handler
            .read::<ClusterInfo>(cf, &cluster_key)
        {
            Ok(cluster_info) => {
                return cluster_info;
            }
            Err(_) => {}
        }
        return None;
    }

    pub fn add_cluster_node(&self, cluster_name: &String, node_id: u64) {
        if let Some(mut cluster_info) = self.get(&cluster_name) {
            if !cluster_info.nodes.contains(&node_id) {
                cluster_info.nodes.push(node_id);
                self.save(cluster_info);
            }
        }
    }

    pub fn remove_cluster_node(&self, cluster_name: &String, node_id: u64) {
        if let Some(mut cluster_info) = self.get(&cluster_name) {
            match cluster_info.nodes.binary_search(&node_id) {
                Ok(index) => {
                    cluster_info.nodes.remove(index);
                    self.save(cluster_info);
                }
                Err(_) => {}
            }
        }
    }

    pub fn all_cluster(&self) -> Vec<ClusterInfo> {
        let mut result = Vec::new();

        return result;
    }
}
