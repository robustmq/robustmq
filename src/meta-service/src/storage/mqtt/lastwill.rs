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

use metadata_struct::mqtt::lastwill::LastWillData;

use crate::core::error::MetaServiceError;
use crate::storage::engine_meta::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_save_by_meta,
};
use crate::storage::keys::storage_key_mqtt_last_will;
use rocksdb_engine::RocksDBEngine;

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
        last_will_message: LastWillData,
    ) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        engine_save_by_meta(self.rocksdb_engine_handler.clone(), key, last_will_message)?;
        Ok(())
    }

    pub fn get(
        &self,
        cluster_name: &str,
        client_id: &str,
    ) -> Result<Option<LastWillData>, MetaServiceError> {
        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        let result = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)?;
        if let Some(data) = result {
            return Ok(Some(serde_json::from_str::<LastWillData>(&data.data)?));
        }
        Ok(None)
    }

    pub fn delete(&self, cluster_name: &str, client_id: &str) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use broker_core::rocksdb::column_family_list;
    use common_base::utils::file_utils::test_temp_dir;
    use common_config::broker::{default_broker_config, init_broker_conf_by_config};
    use metadata_struct::mqtt::lastwill::LastWillData;
    use metadata_struct::mqtt::session::MqttSession;
    use rocksdb_engine::RocksDBEngine;

    use super::MqttLastWillStorage;
    use crate::storage::mqtt::session::MqttSessionStorage;

    #[tokio::test]
    async fn lastwill_storage_test() {
        let config = default_broker_config();
        init_broker_conf_by_config(config.clone());

        let rs = Arc::new(RocksDBEngine::new(
            &test_temp_dir(),
            config.rocksdb.max_open_files,
            column_family_list(),
        ));
        let session_storage = MqttSessionStorage::new(rs.clone());

        let cluster_name = "test_cluster".to_string();
        let client_id = "loboxu".to_string();
        let session = MqttSession::default();

        session_storage
            .save(&cluster_name, &client_id, session)
            .unwrap();

        let lastwill_storage = MqttLastWillStorage::new(rs.clone());
        let last_will_message = LastWillData {
            client_id: client_id.clone(),
            last_will: None,
            last_will_properties: None,
        };
        lastwill_storage
            .save(&cluster_name, &client_id, last_will_message)
            .unwrap();

        let data = lastwill_storage.get(&cluster_name, &client_id).unwrap();
        assert!(data.is_some());

        lastwill_storage.delete(&cluster_name, &client_id).unwrap();

        let data = lastwill_storage.get(&cluster_name, &client_id).unwrap();
        assert!(data.is_none());
    }
}
