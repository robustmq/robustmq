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
    storage_key_mqtt_subscribe_cluster_prefix
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
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, auto_subscribe_rule)?;
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
            let topic = serde_json::from_slice::<MqttAutoSubscribeRule>(&raw.data)?;
            results.push(topic);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn subscribe_storage_test() {}
}
