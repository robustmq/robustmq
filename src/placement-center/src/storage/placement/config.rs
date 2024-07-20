use crate::storage::{keys::key_resource_config, rocksdb::RocksDBEngine, StorageDataWrap};
use common_base::errors::RobustMQError;
use std::sync::Arc;

pub struct ResourceConfigStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ResourceConfigStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ResourceConfigStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(
        &self,
        cluster_name: String,
        resource_key: Vec<String>,
        config: Vec<u8>,
    ) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = key_resource_config(cluster_name, resource_key.join("/"));
        let data = StorageDataWrap::new(config);
        match self.rocksdb_engine_handler.write(cf, &key, &data) {
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
        let key = key_resource_config(cluster_name, resource_key.join("/"));
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
    ) -> Result<Option<StorageDataWrap>, RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = key_resource_config(cluster_name, resource_key.join("/"));
        match self
            .rocksdb_engine_handler
            .read::<StorageDataWrap>(cf, &key)
        {
            Ok(cluster_info) => {
                return Ok(cluster_info);
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }
}
