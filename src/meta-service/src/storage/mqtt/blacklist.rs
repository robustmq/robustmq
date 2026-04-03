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
use metadata_struct::auth::blacklist::SecurityBlackList;

use rocksdb_engine::keys::meta::{
    storage_key_mqtt_blacklist, storage_key_mqtt_blacklist_prefix,
    storage_key_mqtt_blacklist_tenant_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};

pub struct MqttBlackListStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl MqttBlackListStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        MqttBlackListStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, blacklist: SecurityBlackList) -> Result<(), CommonError> {
        let key = storage_key_mqtt_blacklist(&blacklist.tenant, &blacklist.name);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, blacklist)
    }

    pub fn get(&self, tenant: &str, name: &str) -> Result<Option<SecurityBlackList>, CommonError> {
        let key = storage_key_mqtt_blacklist(tenant, name);
        Ok(
            engine_get_by_meta_metadata::<SecurityBlackList>(&self.rocksdb_engine_handler, &key)?
                .map(|raw| raw.data),
        )
    }

    pub fn list_all(&self) -> Result<Vec<SecurityBlackList>, CommonError> {
        let prefix_key = storage_key_mqtt_blacklist_prefix();
        let data = engine_prefix_list_by_meta_metadata::<SecurityBlackList>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn list_by_tenant(&self, tenant: &str) -> Result<Vec<SecurityBlackList>, CommonError> {
        let prefix_key = storage_key_mqtt_blacklist_tenant_prefix(tenant);
        let data = engine_prefix_list_by_meta_metadata::<SecurityBlackList>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn delete(&self, tenant: &str, name: &str) -> Result<(), CommonError> {
        let key = storage_key_mqtt_blacklist(tenant, name);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)
    }
}
