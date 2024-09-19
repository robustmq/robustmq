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

use super::session::MQTTSessionStorage;
use crate::storage::{
    engine::{engine_delete_by_cluster, engine_get_by_cluster, engine_save_by_cluster},
    keys::storage_key_mqtt_last_will,
    rocksdb::RocksDBEngine,
};
use common_base::error::{common::CommonError, mqtt_broker::MQTTBrokerError};
use metadata_struct::mqtt::lastwill::LastWillData;
use std::sync::Arc;

pub struct MQTTLastWillStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MQTTLastWillStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MQTTLastWillStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(
        &self,
        cluster_name: &String,
        client_id: &String,
        last_will_message: LastWillData,
    ) -> Result<(), CommonError> {
        let session_storage = MQTTSessionStorage::new(self.rocksdb_engine_handler.clone());
        let results = session_storage.get(cluster_name, client_id)?;
        if results.is_none() {
            return Err(MQTTBrokerError::SessionDoesNotExist.into());
        }

        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, last_will_message);
    }

    pub fn get(
        &self,
        cluster_name: &String,
        client_id: &String,
    ) -> Result<Option<LastWillData>, CommonError> {
        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key) {
            Ok(Some(data)) => match serde_json::from_slice::<LastWillData>(&data.data) {
                Ok(lastwill) => {
                    return Ok(Some(lastwill));
                }
                Err(e) => {
                    return Err(e.into());
                }
            },
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => Err(e),
        }
    }

    pub fn delete(&self, cluster_name: &String, client_id: &String) -> Result<(), CommonError> {
        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }
}

#[cfg(test)]
mod tests {
    use std::{fs::remove_dir_all, sync::Arc};

    use super::MQTTLastWillStorage;
    use crate::storage::{mqtt::session::MQTTSessionStorage, rocksdb::RocksDBEngine};
    use common_base::{config::placement_center::PlacementCenterConfig, tools::unique_id};
    use metadata_struct::mqtt::{lastwill::LastWillData, session::MQTTSession};

    #[tokio::test]
    async fn lastwill_storage_test() {
        let mut config = PlacementCenterConfig::default();
        config.rocksdb.data_path = format!("/tmp/{}", unique_id());
        config.rocksdb.max_open_files = Some(10);

        let rs = Arc::new(RocksDBEngine::new(&config));
        let session_storage = MQTTSessionStorage::new(rs.clone());

        let cluster_name = "test_cluster".to_string();
        let client_id = "loboxu".to_string();
        let session = MQTTSession::default();

        session_storage
            .save(&cluster_name, &client_id, session)
            .unwrap();

        let lastwill_storage = MQTTLastWillStorage::new(rs.clone());
        let last_will_message = LastWillData {
            client_id: client_id.clone(),
            last_will: None,
            last_will_properties: None,
        };
        lastwill_storage
            .save(&cluster_name, &client_id, last_will_message)
            .unwrap();

        let data = lastwill_storage.get(&cluster_name, &client_id).unwrap();
        assert!(!data.is_none());

        lastwill_storage.delete(&cluster_name, &client_id).unwrap();

        let data = lastwill_storage.get(&cluster_name, &client_id).unwrap();
        assert!(data.is_none());

        remove_dir_all(config.rocksdb.data_path).unwrap();
    }
}
