use std::sync::Arc;
use common::log::error_meta;
use serde::{Deserialize, Serialize};
use super::{keys::key_cluster, rocksdb::RocksDBEngine};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub cluster_name: String,
    pub cluster_type: String,
    pub nodes: Vec<u64>,
}

#[derive(Clone)]
pub struct ClusterStorage {
    rds: Arc<RocksDBEngine>,
}

impl ClusterStorage {
    pub fn new(rds: Arc<RocksDBEngine>) -> Self {
        ClusterStorage { rds }
    }

    // save cluster info
    pub fn save_cluster(&self, cluster_info: ClusterInfo) {
        let cf = self.rds.cf_cluster();
        let cluster_key = key_cluster(&cluster_info.cluster_name);
        match self.rds.write(cf, &cluster_key, &cluster_info) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    // get cluster info
    pub fn get_cluster(&self, cluster_name: &String) -> Option<ClusterInfo> {
        let cf = self.rds.cf_cluster();
        let cluster_key = key_cluster(&cluster_name);
        match self.rds.read::<ClusterInfo>(cf, &cluster_key) {
            Ok(cluster_info) => {
                return cluster_info;
            }
            Err(_) => {}
        }
        return None;
    }
}
