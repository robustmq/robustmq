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

use crate::storage::keys::{storage_key_mqtt_connector, storage_key_mqtt_connector_prefix};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};

pub struct MqttConnectorStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttConnectorStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttConnectorStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(&self, connector_name: &str, connector: &MQTTConnector) -> Result<(), CommonError> {
        let key = storage_key_mqtt_connector(connector_name);
        engine_save_by_meta_metadata(self.rocksdb_engine_handler.clone(), &key, connector)
    }

    pub fn list(&self) -> Result<Vec<MQTTConnector>, CommonError> {
        let prefix_key = storage_key_mqtt_connector_prefix();
        let data = engine_prefix_list_by_meta_metadata::<MQTTConnector>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn get(&self, connector_name: &str) -> Result<Option<MQTTConnector>, CommonError> {
        let key = storage_key_mqtt_connector(connector_name);
        Ok(
            engine_get_by_meta_metadata::<MQTTConnector>(
                self.rocksdb_engine_handler.clone(),
                &key,
            )?
            .map(|data| data.data),
        )
    }

    pub fn delete(&self, connector_name: &str) -> Result<(), CommonError> {
        let key = storage_key_mqtt_connector(connector_name);
        engine_delete_by_meta_metadata(self.rocksdb_engine_handler.clone(), &key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup_storage() -> MqttConnectorStorage {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        MqttConnectorStorage::new(test_rocksdb_instance())
    }

    #[test]
    fn test_connector_crud() {
        let storage = setup_storage();

        // Save & Get
        let connector = MQTTConnector::default();
        storage.save("connector_a", &connector).unwrap();
        assert!(storage.get("connector_a").unwrap().is_some());

        // List
        storage
            .save("connector_b", &MQTTConnector::default())
            .unwrap();
        assert_eq!(storage.list().unwrap().len(), 2);

        // Delete & Verify
        storage.delete("connector_b").unwrap();
        assert!(storage.get("connector_b").unwrap().is_none());
        assert_eq!(storage.list().unwrap().len(), 1);
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = setup_storage();
        assert!(storage.get("nonexistent").unwrap().is_none());
    }
}
