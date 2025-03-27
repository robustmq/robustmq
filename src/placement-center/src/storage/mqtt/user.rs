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

use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{storage_key_mqtt_user, storage_key_mqtt_user_cluster_prefix};
use crate::storage::rocksdb::RocksDBEngine;

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
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, user)
    }

    pub fn list(&self, cluster_name: &str) -> Result<Vec<MqttUser>, CommonError> {
        let prefix_key = storage_key_mqtt_user_cluster_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<MqttUser>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn get(&self, cluster_name: &str, username: &str) -> Result<Option<MqttUser>, CommonError> {
        let key: String = storage_key_mqtt_user(cluster_name, username);
        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)? {
            return Ok(Some(serde_json::from_str::<MqttUser>(&data.data)?));
        }
        Ok(None)
    }

    pub fn delete(&self, cluster_name: &str, user_name: &str) -> Result<(), CommonError> {
        let key: String = storage_key_mqtt_user(cluster_name, user_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use common_base::utils::file_utils::test_temp_dir;
    use metadata_struct::mqtt::user::MqttUser;

    use crate::storage::mqtt::user::MqttUserStorage;
    use crate::storage::rocksdb::{column_family_list, RocksDBEngine};

    #[tokio::test]
    async fn user_storage_test() {
        let config = placement_center_test_conf();

        let rs = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let user_storage = MqttUserStorage::new(rs);
        let cluster_name = "test_cluster".to_string();
        let username = "loboxu".to_string();
        let user = MqttUser {
            username: username.clone(),
            password: "pwd123".to_string(),
            is_superuser: true,
        };
        user_storage.save(&cluster_name, &username, user).unwrap();

        let username = "lobo1".to_string();
        let user = MqttUser {
            username: username.clone(),
            password: "pwd1231".to_string(),
            is_superuser: true,
        };
        user_storage.save(&cluster_name, &username, user).unwrap();

        let res = user_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 2);

        let res = user_storage.get(&cluster_name, "lobo1").unwrap();
        assert!(res.is_some());

        let name = "lobo1".to_string();
        user_storage.delete(&cluster_name, &name).unwrap();

        let res = user_storage.get(&cluster_name, "lobo1").unwrap();
        assert!(res.is_none());
    }
}
