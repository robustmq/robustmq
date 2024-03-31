use common_base::{errors::RobustMQError, log::error_meta, tools::unique_id};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::time::sleep;

use crate::server::lock::Lock;

use super::rocksdb::RocksDBEngine;

pub struct GlobalId {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl GlobalId {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        return GlobalId {
            rocksdb_engine_handler,
        };
    }

    pub fn generate_uniq_str(&self) -> String {
        return unique_id();
    }

    pub async fn generate_uniq_id(&self) -> Result<u64, RobustMQError> {
        let key = "global_id_uniq_int_id_lock".to_string();
        let lock = Lock::new(key.clone(), self.rocksdb_engine_handler.clone());
        let now = Instant::now();
        loop {
            if now.elapsed().as_secs() > 30 {
                return Err(RobustMQError::CommmonError(format!(
                    "generate_uniq_id waits for distributed lock timeout, key: {0}",
                    key.clone()
                )));
            }
            let flag = lock.lock_exists();
            println!("{}",flag);
            if !flag {
                match lock.lock() {
                    Ok(_) => {
                        let id = self.uniq_id();
                        match lock.un_lock() {
                            Ok(_) => {
                                return Ok(id);
                            }
                            Err(e) => {
                                error_meta(&e.to_string());
                            }
                        }
                    }
                    Err(e) => error_meta(&e.to_string()),
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
    }

    fn uniq_id(&self) -> u64 {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = "global_id_uniq_int_id".to_string();
        let id: u64 = match self.rocksdb_engine_handler.read::<u64>(cf, &key) {
            Ok(data) => {
                if let Some(da) = data {
                    da
                } else {
                    0
                }
            }
            Err(_) => {
                return 0;
            }
        };
        let rid = id + 1;
        match self.rocksdb_engine_handler.write(cf, &key, &rid) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
        return rid;
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::storage::global_id::GlobalId;
    use crate::storage::rocksdb::RocksDBEngine;
    use common_base::config::placement_center::PlacementCenterConfig;

    #[tokio::test]
    async fn unique_id_string() {
        let mut conf = PlacementCenterConfig::default();
        conf.data_path = "/tmp/test_fold1/data".to_string();
        let rocksdb_engine_handler: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(&conf));
        let gl = GlobalId::new(rocksdb_engine_handler);
        let r = gl.generate_uniq_str();
        assert!(!r.is_empty())
    }

    #[tokio::test]
    async fn unique_id_int() {
        let mut conf = PlacementCenterConfig::default();
        conf.data_path = "/tmp/test_fold1/data".to_string();
        let rocksdb_engine_handler: Arc<RocksDBEngine> = Arc::new(RocksDBEngine::new(&conf));
        let gl = GlobalId::new(rocksdb_engine_handler);
        let da1 = gl.generate_uniq_id().await.unwrap();
        let da2 = gl.generate_uniq_id().await.unwrap();
        let da3 = gl.generate_uniq_id().await.unwrap();
        assert!((da1 + 1) == da2);
        assert!((da1 + 2) == da3);
    }
}
