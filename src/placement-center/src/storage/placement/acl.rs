use crate::storage::{keys::key_resource_acl, rocksdb::RocksDBEngine};
use common_base::errors::RobustMQError;
use std::sync::Arc;

pub struct AclStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl AclStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        AclStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(
        &self,
        cluster_name: String,
        resource_key: Vec<String>,
        acl: String,
    ) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = key_resource_acl(cluster_name, resource_key.join("/"));
        match self.rocksdb_engine_handler.write(cf, &key, &acl) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }

    pub fn delete(
        &self,
        cluster_name: String,
        resource_key: Vec<String>,
    ) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = key_resource_acl(cluster_name, resource_key.join("/"));
        match self.rocksdb_engine_handler.delete(cf, &key) {
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
        cluster_name: String,
        resource_key: Vec<String>,
    ) -> Result<Option<String>, RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = key_resource_acl(cluster_name, resource_key.join("/"));
        match self.rocksdb_engine_handler.read::<String>(cf, &key) {
            Ok(cluster_info) => {
                return Ok(cluster_info);
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }
}
