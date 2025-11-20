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

use common_base::error::common::CommonError;
use metadata_struct::mqtt::user::MqttUser;

use crate::storage::keys::{storage_key_mqtt_user, storage_key_mqtt_user_cluster_prefix};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};

pub struct MqttUserStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttUserStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttUserStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(
        &self,
        cluster_name: &str,
        user_name: &str,
        user: MqttUser,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_user(cluster_name, user_name);
        engine_save_by_meta_metadata(self.rocksdb_engine_handler.clone(), &key, user)
    }

    pub fn list_by_cluster(&self, cluster_name: &str) -> Result<Vec<MqttUser>, CommonError> {
        let prefix_key = storage_key_mqtt_user_cluster_prefix(cluster_name);
        let data = engine_prefix_list_by_meta_metadata::<MqttUser>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn get(&self, cluster_name: &str, username: &str) -> Result<Option<MqttUser>, CommonError> {
        let key: String = storage_key_mqtt_user(cluster_name, username);
        if let Some(data) =
            engine_get_by_meta_metadata::<MqttUser>(self.rocksdb_engine_handler.clone(), &key)?
        {
            return Ok(Some(data.data));
        }
        Ok(None)
    }

    pub fn delete(&self, cluster_name: &str, user_name: &str) -> Result<(), CommonError> {
        let key: String = storage_key_mqtt_user(cluster_name, user_name);
        engine_delete_by_meta_metadata(self.rocksdb_engine_handler.clone(), &key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common_base::tools::now_second;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use rocksdb_engine::test::test_rocksdb_instance;

    fn setup_storage() -> MqttUserStorage {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());
        let db = test_rocksdb_instance();
        MqttUserStorage::new(db)
    }

    fn create_user(username: &str, password: &str, is_superuser: bool) -> MqttUser {
        MqttUser {
            username: username.to_string(),
            password: password.to_string(),
            salt: None,
            is_superuser,
            create_time: now_second(),
        }
    }

    #[test]
    fn test_user_crud_operations() {
        let storage = setup_storage();
        let cluster = "test_cluster";

        // Test: Save and Get
        let user1 = create_user("alice", "pass123", true);
        storage.save(cluster, "alice", user1).unwrap();

        let retrieved = storage.get(cluster, "alice").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().username, "alice");

        // Test: List multiple users
        let user2 = create_user("bob", "pass456", false);
        storage.save(cluster, "bob", user2).unwrap();

        let all_users = storage.list_by_cluster(cluster).unwrap();
        assert_eq!(all_users.len(), 2);

        // Test: Delete and verify
        storage.delete(cluster, "bob").unwrap();
        assert!(storage.get(cluster, "bob").unwrap().is_none());

        let remaining = storage.list_by_cluster(cluster).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].username, "alice");
    }

    #[test]
    fn test_get_nonexistent_user() {
        let storage = setup_storage();
        let result = storage.get("cluster1", "nonexistent").unwrap();
        assert!(result.is_none());
    }
}
