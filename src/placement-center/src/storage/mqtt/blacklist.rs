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
use metadata_struct::acl::mqtt_blacklist::MqttAclBlackList;

use crate::storage::engine::{
    engine_delete_by_cluster, engine_prefix_list_by_cluster, engine_save_by_cluster,
};
use crate::storage::keys::{storage_key_mqtt_blacklist, storage_key_mqtt_blacklist_prefix};
use crate::storage::rocksdb::RocksDBEngine;

pub struct MqttBlackListStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttBlackListStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttBlackListStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(
        &self,
        cluster_name: &String,
        blacklist: MqttAclBlackList,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_blacklist(
            cluster_name,
            &blacklist.blacklist_type.to_string(),
            &blacklist.resource_name,
        );
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, blacklist)
    }

    pub fn list(&self, cluster_name: &String) -> Result<Vec<MqttAclBlackList>, CommonError> {
        let prefix_key = storage_key_mqtt_blacklist_prefix(cluster_name);
        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<MqttAclBlackList>(&raw.data) {
                        Ok(blacklist) => {
                            results.push(blacklist);
                        }
                        Err(e) => {
                            return Err(e.into());
                        }
                    }
                }
                Ok(results)
            }
            Err(e) => Err(e),
        }
    }

    pub fn delete(
        &self,
        cluster_name: &String,
        blacklist_type: &String,
        resource_name: &String,
    ) -> Result<(), CommonError> {
        let key = storage_key_mqtt_blacklist(cluster_name, blacklist_type, resource_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }
}
