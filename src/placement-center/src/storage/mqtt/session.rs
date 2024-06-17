use std::sync::Arc;

use common_base::errors::RobustMQError;

use crate::storage::{
    keys::{storage_key_mqtt_session, storage_key_mqtt_session_cluster_prefix},
    rocksdb::RocksDBEngine,
    StorageDataWrap,
};

pub struct MQTTSessionStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MQTTSessionStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MQTTSessionStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn list(
        &self,
        cluster_name: String,
        client_id: Option<String>,
    ) -> Result<Vec<StorageDataWrap>, RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_mqtt();
        if client_id != None {
            let key: String = storage_key_mqtt_session(cluster_name, client_id.unwrap());
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
        let prefix_key = storage_key_mqtt_session_cluster_prefix(cluster_name);
        let data_list = self.rocksdb_engine_handler.read_prefix(cf, &prefix_key);
        let mut results = Vec::new();
        for raw in data_list {
            for (_, v) in raw {
                match serde_json::from_slice::<StorageDataWrap>(v.as_ref()) {
                    Ok(v) => results.push(v),
                    Err(_) => {
                        continue;
                    }
                }
            }
        }
        return Ok(results);
    }

    pub fn save(
        &self,
        cluster_name: String,
        client_id: String,
        content: String,
    ) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_mqtt();
        let key = storage_key_mqtt_session(cluster_name, client_id);
        let data = StorageDataWrap::new(content);
        match self.rocksdb_engine_handler.write(cf, &key, &data) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }

    pub fn delete(&self, cluster_name: String, client_id: String) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_mqtt();
        let key: String = storage_key_mqtt_session(cluster_name, client_id);
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
    use crate::storage::mqtt::session::MQTTSessionStorage;
    use crate::storage::rocksdb::RocksDBEngine;
    use common_base::config::placement_center::PlacementCenterConfig;
    use metadata_struct::mqtt::session::MQTTSession;
    use std::sync::Arc;

    #[tokio::test]
    async fn topic_storage_test() {
        let mut config = PlacementCenterConfig::default();
        config.data_path = "/tmp/tmp_test".to_string();
        config.data_path = "/tmp/tmp_test".to_string();
        let rs = Arc::new(RocksDBEngine::new(&config));
        let session_storage = MQTTSessionStorage::new(rs);
        let cluster_name = "test_cluster".to_string();
        let client_id = "loboxu".to_string();
        let session = MQTTSession::default();
        session_storage
            .save(cluster_name.clone(), client_id, session.encode())
            .unwrap();

        let client_id = "lobo1".to_string();
        let session = MQTTSession::default();
        session_storage
            .save(cluster_name.clone(), client_id, session.encode())
            .unwrap();

        let res = session_storage.list(cluster_name.clone(), None).unwrap();
        assert_eq!(res.len(), 2);

        let res = session_storage
            .list(cluster_name.clone(), Some("lobo1".to_string()))
            .unwrap();
        assert_eq!(res.len(), 1);

        session_storage
            .delete(cluster_name.clone(), "lobo1".to_string())
            .unwrap();

        let res = session_storage
            .list(cluster_name.clone(), Some("lobo1".to_string()))
            .unwrap();
        assert_eq!(res.len(), 0);
    }
}
