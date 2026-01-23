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

use common_base::error::common::CommonError;
use metadata_struct::mqtt::session::MqttSession;
use rocksdb_engine::keys::meta::{storage_key_mqtt_session, storage_key_mqtt_session_prefix};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_data::{
    engine_delete_by_meta_data, engine_get_by_meta_data, engine_prefix_list_by_meta_data,
    engine_save_by_meta_data,
};
use std::sync::Arc;

pub struct MqttSessionStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttSessionStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttSessionStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(&self, client_id: &str, session: MqttSession) -> Result<(), CommonError> {
        let key = storage_key_mqtt_session(client_id);
        engine_save_by_meta_data(&self.rocksdb_engine_handler, &key, session)
    }

    pub fn list_all(&self) -> Result<Vec<MqttSession>, CommonError> {
        let prefix_key = storage_key_mqtt_session_prefix();
        let data = engine_prefix_list_by_meta_data::<MqttSession>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn get(&self, client_id: &str) -> Result<Option<MqttSession>, CommonError> {
        let key = storage_key_mqtt_session(client_id);
        Ok(
            engine_get_by_meta_data::<MqttSession>(&self.rocksdb_engine_handler, &key)?
                .map(|data| data.data),
        )
    }

    pub fn delete(&self, client_id: &str) -> Result<(), CommonError> {
        let key = storage_key_mqtt_session(client_id);
        engine_delete_by_meta_data(&self.rocksdb_engine_handler, &key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup_storage() -> MqttSessionStorage {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        let db = test_rocksdb_instance();
        MqttSessionStorage::new(db)
    }

    fn create_session(client_id: &str) -> MqttSession {
        MqttSession {
            client_id: client_id.to_string(),
            session_expiry_interval: 3600,
            ..Default::default()
        }
    }

    #[test]
    fn test_session_crud_operations() {
        let storage = setup_storage();

        // Test: Save and Get
        let session1 = create_session("client1");
        storage.save("client1", session1).unwrap();

        let retrieved = storage.get("client1").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().client_id, "client1");

        // Test: List multiple sessions
        let session2 = create_session("client2");
        storage.save("client2", session2).unwrap();

        let all_sessions = storage.list_all().unwrap();
        assert_eq!(all_sessions.len(), 2);

        // Test: Delete and verify
        storage.delete("client2").unwrap();
        assert!(storage.get("client2").unwrap().is_none());

        let remaining = storage.list_all().unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].client_id, "client1");
    }

    #[test]
    fn test_get_nonexistent_session() {
        let storage = setup_storage();
        let result = storage.get("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_save_overwrites_existing() {
        let storage = setup_storage();

        // Save initial session
        let session1 = create_session("client1");
        storage.save("client1", session1).unwrap();

        // Overwrite with new session
        let mut session2 = create_session("client1");
        session2.session_expiry_interval = 7200;
        storage.save("client1", session2).unwrap();

        // Verify overwrite
        let retrieved = storage.get("client1").unwrap().unwrap();
        assert_eq!(retrieved.session_expiry_interval, 7200);

        // Should still have only one session
        let all_sessions = storage.list_all().unwrap();
        assert_eq!(all_sessions.len(), 1);
    }
}
