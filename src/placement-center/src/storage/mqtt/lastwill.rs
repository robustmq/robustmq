// Copyright [RobustMQ]

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
    engine::{engine_delete_by_cluster, engine_get_by_cluster, engine_save_by_cluster},
    keys::{storage_key_mqtt_last_will, storage_key_mqtt_session},
    rocksdb::RocksDBEngine,
    StorageDataWrap,
};
use common_base::errors::RobustMQError;
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
        last_will_message: Vec<u8>,
    ) -> Result<(), RobustMQError> {

        let results = match self.get(cluster_name, client_id) {
            Ok(data) => data,
            Err(e) => {
                return Err(e);
            }
        };
        if results.is_none() {
            return Err(RobustMQError::SessionDoesNotExist);
        }

        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, last_will_message);
    }

    pub fn get(
        &self,
        cluster_name: &String,
        client_id: &String,
    ) -> Result<Option<StorageDataWrap>, RobustMQError> {
        let key = storage_key_mqtt_last_will(cluster_name, client_id);
        return engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }

    pub fn delete_last_will_message(
        &self,
        cluster_name: &String,
        client_id: &String,
    ) -> Result<(), RobustMQError> {
        let key = storage_key_mqtt_session(cluster_name, client_id);
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn lastwill_storage_test() {}
}
