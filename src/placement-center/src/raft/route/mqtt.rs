use crate::storage::{mqtt::user::MQTTUserStorage, rocksdb::RocksDBEngine};
use common_base::errors::RobustMQError;
use prost::Message as _;
use protocol::placement_center::generate::mqtt::{CreateUserRequest, DeleteUserRequest};
use std::sync::Arc;
use tonic::Status;

pub struct DataRouteMQTT {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    mqtt_user_storage: MQTTUserStorage,
}
impl DataRouteMQTT {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        let mqtt_user_storage = MQTTUserStorage::new(rocksdb_engine_handler.clone());
        return DataRouteMQTT {
            rocksdb_engine_handler,
            mqtt_user_storage,
        };
    }

    pub fn create_user(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = CreateUserRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let storage = MQTTUserStorage::new(self.rocksdb_engine_handler.clone());
        match storage.set(req.cluster_name, req.user.unwrap()) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn delete_user(&self, value: Vec<u8>) -> Result<(), RobustMQError> {
        let req = DeleteUserRequest::decode(value.as_ref())
            .map_err(|e| Status::invalid_argument(e.to_string()))
            .unwrap();
        let storage = MQTTUserStorage::new(self.rocksdb_engine_handler.clone());
        match storage.delete(req.cluster_name, req.username) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}
