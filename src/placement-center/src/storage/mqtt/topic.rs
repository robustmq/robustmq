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

use common_base::utils::topic_util::topic_name_regex_match;
use metadata_struct::mqtt::topic::MqttTopic;
use prost::Message;
use protocol::placement_center::placement_center_mqtt::SetExclusiveTopicRequest;

use crate::core::error::PlacementCenterError;
use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{
    storage_key_mqtt_exclusive_topic_name, storage_key_mqtt_exclusive_topic_prefix,
    storage_key_mqtt_topic, storage_key_mqtt_topic_cluster_prefix,
};
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
        cluster_name: &str,
        topic_name: &str,
        topic: MqttTopic,
    ) -> Result<(), PlacementCenterError> {
        let key = storage_key_mqtt_topic(cluster_name, topic_name);
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, topic)?;
        Ok(())
    }

    pub fn list(&self, cluster_name: &str) -> Result<Vec<MqttTopic>, PlacementCenterError> {
        let prefix_key = storage_key_mqtt_topic_cluster_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            let topic = serde_json::from_slice::<MqttTopic>(&raw.data)?;
            results.push(topic);
        }
        Ok(results)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        topicname: &str,
    ) -> Result<Option<MqttTopic>, PlacementCenterError> {
        let key: String = storage_key_mqtt_topic(cluster_name, topicname);

        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)? {
            let topic = serde_json::from_slice::<MqttTopic>(&data.data)?;
            return Ok(Some(topic));
        }
        Ok(None)
    }

    pub fn delete(&self, cluster_name: &str, topic_name: &str) -> Result<(), PlacementCenterError> {
        let key: String = storage_key_mqtt_topic(cluster_name, topic_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)?;
        Ok(())
    }

    pub fn set_nx_exclisve_topic(
        &self,
        cluster_name: &str,
        topic_name: &str,
        value: Vec<u8>,
    ) -> Result<bool, PlacementCenterError> {
        if !self.can_add_exclisve_topic(cluster_name, topic_name)? {
            return Ok(false);
        }
        self.set_exclisve_topic(cluster_name, topic_name, value)?;
        Ok(true)
    }

    pub fn delete_exclisve_topic(
        &self,
        cluster_name: &str,
        topic_name: &str,
    ) -> Result<(), PlacementCenterError> {
        let key_name = storage_key_mqtt_exclusive_topic_name(cluster_name, topic_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key_name)?;
        Ok(())
    }

    fn can_add_exclisve_topic(
        &self,
        cluster_name: &str,
        topic_name: &str,
    ) -> Result<bool, PlacementCenterError> {
        let prefix_key_name = storage_key_mqtt_exclusive_topic_prefix(cluster_name);
        let exclusive_topic_name_list =
            engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key_name)?;
        for wrap in &exclusive_topic_name_list {
            let req = SetExclusiveTopicRequest::decode(wrap.data.as_ref())?;
            if topic_name_regex_match(&req.topic_name, topic_name) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn set_exclisve_topic(
        &self,
        cluster_name: &str,
        topic_name: &str,
        value: Vec<u8>,
    ) -> Result<(), PlacementCenterError> {
        let key_name = storage_key_mqtt_exclusive_topic_name(cluster_name, topic_name);
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key_name, value)?;
        Ok(())
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
            cluster_name: cluster_name.clone(),
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
            cluster_name: cluster_name.to_string(),
            topic_name: topic_name.clone(),
            retain_message: None,
            retain_message_expired_at: None,
        };
        topic_storage
            .save(&cluster_name, &topic_name, topic)
            .unwrap();

        let res = topic_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 2);

        let res = topic_storage.get(&cluster_name, "lobo1").unwrap();
        assert!(res.is_some());

        let name = "lobo1".to_string();
        topic_storage.delete(&cluster_name, &name).unwrap();

        let res = topic_storage.get(&cluster_name, "lobo1").unwrap();
        assert!(res.is_none());

        remove_dir_all(config.rocksdb.data_path).await.unwrap();
    }
}
