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

use crate::core::error::MetaServiceError;
use crate::storage::engine_meta::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_meta,
};
use crate::storage::keys::{
    storage_key_mqtt_topic, storage_key_mqtt_topic_cluster_prefix,
    storage_key_mqtt_topic_rewrite_rule, storage_key_mqtt_topic_rewrite_rule_prefix,
};
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use rocksdb_engine::RocksDBEngine;

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
        topic: MQTTTopic,
    ) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_topic(cluster_name, topic_name);
        engine_save_by_meta(self.rocksdb_engine_handler.clone(), key, topic)?;
        Ok(())
    }

    pub fn list(&self, cluster_name: &str) -> Result<Vec<MQTTTopic>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_topic_cluster_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            let topic = serde_json::from_str::<MQTTTopic>(&raw.data)?;
            results.push(topic);
        }
        Ok(results)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        topicname: &str,
    ) -> Result<Option<MQTTTopic>, MetaServiceError> {
        let key: String = storage_key_mqtt_topic(cluster_name, topicname);

        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)? {
            let topic = serde_json::from_str::<MQTTTopic>(&data.data)?;
            return Ok(Some(topic));
        }
        Ok(None)
    }

    pub fn delete(&self, cluster_name: &str, topic_name: &str) -> Result<(), MetaServiceError> {
        let key: String = storage_key_mqtt_topic(cluster_name, topic_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)?;
        Ok(())
    }

    pub fn save_topic_rewrite_rule(
        &self,
        cluster_name: &str,
        action: &str,
        source_topic: &str,
        topic_rewrite_rule: MqttTopicRewriteRule,
    ) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_topic_rewrite_rule(cluster_name, action, source_topic);
        engine_save_by_meta(self.rocksdb_engine_handler.clone(), key, topic_rewrite_rule)?;
        Ok(())
    }

    pub fn delete_topic_rewrite_rule(
        &self,
        cluster_name: &str,
        action: &str,
        source_topic: &str,
    ) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_topic_rewrite_rule(cluster_name, action, source_topic);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)?;
        Ok(())
    }

    pub fn list_topic_rewrite_rule(
        &self,
        cluster_name: &str,
    ) -> Result<Vec<MqttTopicRewriteRule>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_topic_rewrite_rule_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            let topic = serde_json::from_str::<MqttTopicRewriteRule>(&raw.data)?;
            results.push(topic);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::storage::mqtt::topic::MqttTopicStorage;
    use broker_core::rocksdb::column_family_list;
    use common_base::tools::now_second;
    use common_base::utils::file_utils::test_temp_dir;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use metadata_struct::mqtt::topic::MQTTTopic;
    use rocksdb_engine::RocksDBEngine;

    #[tokio::test]
    async fn topic_storage_test() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());

        let rs = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));
        let topic_storage = MqttTopicStorage::new(rs);
        let cluster_name = "test_cluster".to_string();
        let topic_name = "loboxu".to_string();
        let topic = MQTTTopic {
            topic_id: "xxx".to_string(),
            cluster_name: cluster_name.clone(),
            topic_name: topic_name.clone(),
            retain_message: None,
            retain_message_expired_at: None,
            create_time: now_second(),
        };
        topic_storage
            .save(&cluster_name, &topic_name, topic)
            .unwrap();

        let topic_name = "lobo1".to_string();
        let topic = MQTTTopic {
            topic_id: "xxx".to_string(),
            cluster_name: cluster_name.to_string(),
            topic_name: topic_name.clone(),
            retain_message: None,
            retain_message_expired_at: None,
            create_time: now_second(),
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
    }
}
