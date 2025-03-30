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

use std::sync::Arc;

use common_base::error::common::CommonError;
use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
use metadata_struct::mqtt::subscribe_data::MqttSubscribe;

use crate::core::error::PlacementCenterError;
use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{
    storage_key_mqtt_auto_subscribe_rule, storage_key_mqtt_auto_subscribe_rule_prefix,
    storage_key_mqtt_subscribe, storage_key_mqtt_subscribe_client_id_prefix,
    storage_key_mqtt_subscribe_cluster_prefix,
};
use crate::storage::rocksdb::RocksDBEngine;

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
        cluster_name: &str,
        client_id: &str,
        path: &str,
        subscribe: MqttSubscribe,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_subscribe(cluster_name, client_id, path);
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, subscribe)
    }

    pub fn list_by_cluster(&self, cluster_name: &str) -> Result<Vec<MqttSubscribe>, CommonError> {
        let prefix_key = storage_key_mqtt_subscribe_cluster_prefix(cluster_name);
        let resp = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in resp {
            let topic = serde_json::from_str::<MqttSubscribe>(&raw.data)?;
            results.push(topic);
        }
        Ok(results)
    }

    pub fn list_by_client_id(
        &self,
        cluster_name: &str,
        client_id: &str,
    ) -> Result<Vec<MqttSubscribe>, CommonError> {
        let prefix_key = storage_key_mqtt_subscribe_client_id_prefix(cluster_name, client_id);
        let resp = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in resp {
            let topic = serde_json::from_str::<MqttSubscribe>(&raw.data)?;
            results.push(topic);
        }
        Ok(results)
    }

    pub fn delete_by_client_id(
        &self,
        cluster_name: &str,
        client_id: &str,
    ) -> Result<(), CommonError> {
        let prefix_key = storage_key_mqtt_subscribe_client_id_prefix(cluster_name, client_id);
        let list = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        for raw in list {
            let sub = serde_json::from_str::<MqttSubscribe>(&raw.data)?;
            self.delete_by_path(&sub.cluster_name, &sub.client_id, &sub.filter.path)?;
        }
        Ok(())
    }

    pub fn get(
        &self,
        cluster_name: &str,
        client_id: &str,
        path: &str,
    ) -> Result<Option<MqttSubscribe>, PlacementCenterError> {
        let key: String = storage_key_mqtt_subscribe(cluster_name, client_id, path);

        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)? {
            let subscribe = serde_json::from_str::<MqttSubscribe>(&data.data)?;
            return Ok(Some(subscribe));
        }
        Ok(None)
    }

    pub fn delete_by_path(
        &self,
        cluster_name: &str,
        client_id: &str,
        path: &str,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_subscribe(cluster_name, client_id, path);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }

    pub fn save_auto_subscribe_rule(
        &self,
        cluster_name: &str,
        topic: &str,
        auto_subscribe_rule: MqttAutoSubscribeRule,
    ) -> Result<(), PlacementCenterError> {
        let key = storage_key_mqtt_auto_subscribe_rule(cluster_name, topic);
        engine_save_by_cluster(
            self.rocksdb_engine_handler.clone(),
            key,
            auto_subscribe_rule,
        )?;
        Ok(())
    }

    pub fn delete_auto_subscribe_rule(
        &self,
        cluster_name: &str,
        topic: &str,
    ) -> Result<(), PlacementCenterError> {
        let key = storage_key_mqtt_auto_subscribe_rule(cluster_name, topic);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)?;
        Ok(())
    }

    pub fn list_auto_subscribe_rule(
        &self,
        cluster_name: &str,
    ) -> Result<Vec<MqttAutoSubscribeRule>, PlacementCenterError> {
        let prefix_key = storage_key_mqtt_auto_subscribe_rule_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            let topic = serde_json::from_str::<MqttAutoSubscribeRule>(&raw.data)?;
            results.push(topic);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::mqtt::subscribe::MqttSubscribeStorage;
    use crate::storage::rocksdb::column_family_list;
    use common_base::config::placement_center::placement_center_test_conf;
    use common_base::utils::file_utils::test_temp_dir;
    use metadata_struct::mqtt::auto_subscribe_rule::MqttAutoSubscribeRule;
    use metadata_struct::mqtt::subscribe_data::MqttSubscribe;
    use protocol::mqtt::common::{Filter, QoS, RetainForwardRule};
    use rocksdb_engine::RocksDBEngine;
    use std::sync::Arc;

    #[tokio::test]
    async fn subscribe_storage_ops() {
        let config = placement_center_test_conf();
        let db = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let storage = MqttSubscribeStorage::new(db);
        let cluster = "test_cluster".to_string();
        let client1 = "client_a".to_string();
        let client2 = "client_b".to_string();
        let topic1 = "topic_a".to_string();
        let topic2 = "topic_b".to_string();

        assert!(storage
            .list_by_client_id(&cluster, &client1)
            .unwrap()
            .is_empty());

        let sub1 = MqttSubscribe {
            cluster_name: cluster.clone(),
            client_id: client1.clone(),
            filter: Filter {
                path: topic1.clone(),
                ..Default::default()
            },
            ..Default::default()
        };
        storage
            .save(&cluster, &client1, &topic1, sub1.clone())
            .unwrap();

        let sub2 = MqttSubscribe {
            cluster_name: cluster.clone(),
            client_id: client1.clone(),
            filter: Filter {
                path: topic2.clone(),
                ..Default::default()
            },
            ..Default::default()
        };
        storage
            .save(&cluster, &client1, &topic2, sub2.clone())
            .unwrap();

        let all_subs = storage.list_by_cluster(&cluster).unwrap();
        assert_eq!(all_subs.len(), 2);

        let client_subs = storage.list_by_client_id(&cluster, &client1).unwrap();
        assert_eq!(client_subs, vec![sub1.clone(), sub2.clone()]);

        let queried = storage.get(&cluster, &client1, &topic1).unwrap();
        assert_eq!(queried, Some(sub1.clone()));

        storage.delete_by_path(&cluster, &client1, &topic1).unwrap();
        assert!(storage.get(&cluster, &client1, &topic1).unwrap().is_none());
        assert!(storage.get(&cluster, &client1, &topic2).unwrap().is_some());

        storage
            .save(&cluster, &client2, &topic1, sub1.clone())
            .unwrap();
        storage.delete_by_client_id(&cluster, &client1).unwrap();

        assert!(storage.get(&cluster, &client1, &topic2).unwrap().is_none());

        assert!(storage
            .list_by_client_id(&cluster, &client1)
            .unwrap()
            .is_empty());

        storage.delete_by_path(&cluster, &client2, &topic1).unwrap();
        assert!(storage
            .list_by_client_id(&cluster, &client2)
            .unwrap()
            .is_empty());
    }

    #[tokio::test]
    async fn auto_subscribe_rule_storage_basic_ops() {
        let config = placement_center_test_conf();
        let db = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));

        let storage = MqttSubscribeStorage::new(db);

        let cluster = "test_cluster";
        let topic = "devices/temperature";

        let rule = MqttAutoSubscribeRule {
            cluster: cluster.to_string(),
            topic: topic.to_string(),
            qos: QoS::AtLeastOnce,
            no_local: true,
            retain_as_published: false,
            retained_handling: RetainForwardRule::OnEverySubscribe,
        };

        storage
            .save_auto_subscribe_rule(cluster, topic, rule.clone())
            .unwrap();

        let rules = storage.list_auto_subscribe_rule(cluster).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0], rule);

        storage.delete_auto_subscribe_rule(cluster, topic).unwrap();

        let rules_after_delete = storage.list_auto_subscribe_rule(cluster).unwrap();
        assert!(rules_after_delete.is_empty());
    }
}
