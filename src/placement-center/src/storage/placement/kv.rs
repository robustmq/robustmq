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
    engine_delete_by_cluster, engine_exists_by_cluster, engine_get_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::rocksdb::RocksDBEngine;

#[derive(Debug, Clone)]
pub struct KvStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl KvStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        KvStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn set(&self, key: String, value: String) -> Result<(), CommonError> {
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), key, value)
    }

    pub fn delete(&self, key: String) -> Result<(), CommonError> {
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }

    pub fn get(&self, key: String) -> Result<Option<String>, CommonError> {
        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), key)? {
            return Ok(Some(serde_json::from_slice::<String>(&data.data)?));
        }
        Ok(None)
    }

    pub fn exists(&self, key: String) -> Result<bool, CommonError> {
        engine_exists_by_cluster(self.rocksdb_engine_handler.clone(), key)
    }
}
