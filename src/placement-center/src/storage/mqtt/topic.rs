use crate::storage::{
    keys::{storage_key_mqtt_topic, storage_key_mqtt_topic_cluster_prefix},
    rocksdb::RocksDBEngine,
    StorageDataWrap,
};
use common_base::errors::RobustMQError;
use prost::Message as _;
use protocol::placement_center::generate::mqtt::Topic;
use std::sync::Arc;

pub struct MQTTTopicStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MQTTTopicStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MQTTTopicStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn list(
        &self,
        cluster_name: String,
        topicname: Option<String>,
    ) -> Result<Vec<StorageDataWrap>, RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_mqtt();
        if topicname != None {
            let key: String = storage_key_mqtt_topic(cluster_name, topicname.unwrap());
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
        let prefix_key = storage_key_mqtt_topic_cluster_prefix(cluster_name);
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

    pub fn save(&self, cluster_name: String, topic: Topic) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_mqtt();
        let key = storage_key_mqtt_topic(cluster_name, topic.topic_name.clone());
        let val = Topic::encode_to_vec(&topic);
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

    pub fn delete(&self, cluster_name: String, topic_name: String) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_mqtt();
        let key: String = storage_key_mqtt_topic(cluster_name, topic_name);
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

    use crate::storage::mqtt::topic::MQTTTopicStorage;
    use crate::storage::rocksdb::RocksDBEngine;
    use common_base::config::placement_center::PlacementCenterConfig;
    use protocol::placement_center::generate::mqtt::Topic;

    #[tokio::test]
    async fn topic_storage_test() {
        let mut config = PlacementCenterConfig::default();
        config.data_path = "/tmp/tmp_test".to_string();
        config.data_path = "/tmp/tmp_test".to_string();
        let rs = Arc::new(RocksDBEngine::new(&config));
        let topic_storage = MQTTTopicStorage::new(rs);
        let cluster_name = "test_cluster".to_string();
        let topic = Topic {
            topic_id: "xxx".to_string(),
            topic_name: "loboxu".to_string(),
        };
        topic_storage.save(cluster_name.clone(), topic).unwrap();

        let topic = Topic {
            topic_id: "xxx".to_string(),
            topic_name: "lobo1".to_string(),
        };
        topic_storage.save(cluster_name.clone(), topic).unwrap();

        let res = topic_storage.list(cluster_name.clone(), None).unwrap();
        assert_eq!(res.len(), 2);

        let res = topic_storage
            .list(cluster_name.clone(), Some("lobo1".to_string()))
            .unwrap();
        assert_eq!(res.len(), 1);

        topic_storage
            .delete(cluster_name.clone(), "lobo1".to_string())
            .unwrap();

        let res = topic_storage
            .list(cluster_name.clone(), Some("lobo1".to_string()))
            .unwrap();
        assert_eq!(res.len(), 0);
    }
}
