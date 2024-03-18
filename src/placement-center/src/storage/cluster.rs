use super::{
    keys::{key_all_cluster, key_cluster},
    rocksdb::RocksDBEngine,
};
use common_base::log::error_meta;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub cluster_uid: String,
    pub cluster_name: String,
    pub cluster_type: String,
    pub nodes: Vec<u64>,
    pub create_time: u128,
}

pub struct ClusterStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ClusterStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ClusterStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save_cluster(&self, cluster_info: ClusterInfo) {
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

    pub fn get_cluster(&self, cluster_name: &String) -> Option<ClusterInfo> {
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

    pub fn save_all_cluster(&self, cluster_name: String) {
        let mut all_cluster = self.get_all_cluster();
        if !all_cluster.contains(&cluster_name) {
            all_cluster.push(cluster_name);
            let cf = self.rocksdb_engine_handler.cf_cluster();
            let key = key_all_cluster();
            match self.rocksdb_engine_handler.write(cf, &key, &all_cluster) {
                Ok(_) => {}
                Err(e) => {
                    error_meta(&e);
                }
            }
        }
    }

    pub fn get_all_cluster(&self) -> Vec<String> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = key_all_cluster();
        match self.rocksdb_engine_handler.read::<Vec<String>>(cf, &key) {
            Ok(data) => {
                if let Some(da) = data {
                    return da;
                }
            }
            Err(_) => {}
        };
        return Vec::new();
    }

    pub fn add_cluster_node(&self, cluster_name: &String, node_id: u64) {
        if let Some(mut cluster_info) = self.get_cluster(&cluster_name) {
            if !cluster_info.nodes.contains(&node_id) {
                cluster_info.nodes.push(node_id);
                self.save_cluster(cluster_info);
            }
        }
    }

    pub fn remove_cluster_node(&self, cluster_name: &String, node_id: u64) {
        if let Some(mut cluster_info) = self.get_cluster(&cluster_name) {
            match cluster_info.nodes.binary_search(&node_id) {
                Ok(index) => {
                    cluster_info.nodes.remove(index);
                    self.save_cluster(cluster_info);
                }
                Err(_) => {}
            }
        }
    }

    pub fn all_cluster(&self) -> Vec<ClusterInfo> {
        let all_cluster = self.get_all_cluster();

        let mut result = Vec::new();
        for cluster_name in all_cluster {
            if let Some(cluster_info) = self.get_cluster(&cluster_name) {
                result.push(cluster_info);
            }
        }
        return result;
    }
}
