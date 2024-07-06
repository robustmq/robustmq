use crate::storage::{keys::key_resource_idempotent, rocksdb::RocksDBEngine, StorageDataWrap};
use common_base::{errors::RobustMQError, tools::now_second};
use prost::Message;
use std::sync::Arc;

pub struct IdempotentStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl IdempotentStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        IdempotentStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(
        &self,
        cluster_name: &String,
        producer_id: &String,
        seq_num: u64,
    ) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = key_resource_idempotent(cluster_name, producer_id, seq_num);
        let data = StorageDataWrap::new(now_second().encode_to_vec());
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
        cluster_name: &String,
        producer_id: &String,
        seq_num: u64,
    ) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = key_resource_idempotent(cluster_name, producer_id, seq_num);
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
        cluster_name: &String,
        producer_id: &String,
        seq_num: u64,
    ) -> Result<Option<StorageDataWrap>, RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = key_resource_idempotent(cluster_name, producer_id, seq_num);
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
