use std::sync::Arc;
use common_base::errors::RobustMQError;
use prost::Message as _;
use protocol::placement_center::generate::kv::{DeleteRequest, SetRequest};
use tonic::Status;
use crate::storage::{kv::KvStorage, rocksdb::RocksDBEngine};
pub struct DataRouteKv {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    kv_storage: KvStorage,
}

impl DataRouteKv {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let kv_storage = KvStorage::new(rocksdb_engine_handler.clone());
        return DataRouteKv {
            rocksdb_engine_handler,
            kv_storage,
        };
    }
    pub fn set(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: SetRequest = SetRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        self.kv_storage.set(req.key, req.value);
        return Ok(());
    }

    pub fn delete(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req: DeleteRequest = DeleteRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        self.kv_storage.delete(req.key);
        return Ok(());
    }
}
