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
use metadata_struct::auth::acl::SecurityAcl;

use rocksdb_engine::keys::meta::{
    storage_key_mqtt_acl, storage_key_mqtt_acl_prefix, storage_key_mqtt_acl_tenant_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};

pub struct AclStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl AclStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        AclStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, acl: SecurityAcl) -> Result<(), CommonError> {
        let key = storage_key_mqtt_acl(&acl.tenant, &acl.name);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, acl)
    }

    pub fn list_all(&self) -> Result<Vec<SecurityAcl>, CommonError> {
        let prefix_key = storage_key_mqtt_acl_prefix();
        let data = engine_prefix_list_by_meta_metadata::<SecurityAcl>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn list_by_tenant(&self, tenant: &str) -> Result<Vec<SecurityAcl>, CommonError> {
        let prefix_key = storage_key_mqtt_acl_tenant_prefix(tenant);
        let data = engine_prefix_list_by_meta_metadata::<SecurityAcl>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn get(&self, tenant: &str, name: &str) -> Result<Option<SecurityAcl>, CommonError> {
        let key = storage_key_mqtt_acl(tenant, name);
        Ok(
            engine_get_by_meta_metadata::<SecurityAcl>(&self.rocksdb_engine_handler, &key)?
                .map(|raw| raw.data),
        )
    }

    pub fn delete(&self, tenant: &str, name: &str) -> Result<(), CommonError> {
        let key = storage_key_mqtt_acl(tenant, name);
        match engine_get_by_meta_metadata::<SecurityAcl>(&self.rocksdb_engine_handler, &key)? {
            None => Ok(()),
            Some(_) => engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key),
        }
    }
}
