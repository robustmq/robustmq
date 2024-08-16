// Copyright 2023 RobustMQ Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::storage::{
    engine::engine_save_by_cluster,
    keys::{storage_key_mqtt_topic, storage_key_mqtt_topic_cluster_prefix},
    rocksdb::RocksDBEngine,
    StorageDataWrap,
};
use common_base::errors::RobustMQError;
use metadata_struct::mqtt::topic::MQTTTopic;
use std::{sync::Arc, vec};

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
        cluster_name: &String,
        topicname: Option<String>,
    ) -> Result<Vec<StorageDataWrap>, RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_mqtt();
        if topicname != None {
            let key: String = storage_key_mqtt_topic(cluster_name, &topicname.unwrap());
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
        let prefix_key = storage_key_mqtt_topic_cluster_prefix(&cluster_name);
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
        cluster_name: &String,
        topic_name: &String,
        content: Vec<u8>,
    ) -> Result<(), RobustMQError> {
        let key = storage_key_mqtt_topic(cluster_name, topic_name);
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, content);
    }

    pub fn delete(&self, cluster_name: &String, topic_name: &String) -> Result<(), RobustMQError> {
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

    pub fn set_topic_retain_message(
        &self,
        cluster_name: &String,
        topic_name: &String,
        retain_message: Vec<u8>,
        retain_message_expired_at: u64,
    ) -> Result<(), RobustMQError> {
        let results = match self.list(cluster_name, Some(topic_name.clone())) {
            Ok(data) => data,
            Err(e) => {
                return Err(e);
            }
        };
        if results.is_empty() {
            return Err(RobustMQError::TopicDoesNotExist(topic_name.clone()));
        }

        let topic = results.get(0).unwrap();
        match serde_json::from_slice::<MQTTTopic>(&topic.data.as_slice()) {
            Ok(mut mqtt_topic) => {
                if retain_message.len() == 0 {
                    mqtt_topic.retain_message = None;
                    mqtt_topic.retain_message_expired_at = None;
                } else {
                    mqtt_topic.retain_message = Some(retain_message);
                    mqtt_topic.retain_message_expired_at = Some(retain_message_expired_at);
                }
                match self.save(cluster_name, topic_name, mqtt_topic.encode()) {
                    Ok(_) => {
                        return Ok(());
                    }
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e.to_string()));
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
    use metadata_struct::mqtt::topic::MQTTTopic;

    #[tokio::test]
    async fn topic_storage_test() {
        let mut config = PlacementCenterConfig::default();
        config.data_path = "/tmp/tmp_test".to_string();
        config.data_path = "/tmp/tmp_test".to_string();
        let rs = Arc::new(RocksDBEngine::new(&config));
        let topic_storage = MQTTTopicStorage::new(rs);
        let cluster_name = "test_cluster".to_string();
        let topic_name = "loboxu".to_string();
        let topic = MQTTTopic {
            topic_id: "xxx".to_string(),
            topic_name: topic_name.clone(),
            retain_message: None,
            retain_message_expired_at: None,
        };
        topic_storage
            .save(&cluster_name, &topic_name, topic.encode())
            .unwrap();

        let topic_name = "lobo1".to_string();
        let topic = MQTTTopic {
            topic_id: "xxx".to_string(),
            topic_name: topic_name.clone(),
            retain_message: None,
            retain_message_expired_at: None,
        };
        topic_storage
            .save(&cluster_name, &topic_name, topic.encode())
            .unwrap();

        let res = topic_storage.list(&cluster_name, None).unwrap();
        assert_eq!(res.len(), 2);

        let res = topic_storage
            .list(&cluster_name, Some("lobo1".to_string()))
            .unwrap();
        assert_eq!(res.len(), 1);

        let name = "lobo1".to_string();
        topic_storage.delete(&cluster_name, &name).unwrap();

        let res = topic_storage
            .list(&cluster_name, Some("lobo1".to_string()))
            .unwrap();
        assert_eq!(res.len(), 0);
    }
}
