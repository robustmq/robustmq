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
use crate::storage::keys::{
    storage_key_mqtt_schema, storage_key_mqtt_schema_bind,
    storage_key_mqtt_schema_bind_prefix_by_resource, storage_key_mqtt_schema_prefix,
};
use metadata_struct::schema::{SchemaData, SchemaResourceBind};
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

    pub fn save(&self, schema_name: &str, schema: &SchemaData) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_schema(schema_name);
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

    pub fn get(&self, schema_name: &str) -> Result<Option<SchemaData>, MetaServiceError> {
        let key: String = storage_key_mqtt_schema(schema_name);

        if let Some(data) =
            engine_get_by_meta_metadata::<SchemaData>(&self.rocksdb_engine_handler, &key)?
        {
            let topic: SchemaData = data.data;
            return Ok(Some(topic));
        }
        Ok(None)
    }

    pub fn delete(&self, schema_name: &str) -> Result<(), MetaServiceError> {
        let key: String = storage_key_mqtt_schema(schema_name);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)?;
        Ok(())
    }

    pub fn save_bind(&self, bind_data: &SchemaResourceBind) -> Result<(), MetaServiceError> {
        let key = storage_key_mqtt_schema_bind(&bind_data.resource_name, &bind_data.schema_name);
        engine_save_by_meta_metadata(&self.rocksdb_engine_handler, &key, bind_data)?;
        Ok(())
    }

    pub fn list_bind_by_resource(
        &self,
        resource_name: &str,
    ) -> Result<Vec<SchemaResourceBind>, MetaServiceError> {
        let prefix_key = storage_key_mqtt_schema_bind_prefix_by_resource(resource_name);
        let data = engine_prefix_list_by_meta_metadata::<SchemaResourceBind>(
            &self.rocksdb_engine_handler,
            &prefix_key,
        )?;
        let mut results = Vec::new();
        for raw in data {
            results.push(raw.data);
        }
        Ok(results)
    }

    pub fn get_bind(
        &self,
        resource_name: &str,
        schema_name: &str,
    ) -> Result<Option<SchemaResourceBind>, MetaServiceError> {
        let key: String = storage_key_mqtt_schema_bind(resource_name, schema_name);

        if let Some(data) =
            engine_get_by_meta_metadata::<SchemaResourceBind>(&self.rocksdb_engine_handler, &key)?
        {
            return Ok(Some(data.data));
        }
        Ok(None)
    }

    pub fn delete_bind(
        &self,
        resource_name: &str,
        schema_name: &str,
    ) -> Result<(), MetaServiceError> {
        let key: String = storage_key_mqtt_schema_bind(resource_name, schema_name);
        engine_delete_by_meta_metadata(&self.rocksdb_engine_handler, &key)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use metadata_struct::schema::SchemaType;
    use metadata_struct::schema::{SchemaData, SchemaResourceBind};
    use rocksdb_engine::storage::family::column_family_list;
    use tempfile::tempdir;

    use crate::storage::common::schema::SchemaStorage;
    use rocksdb_engine::rocksdb::RocksDBEngine;

    #[tokio::test]
    async fn schema_storage_test() {
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            tempdir().unwrap().path().to_str().unwrap(),
            100,
            column_family_list(),
        ));

        let schema_storage = SchemaStorage::new(rocksdb_engine.clone());

        let schema_name = "test_schema".to_string();
        let desc = "description";
        let schema = "{\"type\":\"object\"}";

        let schema_data = SchemaData {
            name: schema_name.clone(),
            schema_type: SchemaType::JSON,
            desc: desc.to_string(),
            schema: schema.to_string(),
        };

        //test func save()
        schema_storage.save(&schema_name, &schema_data).unwrap();

        //test func get()
        let retrieved_schema = schema_storage
            .get(&schema_name)
            .unwrap()
            .expect("schema not found");
        assert_eq!(retrieved_schema.name, "test_schema");
        assert_eq!(retrieved_schema.schema_type, SchemaType::JSON);

        //test func list()
        let schemas = schema_storage.list().unwrap();
        assert_eq!(schemas.len(), 1);
        assert_eq!(schemas[0].name, "test_schema");

        //test func delete()
        schema_storage.delete(&schema_name).unwrap();
        let deleted_schema = schema_storage.get(&schema_name).unwrap();
        assert!(deleted_schema.is_none());
    }

    #[tokio::test]
    async fn schema_bind_storage_test() {
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            tempdir().unwrap().path().to_str().unwrap(),
            100,
            column_family_list(),
        ));

        let schema_storage = SchemaStorage::new(rocksdb_engine.clone());

        //create test data
        let schema_name = "test_schema".to_string();
        let resource_name = "test_resource";

        let bind_data = SchemaResourceBind {
            schema_name: schema_name.clone(),
            resource_name: resource_name.to_string(),
        };

        //test save_bind()
        schema_storage.save_bind(&bind_data).unwrap();

        //test list_bind_by_resource()
        let retrieved_binds = schema_storage.list_bind_by_resource(resource_name).unwrap();
        assert_eq!(retrieved_binds.len(), 1);
        assert_eq!(retrieved_binds[0].schema_name, "test_schema");

        //test get_bind()
        let retrieved_bind = schema_storage
            .get_bind(resource_name, &schema_name)
            .unwrap()
            .expect("Bind not found");
        assert_eq!(retrieved_bind.schema_name, "test_schema");

        //test delete_bind()
        schema_storage
            .delete_bind(resource_name, &schema_name)
            .unwrap();
        let deleted_bind = schema_storage
            .get_bind(&schema_name, resource_name)
            .unwrap();
        assert!(deleted_bind.is_none());
    }
}
