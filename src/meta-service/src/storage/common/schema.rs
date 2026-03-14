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
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
use rocksdb_engine::keys::meta::{
    storage_key_mqtt_schema, storage_key_mqtt_schema_bind, storage_key_mqtt_schema_bind_prefix,
    storage_key_mqtt_schema_bind_prefix_by_resource, storage_key_mqtt_schema_bind_tenant_prefix,
    storage_key_mqtt_schema_prefix, storage_key_mqtt_schema_tenant_prefix,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};
use std::sync::Arc;

pub struct SchemaStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl SchemaStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        SchemaStorage {
            rocksdb_engine_handler,
        }
    }

    // Schema
    pub fn save(
        &self,
        tenant: &str,
        schema_name: &str,
        schema: &SchemaData,
    ) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_schema(tenant, schema_name);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, schema)?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<SchemaData>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_schema_prefix();
        let data = engine_prefix_list_by_meta_metadata::<SchemaData>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn list_by_tenant(&self, tenant: &str) -> Result<Vec<SchemaData>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_schema_tenant_prefix(tenant);
        let data = engine_prefix_list_by_meta_metadata::<SchemaData>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.into_iter().map(|raw| raw.data).collect())
    }

    pub fn get(
        &self,
        tenant: &str,
        schema_name: &str,
    ) -> Result<Option<SchemaData>, MetaServiceError> {
        let key = storage_key_mqtt_schema(tenant, schema_name);
        if let Some(data) =
            engine_get_by_meta_metadata::<SchemaData>(&self.rocksdb_engine_handler, &key)?
        {
            return Ok(Some(data.data));
        }
        Ok(None)
    }

    pub fn delete(&self, tenant: &str, schema_name: &str) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_schema(tenant, schema_name);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)?;
        Ok(())
    }

    // Schema Bind
    pub fn list_bind(&self) -> Result<Vec<SchemaResourceBind>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_schema_bind_prefix();
        let data = engine_prefix_list_by_meta_metadata::<SchemaResourceBind>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.iter().map(|raw| raw.data.clone()).collect())
    }

    pub fn list_bind_by_tenant(
        &self,
        tenant: &str,
    ) -> Result<Vec<SchemaResourceBind>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_schema_bind_tenant_prefix(tenant);
        let data = engine_prefix_list_by_meta_metadata::<SchemaResourceBind>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.iter().map(|raw| raw.data.clone()).collect())
    }

    pub fn save_bind(&self, bind_data: &SchemaResourceBind) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_schema_bind(
            &bind_data.tenant,
            &bind_data.resource_name,
            &bind_data.schema_name,
        );
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, bind_data)?;
        Ok(())
    }

    pub fn list_bind_by_resource(
        &self,
        tenant: &str,
        resource_name: &str,
    ) -> Result<Vec<SchemaResourceBind>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_schema_bind_prefix_by_resource(tenant, resource_name);
        let data = engine_prefix_list_by_meta_metadata::<SchemaResourceBind>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        Ok(data.iter().map(|raw| raw.data.clone()).collect())
    }

    pub fn get_bind(
        &self,
        tenant: &str,
        resource_name: &str,
        schema_name: &str,
    ) -> Result<Option<SchemaResourceBind>, MetaServiceError> {
        let key = storage_key_mqtt_schema_bind(tenant, resource_name, schema_name);
        if let Some(data) =
            engine_get_by_meta_metadata::<SchemaResourceBind>(&self.rocksdb_engine_handler, &key)?
        {
            return Ok(Some(data.data));
        }
        Ok(None)
    }

    pub fn delete_bind(
        &self,
        tenant: &str,
        resource_name: &str,
        schema_name: &str,
    ) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_schema_bind(tenant, resource_name, schema_name);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)?;
        Ok(())
    }
}
