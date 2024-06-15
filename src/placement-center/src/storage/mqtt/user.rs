use crate::storage::{keys::storage_key_mqtt_user, rocksdb::RocksDBEngine, StorageDataWrap};
use common_base::errors::RobustMQError;
use prost::Message as _;
use protocol::placement_center::generate::mqtt::User;
use std::sync::Arc;

pub struct MQTTUserStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MQTTUserStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MQTTUserStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn list(
        &self,
        cluster_name: String,
        username: String,
    ) -> Result<Vec<StorageDataWrap>, RobustMQError> {
        if !username.is_empty() {
            let cf = self.rocksdb_engine_handler.cf_mqtt();
            let key: String = storage_key_mqtt_user(cluster_name, username);
            match self
                .rocksdb_engine_handler
                .read::<StorageDataWrap>(cf, &key)
            {
                Ok(Some(data)) => {
                    return Ok(vec![data]);
                }
                Ok(None) => {
                    return Ok(Vec::new());
                }
                Err(e) => {
                    return Err(RobustMQError::CommmonError(e));
                }
            }
        }

        
        return Ok(Vec::new());
    }

    pub fn set(&self, cluster_name: String, user: User) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_mqtt();
        let key = storage_key_mqtt_user(cluster_name, user.username.clone());
        let val = User::encode_to_vec(&user);
        let data = StorageDataWrap::new(val);
        match self.rocksdb_engine_handler.write(cf, &key, &data) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }

    pub fn delete(&self, cluster_name: String, user_name: String) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_mqtt();
        let key: String = storage_key_mqtt_user(cluster_name, user_name);
        match self.rocksdb_engine_handler.delete(cf, &key) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }
}
