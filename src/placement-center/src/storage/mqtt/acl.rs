// Copyright 2023 RobustMQ Team
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

use crate::storage::{
    engine::{engine_delete_by_cluster, engine_prefix_list_by_cluster, engine_save_by_cluster},
    keys::{storage_key_mqtt_acl, storage_key_mqtt_acl_prefix},
    rocksdb::RocksDBEngine,
};
use common_base::error::common::CommonError;
use metadata_struct::acl::mqtt_acl::MQTTAcl;
use std::sync::Arc;

pub struct AclStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl AclStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        AclStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, cluster_name: &String, acl: MQTTAcl) -> Result<(), CommonError> {
        let key = storage_key_mqtt_acl(cluster_name, &acl.username);
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, acl);
    }

    pub fn list(&self, cluster_name: &String) -> Result<Vec<MQTTAcl>, CommonError> {
        let prefix_key = storage_key_mqtt_acl_prefix(cluster_name);
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<Vec<MQTTAcl>>(&raw.data) {
                        Ok(acl_list) => {
                            results.extend(acl_list);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                return Ok(results);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn delete(&self, cluster_name: &String, username: &String) -> Result<(), CommonError> {
        let key = storage_key_mqtt_acl(cluster_name, username);
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn unique_id_int() {}
}
