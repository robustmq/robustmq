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

use crate::core::error::MetaServiceError;
use metadata_struct::tenant::Tenant;
use rocksdb_engine::keys::meta::{storage_key_tenant, storage_key_tenant_prefix};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};
use std::sync::Arc;

pub struct TenantStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl TenantStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        TenantStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, tenant: &Tenant) -> Result<(), MetaServiceError> {
        let key = storage_key_tenant(&tenant.tenant_name);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, tenant)?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<Tenant>, MetaServiceError> {
        let prefix_key = storage_key_tenant_prefix();
        let data = engine_prefix_list_by_meta_metadata::<Tenant>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn get(&self, tenant_name: &str) -> Result<Option<Tenant>, MetaServiceError> {
        let key = storage_key_tenant(tenant_name);
        if let Some(data) =
            engine_get_by_meta_metadata::<Tenant>(&self.rocksdb_engine_handler, &key)?
        {
            return Ok(Some(data.data));
        }
        Ok(None)
    }

    pub fn delete(&self, tenant_name: &str) -> Result<(), MetaServiceError> {
        let key = storage_key_tenant(tenant_name);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)?;
        Ok(())
    }
}
