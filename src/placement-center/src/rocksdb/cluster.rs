use std::sync::Arc;

use super::{keys::key_cluster, rocksdb::RocksDBEngine};
use common::{config::placement_center::placement_center_conf, log::error_meta};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ClusterInfo {
    pub cluster_name: String,
    pub cluster_type: String,
    pub nodes: Vec<u64>,
}

pub struct ClusterStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ClusterStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let config = placement_center_conf();
        ClusterStorage {
            rocksdb_engine_handler,
        }
    }

    // save cluster info
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

    // get cluster info
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
}
