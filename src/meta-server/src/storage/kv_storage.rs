use std::sync::{Arc, RwLock};

use super::{raft_core::RaftRocksDBStorageCore, rocksdb::RocksDBStorage};

pub struct KvStorage {
    rds: RocksDBStorage,
}

impl KvStorage {
    pub fn new(rds: RocksDBStorage) -> Self {
        KvStorage { rds }
    }

    pub fn set(&self, key: String, value: Vec<u8>) {
    }

    pub fn get(&self, key: String) {}

    pub fn delete(&self, key: String) {}

    pub fn exists(&self, key: String) {}
}
