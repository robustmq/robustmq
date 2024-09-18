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

use crate::storage::{
    engine::{
        engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
        engine_save_by_cluster,
    },
    keys::{storage_key_mqtt_session, storage_key_mqtt_session_cluster_prefix},
    rocksdb::RocksDBEngine,
    StorageDataWrap,
};
use common_base::error::common::CommonError;
use metadata_struct::mqtt::session::MQTTSession;
use std::sync::Arc;

pub struct MQTTSessionStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MQTTSessionStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MQTTSessionStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(
        &self,
        cluster_name: &String,
        client_id: &String,
        session: MQTTSession,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_session(cluster_name, client_id);
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, session);
    }

    pub fn list(&self, cluster_name: &String) -> Result<Vec<StorageDataWrap>, CommonError> {
        let prefix_key = storage_key_mqtt_session_cluster_prefix(&cluster_name);
        return engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key);
    }

    pub fn get(
        &self,
        cluster_name: &String,
        client_id: &String,
    ) -> Result<Option<MQTTSession>, CommonError> {
        let key: String = storage_key_mqtt_session(cluster_name, client_id);
        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key) {
            Ok(Some(data)) => match serde_json::from_slice::<MQTTSession>(&data.data) {
                Ok(session) => {
                    return Ok(Some(session));
                }
                Err(e) => {
                    return Err(e.into());
                }
            },
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn delete(&self, cluster_name: &String, client_id: &String) -> Result<(), CommonError> {
        let key: String = storage_key_mqtt_session(cluster_name, client_id);
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::mqtt::session::MQTTSessionStorage;
    use crate::storage::rocksdb::RocksDBEngine;
    use common_base::config::placement_center::PlacementCenterConfig;
    use common_base::tools::unique_id;
    use metadata_struct::mqtt::session::MQTTSession;
    use std::fs::remove_dir_all;
    use std::sync::Arc;

    #[tokio::test]
    async fn topic_storage_test() {
        let mut config = PlacementCenterConfig::default();
        config.data_path = format!("/tmp/{}", unique_id());
        config.rocksdb.max_open_files = Some(10);
        
        let rs = Arc::new(RocksDBEngine::new(&config));
        let session_storage = MQTTSessionStorage::new(rs);
        let cluster_name = "test_cluster".to_string();
        let client_id = "loboxu".to_string();
        let session = MQTTSession::default();
        session_storage
            .save(&cluster_name, &client_id, session)
            .unwrap();

        let client_id = "lobo1".to_string();
        let session = MQTTSession::default();
        session_storage
            .save(&cluster_name, &client_id, session)
            .unwrap();

        let res = session_storage.list(&cluster_name).unwrap();
        assert_eq!(res.len(), 2);

        let res = session_storage
            .get(&cluster_name, &"lobo1".to_string())
            .unwrap();
        assert!(!res.is_none());

        session_storage
            .delete(&cluster_name, &"lobo1".to_string())
            .unwrap();

        let res = session_storage
            .get(&cluster_name, &"lobo1".to_string())
            .unwrap();
        assert!(res.is_none());

        remove_dir_all(config.data_path).unwrap();
    }
}
