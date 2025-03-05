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

use crate::core::error::PlacementCenterError;
use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{
    storage_key_mqtt_schema, storage_key_mqtt_schema_bind,
    storage_key_mqtt_schema_bind_prefix_by_cluster,
    storage_key_mqtt_schema_bind_prefix_by_resource, storage_key_mqtt_schema_prefix,
};
use crate::storage::rocksdb::RocksDBEngine;
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
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

    pub fn save(
        &self,
        cluster_name: &str,
        schema_name: &str,
        schema: &SchemaData,
    ) -> Result<(), PlacementCenterError> {
        let key = storage_key_mqtt_schema(cluster_name, schema_name);
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, schema)?;
        Ok(())
    }

    pub fn list(&self, cluster_name: &str) -> Result<Vec<SchemaData>, PlacementCenterError> {
        let prefix_key = storage_key_mqtt_schema_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            let topic = serde_json::from_slice::<SchemaData>(&raw.data)?;
            results.push(topic);
        }
        Ok(results)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        schema_name: &str,
    ) -> Result<Option<SchemaData>, PlacementCenterError> {
        let key: String = storage_key_mqtt_schema(cluster_name, schema_name);

        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)? {
            let topic = serde_json::from_slice::<SchemaData>(&data.data)?;
            return Ok(Some(topic));
        }
        Ok(None)
    }

    pub fn delete(
        &self,
        cluster_name: &str,
        schema_name: &str,
    ) -> Result<(), PlacementCenterError> {
        let key: String = storage_key_mqtt_schema(cluster_name, schema_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)?;
        Ok(())
    }

    pub fn save_bind(
        &self,
        cluster_name: &str,
        bind_data: SchemaResourceBind,
    ) -> Result<(), PlacementCenterError> {
        let key = storage_key_mqtt_schema_bind(
            cluster_name,
            &bind_data.schema_name,
            &bind_data.resource_name,
        );
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, bind_data)?;
        Ok(())
    }

    pub fn list_bind_by_resource(
        &self,
        cluster_name: &str,
        resource_name: &str,
    ) -> Result<Vec<SchemaResourceBind>, PlacementCenterError> {
        let prefix_key =
            storage_key_mqtt_schema_bind_prefix_by_resource(cluster_name, resource_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            let topic = serde_json::from_slice::<SchemaResourceBind>(&raw.data)?;
            results.push(topic);
        }
        Ok(results)
    }

    pub fn list_bind_by_cluster(
        &self,
        cluster_name: &str,
    ) -> Result<Vec<SchemaResourceBind>, PlacementCenterError> {
        let prefix_key = storage_key_mqtt_schema_bind_prefix_by_cluster(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;
        let mut results = Vec::new();
        for raw in data {
            let topic = serde_json::from_slice::<SchemaResourceBind>(&raw.data)?;
            results.push(topic);
        }
        Ok(results)
    }

    pub fn get_bind(
        &self,
        cluster_name: &str,
        resource_name: &str,
        schema_name: &str,
    ) -> Result<Option<SchemaResourceBind>, PlacementCenterError> {
        let key: String = storage_key_mqtt_schema_bind(cluster_name, resource_name, schema_name);

        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)? {
            let topic = serde_json::from_slice::<SchemaResourceBind>(&data.data)?;
            return Ok(Some(topic));
        }
        Ok(None)
    }

    pub fn delete_bind(
        &self,
        cluster_name: &str,
        resource_name: &str,
        schema_name: &str,
    ) -> Result<(), PlacementCenterError> {
        let key: String = storage_key_mqtt_schema_bind(cluster_name, resource_name, schema_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn schema_storage_test() {}

    #[tokio::test]
    async fn schema_bind_storage_test() {}
}
