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

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//  http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::core::error::MetaServiceError;
use crate::storage::keys::{
    storage_key_mqtt_auto_subscribe_rule, storage_key_mqtt_auto_subscribe_rule_prefix,
    storage_key_mqtt_subscribe, storage_key_mqtt_subscribe_client_id_prefix,
    storage_key_mqtt_subscribe_prefix,
};
use common_base::error::common::CommonError;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};
use std::sync::Arc;

pub struct MqttSubscribeStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttSubscribeStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttSubscribeStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(
        &self,
        client_id: &str,
        path: &str,
        subscribe: MqttSubscribe,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_subscribe(client_id, path);
        engine_save_by_meta_metadata(self.rocksdb_engine_handler.clone(), &key, subscribe)
    }

    pub fn list_all(&self) -> Result<Vec<MqttSubscribe>, CommonError> {
        let prefix_key = storage_key_mqtt_subscribe_prefix();
        let resp = engine_prefix_list_by_meta_metadata::<MqttSubscribe>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        Ok(resp.into_iter().map(|raw| raw.data).collect())
    }

    pub fn list_by_client_id(&self, client_id: &str) -> Result<Vec<MqttSubscribe>, CommonError> {
        let prefix_key = storage_key_mqtt_subscribe_client_id_prefix(client_id);
        let resp = engine_prefix_list_by_meta_metadata::<MqttSubscribe>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        Ok(resp.into_iter().map(|raw| raw.data).collect())
    }

    pub fn delete_by_client_id(&self, client_id: &str) -> Result<(), CommonError> {
        let prefix_key = storage_key_mqtt_subscribe_client_id_prefix(client_id);
        let list = engine_prefix_list_by_meta_metadata::<MqttSubscribe>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        for raw in list {
            let sub: MqttSubscribe = raw.data;
            self.delete_by_path(&sub.client_id, &sub.filter.path)?;
        }
        Ok(())
    }

    pub fn get(
        &self,
        client_id: &str,
        path: &str,
    ) -> Result<Option<MqttSubscribe>, MetaServiceError> {
        let key = storage_key_mqtt_subscribe(client_id, path);
        Ok(
            engine_get_by_meta_metadata::<MqttSubscribe>(
                self.rocksdb_engine_handler.clone(),
                &key,
            )?
            .map(|data| data.data),
        )
    }

    pub fn delete_by_path(&self, client_id: &str, path: &str) -> Result<(), CommonError> {
        let key = storage_key_mqtt_subscribe(client_id, path);
        engine_delete_by_meta_metadata(self.rocksdb_engine_handler.clone(), &key)
    }

    pub fn save_auto_subscribe_rule(
        &self,
        topic: &str,
        auto_subscribe_rule: MqttAutoSubscribeRule,
    ) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_auto_subscribe_rule(topic);
        engine_save_by_meta_metadata(
            self.rocksdb_engine_handler.clone(),
            &key,
            auto_subscribe_rule,
        )?;
        Ok(())
    }

    pub fn delete_auto_subscribe_rule(&self, topic: &str) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_auto_subscribe_rule(topic);
        engine_delete_by_meta_metadata(self.rocksdb_engine_handler.clone(), &key)?;
        Ok(())
    }

    pub fn list_all_auto_subscribe_rules(
        &self,
    ) -> Result<Vec<MqttAutoSubscribeRule>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_auto_subscribe_rule_prefix();
        let data = engine_prefix_list_by_meta_metadata::<MqttAutoSubscribeRule>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use protocol::mqtt::common::{Filter, QoS, RetainHandling};
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup_storage() -> MqttSubscribeStorage {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        MqttSubscribeStorage::new(test_rocksdb_instance())
    }

    fn create_subscribe(client_id: &str, topic: &str) -> MqttSubscribe {
        MqttSubscribe {
            client_id: client_id.to_string(),
            filter: Filter {
                path: topic.to_string(),
                qos: QoS::AtLeastOnce,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn create_auto_subscribe_rule(topic: &str) -> MqttAutoSubscribeRule {
        MqttAutoSubscribeRule {
            topic: topic.to_string(),
            qos: QoS::AtLeastOnce,
            no_local: false,
            retain_as_published: false,
            retained_handling: RetainHandling::Never,
        }
    }

    #[test]
    fn test_subscribe_crud() {
        let storage = setup_storage();
        let client = "client_a";

        // Save & Get
        let sub = create_subscribe(client, "sensor/temp");
        storage.save(client, "sensor/temp", sub.clone()).unwrap();
        assert!(storage.get(client, "sensor/temp").unwrap().is_some());

        // List by client
        storage
            .save(
                client,
                "sensor/humidity",
                create_subscribe(client, "sensor/humidity"),
            )
            .unwrap();
        assert_eq!(storage.list_by_client_id(client).unwrap().len(), 2);

        // Delete by path
        storage.delete_by_path(client, "sensor/temp").unwrap();
        assert!(storage.get(client, "sensor/temp").unwrap().is_none());
        assert_eq!(storage.list_by_client_id(client).unwrap().len(), 1);
    }

    #[test]
    fn test_list_all() {
        let storage = setup_storage();

        storage
            .save("client_a", "topic1", create_subscribe("client_a", "topic1"))
            .unwrap();
        storage
            .save("client_b", "topic2", create_subscribe("client_b", "topic2"))
            .unwrap();

        let all = storage.list_all().unwrap();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_delete_by_client_id() {
        let storage = setup_storage();
        let client = "client_a";

        // Create multiple subscriptions for one client
        storage
            .save(client, "topic1", create_subscribe(client, "topic1"))
            .unwrap();
        storage
            .save(client, "topic2", create_subscribe(client, "topic2"))
            .unwrap();
        assert_eq!(storage.list_by_client_id(client).unwrap().len(), 2);

        // Delete all subscriptions for the client
        storage.delete_by_client_id(client).unwrap();
        assert!(storage.list_by_client_id(client).unwrap().is_empty());
    }

    #[test]
    fn test_auto_subscribe_rule() {
        let storage = setup_storage();
        let topic = "devices/#";

        // Save rule
        let rule = create_auto_subscribe_rule(topic);
        storage
            .save_auto_subscribe_rule(topic, rule.clone())
            .unwrap();

        // List rules
        let rules = storage.list_all_auto_subscribe_rules().unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].topic, topic);

        // Delete rule
        storage.delete_auto_subscribe_rule(topic).unwrap();
        assert!(storage.list_all_auto_subscribe_rules().unwrap().is_empty());
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = setup_storage();
        assert!(storage.get("client1", "topic1").unwrap().is_none());
    }
}
