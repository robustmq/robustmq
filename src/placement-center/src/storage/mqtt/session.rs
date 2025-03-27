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
use rocksdb_engine::warp::StorageDataWrap;

use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{storage_key_mqtt_session, storage_key_mqtt_session_cluster_prefix};
use crate::storage::rocksdb::RocksDBEngine;

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
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, session)
    }

    pub fn list(&self, cluster_name: &str) -> Result<Vec<StorageDataWrap>, CommonError> {
        let prefix_key = storage_key_mqtt_session_cluster_prefix(cluster_name);
        engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        client_id: &str,
    ) -> Result<Option<MqttSession>, CommonError> {
        let key: String = storage_key_mqtt_session(cluster_name, client_id);
        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)? {
            return Ok(Some(serde_json::from_str::<MqttSession>(&data.data)?));
        }
        Ok(None)
    }

    pub fn delete(&self, cluster_name: &str, client_id: &str) -> Result<(), CommonError> {
        let key: String = storage_key_mqtt_session(cluster_name, client_id);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common_base::config::placement_center::placement_center_test_conf;
    use common_base::utils::file_utils::test_temp_dir;
    use metadata_struct::mqtt::session::MqttSession;

    use crate::storage::mqtt::session::MqttSessionStorage;
    use crate::storage::rocksdb::{column_family_list, RocksDBEngine};

    #[tokio::test]
    async fn session_storage_test() {
        let config = placement_center_test_conf();
        let rs = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files.unwrap(),
            column_family_list(),
        ));
        let session_storage = MqttSessionStorage::new(rs);
        let cluster_name = "test_cluster".to_string();
        let client_id = "loboxu".to_string();
        let session = MqttSession::default();
        session_storage
            .save(&cluster_name, &client_id, session)
            .unwrap();

        let client_id = "lobo1".to_string();
        let session = MqttSession::default();
        session_storage
            .save(&cluster_name, &client_id, session)
            .unwrap();

        let res = session_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 2);

        let res = session_storage.get(&cluster_name, "lobo1").unwrap();
        assert!(res.is_some());

        session_storage.delete(&cluster_name, "lobo1").unwrap();

        let res = session_storage.get(&cluster_name, "lobo1").unwrap();
        assert!(res.is_none());
    }
}
