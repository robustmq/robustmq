use crate::storage::rocksdb::RocksDBEngine;
use common_base::errors::RobustMQError;
use std::sync::Arc;

pub struct KvStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl KvStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        KvStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn set(&self, key: String, value: String) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_data();
        match self.rocksdb_engine_handler.write(cf, &key, &value) {
            Ok(_) => {}
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
        return Ok(());
    }

    pub fn delete(&self, key: String) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_data();
        match self.rocksdb_engine_handler.delete(cf, &key) {
            Ok(_) => {}
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
        return Ok(());
    }

    pub fn get(&self, key: String) -> Result<Option<String>, RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_data();
        match self.rocksdb_engine_handler.read::<String>(cf, &key) {
            Ok(cluster_info) => {
                return Ok(cluster_info);
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
            }
        }
    }

    pub fn exists(&self, key: String) -> bool {
        let cf = self.rocksdb_engine_handler.cf_data();
        return self.rocksdb_engine_handler.exist(cf, &key);
    }
}
