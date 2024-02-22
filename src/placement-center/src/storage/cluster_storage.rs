use std::sync::{Arc, RwLock};

use super::rocksdb::RocksDBStorage;

pub struct ClusterStorage {
    rds: Arc<RocksDBStorage>,
}

impl ClusterStorage {
    pub fn new(rds: Arc<RocksDBStorage>) -> Self {
        ClusterStorage { rds }
    }

    pub fn save_broker_info(&self) {}

    pub fn get_broker_info(&self) {}

}
