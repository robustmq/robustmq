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

use crate::core::error::MetaServiceError;
use crate::storage::keys::{
    storage_key_mqtt_retain_message, storage_key_mqtt_retain_message_prefix,
    storage_key_mqtt_topic, storage_key_mqtt_topic_cluster_prefix,
    storage_key_mqtt_topic_rewrite_rule, storage_key_mqtt_topic_rewrite_rule_prefix,
};
use metadata_struct::mqtt::retain_message::MQTTRetainMessage;
use metadata_struct::mqtt::topic::MQTTTopic;
use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_data::{
    engine_delete_by_meta_data, engine_get_by_meta_data, engine_prefix_list_by_meta_data,
    engine_save_by_meta_data,
};
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_prefix_list_by_meta_metadata,
    engine_save_by_meta_metadata,
};
use std::sync::Arc;

pub struct MqttTopicStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttTopicStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttTopicStorage {
            rocksdb_engine_handler,
        }
    }

    // Topic
    pub fn save(&self, topic_name: &str, topic: MQTTTopic) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_topic(topic_name);
        engine_save_by_meta_data(self.rocksdb_engine_handler.clone(), &key, topic)?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<MQTTTopic>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_topic_cluster_prefix();
        let data = engine_prefix_list_by_meta_data::<MQTTTopic>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn get(&self, topic_name: &str) -> Result<Option<MQTTTopic>, MetaServiceError> {
        let key = storage_key_mqtt_topic(topic_name);
        Ok(
            engine_get_by_meta_data::<MQTTTopic>(self.rocksdb_engine_handler.clone(), &key)?
                .map(|data| data.data),
        )
    }

    pub fn delete(&self, topic_name: &str) -> Result<(), MetaServiceError> {
        let key: String = storage_key_mqtt_topic(topic_name);
        engine_delete_by_meta_data(self.rocksdb_engine_handler.clone(), &key)?;
        Ok(())
    }

    // Rewrite Rule
    pub fn save_topic_rewrite_rule(
        &self,
        action: &str,
        source_topic: &str,
        topic_rewrite_rule: MqttTopicRewriteRule,
    ) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_topic_rewrite_rule(action, source_topic);
        engine_save_by_meta_metadata(
            self.rocksdb_engine_handler.clone(),
            &key,
            topic_rewrite_rule,
        )?;
        Ok(())
    }

    pub fn delete_topic_rewrite_rule(
        &self,
        action: &str,
        source_topic: &str,
    ) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_topic_rewrite_rule(action, source_topic);
        engine_delete_by_meta_metadata(self.rocksdb_engine_handler.clone(), &key)?;
        Ok(())
    }

    pub fn list_all_topic_rewrite_rules(
        &self,
    ) -> Result<Vec<MqttTopicRewriteRule>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_topic_rewrite_rule_prefix();
        let data = engine_prefix_list_by_meta_metadata::<MqttTopicRewriteRule>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    // Retain Message
    pub fn save_retain_message(
        &self,
        retain_message: MQTTRetainMessage,
    ) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_retain_message(&retain_message.topic_name);
        engine_save_by_meta_data(self.rocksdb_engine_handler.clone(), &key, retain_message)?;
        Ok(())
    }

    pub fn delete_retain_message(&self, topic_name: &str) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_retain_message(topic_name);
        engine_delete_by_meta_data(self.rocksdb_engine_handler.clone(), &key)?;
        Ok(())
    }

    pub fn get_retain_message(
        &self,
        topic_name: &str,
    ) -> Result<Option<MQTTRetainMessage>, MetaServiceError> {
        let key = storage_key_mqtt_retain_message(topic_name);
        Ok(
            engine_get_by_meta_data::<MQTTRetainMessage>(
                self.rocksdb_engine_handler.clone(),
                &key,
            )?
            .map(|data| data.data),
        )
    }

    pub fn list_all_retain_messages(&self) -> Result<Vec<MQTTRetainMessage>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_retain_message_prefix();
        let data = engine_prefix_list_by_meta_data::<MQTTRetainMessage>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::tools::now_second;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use metadata_struct::mqtt::retain_message::MQTTRetainMessage;
    use metadata_struct::mqtt::topic_rewrite_rule::MqttTopicRewriteRule;
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup_storage() -> MqttTopicStorage {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        MqttTopicStorage::new(test_rocksdb_instance())
    }

    fn create_topic(topic_name: &str) -> MQTTTopic {
        MQTTTopic {
            topic_name: topic_name.to_string(),
            create_time: now_second(),
        }
    }

    fn create_rewrite_rule(action: &str, source: &str, dest: &str) -> MqttTopicRewriteRule {
        MqttTopicRewriteRule {
            action: action.to_string(),
            source_topic: source.to_string(),
            dest_topic: dest.to_string(),
            regex: String::new(),
            timestamp: now_second() as u128,
        }
    }

    fn create_retain_message(topic: &str, message: &[u8]) -> MQTTRetainMessage {
        use bytes::Bytes;
        MQTTRetainMessage {
            topic_name: topic.to_string(),
            retain_message: Bytes::from(message.to_vec()),
            retain_message_expired_at: now_second() + 3600,
            create_time: now_second(),
        }
    }

    #[test]
    fn test_topic_crud() {
        let storage = setup_storage();

        // Save & Get
        storage
            .save("sensor/temp", create_topic("sensor/temp"))
            .unwrap();
        assert!(storage.get("sensor/temp").unwrap().is_some());

        // List
        storage
            .save("sensor/humidity", create_topic("sensor/humidity"))
            .unwrap();
        assert_eq!(storage.list().unwrap().len(), 2);

        // Delete & Verify
        storage.delete("sensor/humidity").unwrap();
        assert!(storage.get("sensor/humidity").unwrap().is_none());
        assert_eq!(storage.list().unwrap().len(), 1);
    }

    #[test]
    fn test_topic_rewrite_rule() {
        let storage = setup_storage();

        // Save rule
        let rule = create_rewrite_rule("subscribe", "old/+/topic", "new/+/topic");
        storage
            .save_topic_rewrite_rule("subscribe", "old/+/topic", rule)
            .unwrap();

        // List rules
        let rules = storage.list_all_topic_rewrite_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].source_topic, "old/+/topic");

        // Delete rule
        storage
            .delete_topic_rewrite_rule("subscribe", "old/+/topic")
            .unwrap();
        assert_eq!(storage.list_all_topic_rewrite_rules().unwrap().len(), 0);
    }

    #[test]
    fn test_retain_message() {
        let storage = setup_storage();
        let topic = "sensor/data";

        // Save retain message
        let msg = create_retain_message(topic, b"temperature:25");
        storage.save_retain_message(msg).unwrap();

        // Get message
        let retrieved = storage.get_retain_message(topic).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(
            retrieved.unwrap().retain_message.as_ref(),
            b"temperature:25"
        );

        // Delete message
        storage.delete_retain_message(topic).unwrap();
        assert!(storage.get_retain_message(topic).unwrap().is_none());
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = setup_storage();
        assert!(storage.get("nonexistent").unwrap().is_none());
        assert!(storage
            .get_retain_message("nonexistent")
            .unwrap()
            .is_none());
    }
}
