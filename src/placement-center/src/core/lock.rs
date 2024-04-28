use crate::storage::rocksdb::RocksDBEngine;
use common_base::errors::RobustMQError;
use std::sync::Arc;

pub struct Lock {
    pub key: String,
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl Lock {
    pub fn new(key: String, rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        return Lock {
            key,
            rocksdb_engine_handler,
        };
    }

    pub fn lock(&self) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let val = 1;
        match self.rocksdb_engine_handler.write(cf, &self.key, &val) {
            Ok(_) => {}
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
        return Ok(());
    }

    pub fn un_lock(&self) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        match self.rocksdb_engine_handler.delete(cf, &self.key) {
            Ok(_) => {}
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
        return Ok(());
    }

    pub fn lock_exists(&self) -> bool {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        return self.rocksdb_engine_handler.exist(cf, &self.key);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::core::lock::Lock;
    use crate::storage::rocksdb::RocksDBEngine;
    use common_base::config::placement_center::PlacementCenterConfig;

    #[tokio::test]
    async fn lock_test() {
        let mut conf = PlacementCenterConfig::default();
        conf.data_path = "/tmp/test_fold1/data".to_string();
        let rocksdb_engine_handler: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(&conf));
        let key = "test_lock".to_string();
        let lc = Lock::new(key, rocksdb_engine_handler);
        let _ = lc.lock();
        assert!(lc.lock_exists());
        let _ = lc.un_lock();
        assert!(!lc.lock_exists());
    }
}
