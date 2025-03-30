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
use metadata_struct::mqtt::bridge::connector::MQTTConnector;

use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{storage_key_mqtt_connector, storage_key_mqtt_connector_prefix};
use crate::storage::rocksdb::RocksDBEngine;

pub struct MqttConnectorStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttConnectorStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttConnectorStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(
        &self,
        cluster_name: &str,
        connector_name: &str,
        connector: &MQTTConnector,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_connector(cluster_name, connector_name);
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, connector)
    }

    pub fn list(&self, cluster_name: &str) -> Result<Vec<MQTTConnector>, CommonError> {
        let prefix_key = storage_key_mqtt_connector_prefix(cluster_name);
        let mut results = Vec::new();
        for raw in engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)? {
            if let Ok(data) = serde_json::from_str::<MQTTConnector>(&raw.data) {
                results.push(data);
            }
        }
        Ok(results)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        connector_name: &str,
    ) -> Result<Option<MQTTConnector>, CommonError> {
        let key = storage_key_mqtt_connector(cluster_name, connector_name);
        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)? {
            return Ok(Some(serde_json::from_str::<MQTTConnector>(&data.data)?));
        }
        Ok(None)
    }

    pub fn delete(&self, cluster_name: &str, connector_name: &str) -> Result<(), CommonError> {
        let key = storage_key_mqtt_connector(cluster_name, connector_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use common_base::utils::file_utils::test_temp_dir;
    use metadata_struct::mqtt::bridge::connector::MQTTConnector;

    use crate::storage::mqtt::connector::MqttConnectorStorage;
    use crate::storage::rocksdb::{column_family_list, RocksDBEngine};

    #[tokio::test]
    async fn connector_storage_test() {
        let config = placement_center_test_conf();
        let rs = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let connector_storage = MqttConnectorStorage::new(rs);
        let cluster_name = "test_cluster".to_string();
        let connector_id = "loboxu".to_string();
        let connector = MQTTConnector::default();
        connector_storage
            .save(&cluster_name, &connector_id, &connector)
            .unwrap();

        let connector_id = "lobo1".to_string();
        let connector = MQTTConnector::default();
        connector_storage
            .save(&cluster_name, &connector_id, &connector)
            .unwrap();

        let res = connector_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 2);

        let res = connector_storage.get(&cluster_name, "lobo1").unwrap();
        assert!(res.is_some());

        connector_storage.delete(&cluster_name, "lobo1").unwrap();

        let res = connector_storage.get(&cluster_name, "lobo1").unwrap();
        assert!(res.is_none());
    }
}
