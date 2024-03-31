use std::sync::Arc;

use common_base::log::error_meta;

use super::rocksdb::RocksDBEngine;

pub struct KvStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl KvStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        KvStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn set(&self, key: String, value: String) {
        let cf = self.rocksdb_engine_handler.cf_data();
        match self.rocksdb_engine_handler.write(cf, &key, &value) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    pub fn delete(&self, key: String) {
        let cf = self.rocksdb_engine_handler.cf_data();
        match self.rocksdb_engine_handler.delete(cf, &key) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }

    pub fn get(&self, key: String) -> Option<String> {
        let cf = self.rocksdb_engine_handler.cf_data();
        match self.rocksdb_engine_handler.read::<String>(cf, &key) {
            Ok(cluster_info) => {
                return cluster_info;
            }
            Err(_) => {}
        }
        return None;
    }

    pub fn exists(&self, key: String) -> bool {
        let cf = self.rocksdb_engine_handler.cf_data();
        return self.rocksdb_engine_handler.exist(cf, &key);
    }
}
