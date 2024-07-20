use crate::storage::{
    keys::{key_cluster, key_cluster_prefix, key_cluster_prefix_by_type},
    rocksdb::RocksDBEngine,
};
use common_base::errors::RobustMQError;
use metadata_struct::placement::cluster::ClusterInfo;
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

    pub fn save(&self, cluster_info: ClusterInfo) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let cluster_key = key_cluster(&cluster_info.cluster_type, &cluster_info.cluster_name);
        match self
            .rocksdb_engine_handler
            .write(cf, &cluster_key, &cluster_info)
        {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }

    pub fn get(
        &self,
        cluster_type: &String,
        cluster_name: &String,
    ) -> Result<Option<ClusterInfo>, RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let cluster_key = key_cluster(cluster_type, cluster_name);
        match self
            .rocksdb_engine_handler
            .read::<ClusterInfo>(cf, &cluster_key)
        {
            Ok(cluster_info) => {
                return Ok(cluster_info);
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }

    pub fn delete(
        &self,
        cluster_type: &String,
        cluster_name: &String,
    ) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_mqtt();
        let key: String = key_cluster(cluster_type, cluster_name);
        match self.rocksdb_engine_handler.delete(cf, &key) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }

    pub fn list(&self, cluster_type: Option<String>) -> Result<Vec<ClusterInfo>, RobustMQError> {
        let prefix_key = if let Some(ct) = cluster_type {
            key_cluster_prefix_by_type(&ct)
        } else {
            key_cluster_prefix()
        };
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let data_list = self.rocksdb_engine_handler.read_prefix(cf, &prefix_key);
        let mut results = Vec::new();
        for raw in data_list {
            for (k, v) in raw {
                match serde_json::from_slice::<ClusterInfo>(v.as_ref()) {
                    Ok(v) => results.push(v),
                    Err(_) => {
                        continue;
                    }
                }
            }
        }
        return Ok(results);
    }
}
