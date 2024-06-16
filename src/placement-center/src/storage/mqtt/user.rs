use crate::storage::{
    keys::{storage_key_mqtt_user, storage_key_mqtt_user_cluster_prefix},
    rocksdb::RocksDBEngine,
    StorageDataWrap,
};
use bincode::deserialize;
use common_base::{errors::RobustMQError, log::error};
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
        username: Option<String>,
    ) -> Result<Vec<StorageDataWrap>, RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_mqtt();
        if username != None {
            let key: String = storage_key_mqtt_user(cluster_name, username.unwrap());
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
        let prefix_key = storage_key_mqtt_user_cluster_prefix(cluster_name);
        let data_list = self.rocksdb_engine_handler.read_prefix(cf, &prefix_key);
        let mut results = Vec::new();
        for raw in data_list {
            for (_, v) in raw {
                match deserialize::<StorageDataWrap>(v.as_ref()) {
                    Ok(v) => results.push(v),
                    Err(e) => {
                        println!("DDD:{}",e.to_string());
                        continue;
                    }
                }
            }
        }
        return Ok(results);
    }

    pub fn save(&self, cluster_name: String, user: User) -> Result<(), RobustMQError> {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::storage::mqtt::user::MQTTUserStorage;
    use crate::storage::rocksdb::RocksDBEngine;
    use common_base::config::placement_center::PlacementCenterConfig;
    use protocol::placement_center::generate::mqtt::User;

    #[tokio::test]
    async fn user_storage_test() {
        let mut config = PlacementCenterConfig::default();
        config.data_path = "/tmp/tmp_test".to_string();
        config.data_path = "/tmp/tmp_test".to_string();
        let rs = Arc::new(RocksDBEngine::new(&config));
        let user_storage = MQTTUserStorage::new(rs);
        let cluster_name = "test_cluster".to_string();
        let user = User {
            username: "loboxu".to_string(),
            password: "pwd123".to_string(),
            super_user: true,
        };
        user_storage.save(cluster_name.clone(), user).unwrap();

        let user = User {
            username: "lobo1".to_string(),
            password: "pwd1231".to_string(),
            super_user: true,
        };
        user_storage.save(cluster_name.clone(), user).unwrap();

        let res = user_storage.list(cluster_name.clone(), None).unwrap();
        assert_eq!(res.len(), 2);

        let res = user_storage
            .list(cluster_name.clone(), Some("lobo1".to_string()))
            .unwrap();
        assert_eq!(res.len(), 1);

        user_storage
            .delete(cluster_name.clone(), "lobo1".to_string())
            .unwrap();

        let res = user_storage
            .list(cluster_name.clone(), Some("lobo1".to_string()))
            .unwrap();
        assert_eq!(res.len(), 0);
    }
}
