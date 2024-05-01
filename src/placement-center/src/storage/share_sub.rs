use crate::structs::share_sub::ShareSub;
use common_base::{errors::RobustMQError, log::error_meta};
use std::sync::Arc;

use super::{keys::share_sub_key, rocksdb::RocksDBEngine};

pub struct ShareSubStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShareSubStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ShareSubStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, share_sub: ShareSub) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = share_sub_key(share_sub.cluster_name.clone(), share_sub.group_name.clone());
        match self.rocksdb_engine_handler.write(cf, &key, &share_sub) {
            Ok(_) => return Ok(()),
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        }
    }

    pub fn get(&self, cluster_name: String, group_name: String) -> Option<ShareSub> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = share_sub_key(cluster_name, group_name);
        match self.rocksdb_engine_handler.read::<ShareSub>(cf, &key) {
            Ok(ci) => {
                return ci;
            }
            Err(_) => {}
        }
        return None;
    }

    pub fn delete(&self, cluster_name: String, group_name: String) {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = share_sub_key(cluster_name, group_name);
        match self.rocksdb_engine_handler.delete(cf, &key) {
            Ok(_) => {}
            Err(e) => {
                error_meta(&e);
            }
        }
    }
}
