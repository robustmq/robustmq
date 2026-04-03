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
use metadata_struct::mqtt::user::SecurityUser;

use rocksdb_engine::keys::meta::{
    storage_key_mqtt_user, storage_key_mqtt_user_prefix, storage_key_mqtt_user_tenant_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};

pub struct SecurityUserStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl SecurityUserStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        SecurityUserStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, tenant: &str, user_name: &str, user: SecurityUser) -> Result<(), CommonError> {
        let key = storage_key_mqtt_user(tenant, user_name);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, user)
    }

    pub fn list_all(&self) -> Result<Vec<SecurityUser>, CommonError> {
        let prefix_key = storage_key_mqtt_user_prefix();
        let data = engine_prefix_list_by_meta_metadata::<SecurityUser>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn list_by_tenant(&self, tenant: &str) -> Result<Vec<SecurityUser>, CommonError> {
        let prefix_key = storage_key_mqtt_user_tenant_prefix(tenant);
        let data = engine_prefix_list_by_meta_metadata::<SecurityUser>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn get(&self, tenant: &str, username: &str) -> Result<Option<SecurityUser>, CommonError> {
        let key: String = storage_key_mqtt_user(tenant, username);
        if let Some(data) =
            engine_get_by_meta_metadata::<SecurityUser>(&self.rocksdb_engine_handler, &key)?
        {
            return Ok(Some(data.data));
        }
        Ok(None)
    }

    pub fn delete(&self, tenant: &str, user_name: &str) -> Result<(), CommonError> {
        let key: String = storage_key_mqtt_user(tenant, user_name);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)
    }
}
