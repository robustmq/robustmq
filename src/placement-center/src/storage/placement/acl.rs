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
    engine::{
        engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
        engine_save_by_cluster,
    },
    keys::{key_resource_acl, key_resource_acl_prefix},
    rocksdb::RocksDBEngine,
};
use common_base::error::robustmq::RobustMQError;
use metadata_struct::acl::CommonAcl;
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

    pub fn save(&self, cluster_name: &String, acl: CommonAcl) -> Result<(), RobustMQError> {
        let mut data = self.get(
            cluster_name,
            &acl.principal_type.to_string(),
            &acl.principal,
        )?;
        for raw in data.clone() {
            if raw == acl {
                return Ok(());
            }
        }

        let key = key_resource_acl(
            cluster_name,
            &acl.principal_type.to_string(),
            &acl.principal,
        );
        data.push(acl);
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, data);
    }

    pub fn list(&self, cluster_name: &String) -> Result<Vec<CommonAcl>, RobustMQError> {
        let prefix_key = key_resource_acl_prefix(cluster_name);

        match engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key) {
            Ok(data) => {
                let mut results = Vec::new();
                for raw in data {
                    match serde_json::from_slice::<Vec<CommonAcl>>(&raw.data) {
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

    pub fn delete(
        &self,
        cluster_name: &String,
        principal_type: &String,
        principal: &String,
    ) -> Result<(), RobustMQError> {
        let key = key_resource_acl(cluster_name, principal_type, principal);
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }

    pub fn get(
        &self,
        cluster_name: &String,
        principal_type: &String,
        principal: &String,
    ) -> Result<Vec<CommonAcl>, RobustMQError> {
        let key = key_resource_acl(cluster_name, principal_type, principal);
        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key) {
            Ok(Some(data)) => match serde_json::from_slice::<Vec<CommonAcl>>(&data.data) {
                Ok(config) => {
                    return Ok(config);
                }
                Err(e) => {
                    return Err(e.into());
                }
            },
            Ok(None) => {
                return Ok(Vec::new());
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn unique_id_int() {}
}
