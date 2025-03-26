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

use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_save_by_cluster,
};
use crate::storage::keys::key_resource_config;
use crate::storage::rocksdb::RocksDBEngine;

pub struct ResourceConfigStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ResourceConfigStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ResourceConfigStorage {
            rocksdb_engine_handler,
        }
    }
    pub fn save(
        &self,
        cluster_name: String,
        resource_key: Vec<String>,
        config: Vec<u8>,
    ) -> Result<(), CommonError> {
        let key = key_resource_config(cluster_name, resource_key.join("/"));
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, config)
    }

    pub fn delete(
        &self,
        cluster_name: String,
        resource_key: Vec<String>,
    ) -> Result<(), CommonError> {
        let key = key_resource_config(cluster_name, resource_key.join("/"));
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }

    pub fn get(
        &self,
        cluster_name: String,
        resource_key: Vec<String>,
    ) -> Result<Option<Vec<u8>>, CommonError> {
        let key = key_resource_config(cluster_name, resource_key.join("/"));

        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)? {
            return Ok(Some(serde_json::from_str::<Vec<u8>>(&data.data)?));
        }
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use super::ResourceConfigStorage;
    use crate::storage::rocksdb::RocksDBEngine;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[test]
    fn resource_config_storage_test() {
        let rocksdb_engine = Arc::new(RocksDBEngine::new(
            tempdir().unwrap().path().to_str().unwrap(),
            100,
            vec!["cluster".to_string()],
        ));
        let resource_storage = ResourceConfigStorage::new(rocksdb_engine);

        let cluster_name = "cluster1".to_string();
        let resource_key = vec!["config".to_string(), "sub_config".to_string()];
        let config_data = vec![1, 2, 3, 4, 5];

        assert!(resource_storage
            .save(
                cluster_name.clone(),
                resource_key.clone(),
                config_data.clone()
            )
            .is_ok());

        let retrieved_config = resource_storage
            .get(cluster_name.clone(), resource_key.clone())
            .unwrap();
        assert!(retrieved_config.is_some());
        assert_eq!(retrieved_config.unwrap(), config_data);

        assert!(resource_storage
            .delete(cluster_name.clone(), resource_key.clone())
            .is_ok());

        let deleted_config = resource_storage
            .get(cluster_name.clone(), resource_key.clone())
            .unwrap();
        assert!(deleted_config.is_none());

        let nonexistent_config = resource_storage
            .get(
                "nonexistent_cluster".to_string(),
                vec!["nonexistent".to_string()],
            )
            .unwrap();
        assert!(nonexistent_config.is_none());
    }
}
