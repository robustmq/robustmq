// Copyright 2023 RobustMQ Team
//
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

use std::sync::Arc;

use common_base::error::common::CommonError;
use common_base::error::mqtt_broker::MQTTBrokerError;
use metadata_struct::mqtt::topic::MqttTopic;

use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{storage_key_mqtt_topic, storage_key_mqtt_topic_cluster_prefix};
use crate::storage::rocksdb::RocksDBEngine;

pub struct MqttTopicStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttTopicStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttTopicStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(
        &self,
        cluster_name: &String,
        topic_name: &String,
        topic: MqttTopic,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_topic(cluster_name, topic_name);
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, topic)
    }

    pub fn list(&self, cluster_name: &String) -> Result<Vec<MqttTopic>, CommonError> {
        let prefix_key = storage_key_mqtt_topic_cluster_prefix(cluster_name);
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<MqttTopic>(&raw.data) {
                        Ok(topic) => {
                            results.push(topic);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                Ok(results)
            }
            Err(e) => Err(e),
        }
    }

    pub fn get(
        &self,
        cluster_name: &String,
        topicname: &String,
    ) -> Result<Option<MqttTopic>, CommonError> {
        let key: String = storage_key_mqtt_topic(cluster_name, topicname);
        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key) {
            Ok(Some(data)) => match serde_json::from_slice::<MqttTopic>(&data.data) {
                Ok(lastwill) => Ok(Some(lastwill)),
                Err(e) => Err(e.into()),
            },
            Ok(None) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn delete(&self, cluster_name: &String, topic_name: &String) -> Result<(), CommonError> {
        let key: String = storage_key_mqtt_topic(cluster_name, topic_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }

    pub fn set_topic_retain_message(
        &self,
        cluster_name: &String,
        topic_name: &String,
        retain_message: Vec<u8>,
        retain_message_expired_at: u64,
    ) -> Result<(), CommonError> {
        let mut topic = match self.get(cluster_name, topic_name) {
            Ok(Some(data)) => data,
            Ok(None) => {
                return Err(MQTTBrokerError::TopicDoesNotExist(topic_name.clone()).into());
            }
            Err(e) => {
                return Err(e);
            }
        };

        if retain_message.is_empty() {
            topic.retain_message = None;
            topic.retain_message_expired_at = None;
        } else {
            topic.retain_message = Some(retain_message);
            topic.retain_message_expired_at = Some(retain_message_expired_at);
        }
        self.save(cluster_name, topic_name, topic)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use metadata_struct::mqtt::topic::MqttTopic;
    use tokio::fs::remove_dir_all;

    use crate::storage::mqtt::topic::MqttTopicStorage;
    use crate::storage::rocksdb::{column_family_list, RocksDBEngine};

    #[tokio::test]
    async fn topic_storage_test() {
        let config = placement_center_test_conf();

        let rs = Arc::new(RocksDBEngine::new(
            config.rocksdb.data_path.as_str(),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let topic_storage = MqttTopicStorage::new(rs);
        let cluster_name = "test_cluster".to_string();
        let topic_name = "loboxu".to_string();
        let topic = MqttTopic {
            topic_id: "xxx".to_string(),
            topic_name: topic_name.clone(),
            retain_message: None,
            retain_message_expired_at: None,
        };
        topic_storage
            .save(&cluster_name, &topic_name, topic)
            .unwrap();

        let topic_name = "lobo1".to_string();
        let topic = MqttTopic {
            topic_id: "xxx".to_string(),
            topic_name: topic_name.clone(),
            retain_message: None,
            retain_message_expired_at: None,
        };
        topic_storage
            .save(&cluster_name, &topic_name, topic)
            .unwrap();

        let res = topic_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 2);

        let res = topic_storage
            .get(&cluster_name, &"lobo1".to_string())
            .unwrap();
        assert!(res.is_some());

        let name = "lobo1".to_string();
        topic_storage.delete(&cluster_name, &name).unwrap();

        let res = topic_storage
            .get(&cluster_name, &"lobo1".to_string())
            .unwrap();
        assert!(res.is_none());

        remove_dir_all(config.rocksdb.data_path).await.unwrap();
    }
}
