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
    engine::{engine_delete_by_cluster, engine_get_by_cluster, engine_save_by_cluster},
    keys::key_resource_config,
    rocksdb::RocksDBEngine,
};
use common_base::error::common::CommonError;
use std::sync::Arc;

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
        return engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, config);
    }

    pub fn delete(
        &self,
        cluster_name: String,
        resource_key: Vec<String>,
    ) -> Result<(), CommonError> {
        let key = key_resource_config(cluster_name, resource_key.join("/"));
        return engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key);
    }

    pub fn get(
        &self,
        cluster_name: String,
        resource_key: Vec<String>,
    ) -> Result<Option<Vec<u8>>, CommonError> {
        let key = key_resource_config(cluster_name, resource_key.join("/"));

        match engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key) {
            Ok(Some(data)) => match serde_json::from_slice::<Vec<u8>>(&data.data) {
                Ok(config) => {
                    return Ok(Some(config));
                }
                Err(e) => {
                    return Err(e.into());
                }
            },
            Ok(None) => {
                return Ok(None);
            }
            Err(e) => Err(e),
        }
    }
}
