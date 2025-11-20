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
use metadata_struct::mqtt::session::MqttSession;
use rocksdb_engine::storage::meta_data::{
    engine_delete_by_meta_data, engine_get_by_meta_data, engine_prefix_list_by_meta_data,
    engine_save_by_meta_data,
};

use crate::storage::keys::{storage_key_mqtt_session, storage_key_mqtt_session_cluster_prefix};
use rocksdb_engine::rocksdb::RocksDBEngine;

pub struct MqttSessionStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttSessionStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttSessionStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(
        &self,
        cluster_name: &str,
        client_id: &str,
        session: MqttSession,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_session(cluster_name, client_id);
        engine_save_by_meta_data(self.rocksdb_engine_handler.clone(), &key, session)
    }

    pub fn list(&self, cluster_name: &str) -> Result<Vec<MqttSession>, CommonError> {
        let prefix_key = storage_key_mqtt_session_cluster_prefix(cluster_name);
        let data = engine_prefix_list_by_meta_data::<MqttSession>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn get(
        &self,
        cluster_name: &str,
        client_id: &str,
    ) -> Result<Option<MqttSession>, CommonError> {
        let key = storage_key_mqtt_session(cluster_name, client_id);
        Ok(
            engine_get_by_meta_data::<MqttSession>(self.rocksdb_engine_handler.clone(), &key)?
                .map(|data| data.data),
        )
    }

    pub fn delete(&self, cluster_name: &str, client_id: &str) -> Result<(), CommonError> {
        let key = storage_key_mqtt_session(cluster_name, client_id);
        engine_delete_by_meta_data(self.rocksdb_engine_handler.clone(), &key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::tools::now_second;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup_storage() -> MqttSessionStorage {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        MqttSessionStorage::new(test_rocksdb_instance())
    }

    fn create_session(client_id: &str, connection_id: u64) -> MqttSession {
        MqttSession {
            client_id: client_id.to_string(),
            session_expiry: 3600,
            is_contain_last_will: false,
            last_will_delay_interval: None,
            create_time: now_second(),
            connection_id: Some(connection_id),
            broker_id: Some(1),
            reconnect_time: None,
            distinct_time: None,
        }
    }

    #[test]
    fn test_session_crud() {
        let storage = setup_storage();
        let cluster = "test_cluster";

        // Save & Get
        let session = create_session("client_a", 100);
        storage.save(cluster, "client_a", session).unwrap();
        assert!(storage.get(cluster, "client_a").unwrap().is_some());

        // List
        storage
            .save(cluster, "client_b", create_session("client_b", 101))
            .unwrap();
        assert_eq!(storage.list(cluster).unwrap().len(), 2);

        // Delete & Verify
        storage.delete(cluster, "client_b").unwrap();
        assert!(storage.get(cluster, "client_b").unwrap().is_none());
        assert_eq!(storage.list(cluster).unwrap().len(), 1);
    }

    #[test]
    fn test_update_session() {
        let storage = setup_storage();
        let cluster = "test_cluster";
        let client = "client_a";

        // Save initial session
        storage
            .save(cluster, client, create_session(client, 100))
            .unwrap();
        let initial = storage.get(cluster, client).unwrap().unwrap();
        assert_eq!(initial.connection_id, Some(100));

        // Update session
        storage
            .save(cluster, client, create_session(client, 200))
            .unwrap();
        let updated = storage.get(cluster, client).unwrap().unwrap();
        assert_eq!(updated.connection_id, Some(200));
    }

    #[test]
    fn test_get_nonexistent() {
        let storage = setup_storage();
        assert!(storage.get("cluster1", "nonexistent").unwrap().is_none());
    }
}
