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

use crate::storage::{keys::key_resource_acl, rocksdb::RocksDBEngine, StorageDataWrap};
use common_base::errors::RobustMQError;
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
    pub fn save(&self, cluster_name: String, acl: CommonAcl) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        
        let key = key_resource_acl(
            cluster_name,
            acl.principal_type.clone().to_string(),
            acl.principal.clone(),
        );

        let content = match serde_json::to_vec(&acl) {
            Ok(data) => data,
            Err(e) => return Err(RobustMQError::CommmonError(e.to_string())),
        };

        let data = StorageDataWrap::new(content);
        match self.rocksdb_engine_handler.write(cf, &key, &data) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }

    pub fn delete(&self, cluster_name: String, acl: CommonAcl) -> Result<(), RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = key_resource_acl(cluster_name, acl.principal_type.to_string(), acl.principal);
        match self.rocksdb_engine_handler.delete(cf, &key) {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }

    pub fn get(
        &self,
        cluster_name: String,
        principal_type: String,
        principal: String,
    ) -> Result<Option<String>, RobustMQError> {
        let cf = self.rocksdb_engine_handler.cf_cluster();
        let key = key_resource_acl(cluster_name, principal_type, principal);
        match self.rocksdb_engine_handler.read::<String>(cf, &key) {
            Ok(cluster_info) => {
                return Ok(cluster_info);
            }
            Err(e) => {
                return Err(RobustMQError::CommmonError(e));
            }
        }
    }
}
