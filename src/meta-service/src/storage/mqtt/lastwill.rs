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

use metadata_struct::mqtt::lastwill::MqttLastWillData;
use rocksdb_engine::storage::meta_data::{
    engine_delete_by_meta_data, engine_get_by_meta_data, engine_save_by_meta_data,
};

use crate::core::error::MetaServiceError;
use crate::storage::keys::storage_key_mqtt_last_will;
use rocksdb_engine::rocksdb::RocksDBEngine;

pub struct MqttLastWillStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttLastWillStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttLastWillStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(
        &self,
        cluster_name: &str,
        client_id: &str,
        last_will_message: MqttLastWillData,
    ) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        Ok(engine_save_by_meta_data(
            self.rocksdb_engine_handler.clone(),
            &key,
            last_will_message,
        )?)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        client_id: &str,
    ) -> Result<Option<MqttLastWillData>, MetaServiceError> {
        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        Ok(
            engine_get_by_meta_data::<MqttLastWillData>(self.rocksdb_engine_handler.clone(), &key)?
                .map(|data| data.data),
        )
    }

    pub fn delete(&self, cluster_name: &str, client_id: &str) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        Ok(engine_delete_by_meta_data(
            self.rocksdb_engine_handler.clone(),
            &key,
        )?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup_storage() -> MqttLastWillStorage {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        MqttLastWillStorage::new(test_rocksdb_instance())
    }

    fn create_last_will(client_id: &str, topic: &str) -> MqttLastWillData {
        use bytes::Bytes;
        use protocol::mqtt::common::{LastWill, QoS};

        MqttLastWillData {
            client_id: client_id.to_string(),
            last_will: Some(LastWill {
                topic: Bytes::from(topic.to_string()),
                message: Bytes::from("goodbye"),
                qos: QoS::AtLeastOnce,
                retain: false,
            }),
            last_will_properties: None,
        }
    }

    #[test]
    fn test_lastwill_crud() {
        let storage = setup_storage();
        let cluster = "test_cluster";

        // Save & Get
        let lastwill = create_last_will("client_a", "status/client_a");
        storage.save(cluster, "client_a", lastwill).unwrap();

        let retrieved = storage.get(cluster, "client_a").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().client_id, "client_a");

        // Delete & Verify
        storage.delete(cluster, "client_a").unwrap();
        assert!(storage.get(cluster, "client_a").unwrap().is_none());
    }

    #[test]
    fn test_update_lastwill() {
        let storage = setup_storage();
        let cluster = "test_cluster";
        let client = "client_a";

        // Save initial
        storage
            .save(cluster, client, create_last_will(client, "topic1"))
            .unwrap();
        let initial = storage.get(cluster, client).unwrap().unwrap();
        assert_eq!(
            initial.last_will.as_ref().unwrap().topic.as_ref(),
            b"topic1"
        );

        // Update
        storage
            .save(cluster, client, create_last_will(client, "topic2"))
            .unwrap();
        let updated = storage.get(cluster, client).unwrap().unwrap();
        assert_eq!(
            updated.last_will.as_ref().unwrap().topic.as_ref(),
            b"topic2"
        );
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = setup_storage();
        assert!(storage.get("cluster1", "nonexistent").unwrap().is_none());
    }
}
