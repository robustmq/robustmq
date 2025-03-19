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
use metadata_struct::journal::shard::JournalShard;

use crate::storage::engine::{
    engine_delete_by_cluster, engine_get_by_cluster, engine_prefix_list_by_cluster,
    engine_save_by_cluster,
};
use crate::storage::keys::{
    key_all_shard, key_shard, key_shard_cluster_prefix, key_shard_namespace_prefix,
};
use crate::storage::rocksdb::RocksDBEngine;

pub struct ShardStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShardStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ShardStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, shard_info: &JournalShard) -> Result<(), CommonError> {
        let shard_key = key_shard(
            &shard_info.cluster_name,
            &shard_info.namespace,
            &shard_info.shard_name,
        );
        engine_save_by_cluster(self.rocksdb_engine_handler.clone(), shard_key, shard_info)
    }

    pub fn get(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Result<Option<JournalShard>, CommonError> {
        let shard_key: String = key_shard(cluster_name, namespace, shard_name);
        if let Some(data) = engine_get_by_cluster(self.rocksdb_engine_handler.clone(), shard_key)? {
            return Ok(Some(serde_json::from_str::<JournalShard>(&data.data)?));
        }
        Ok(None)
    }

    pub fn delete(
        &self,
        cluster_name: &str,
        namespace: &str,
        shard_name: &str,
    ) -> Result<(), CommonError> {
        let shard_key = key_shard(cluster_name, namespace, shard_name);
        engine_delete_by_cluster(self.rocksdb_engine_handler.clone(), shard_key)
    }

    pub fn all_shard(&self) -> Result<Vec<JournalShard>, CommonError> {
        let prefix_key = key_all_shard();
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;

        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalShard>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_cluster_namespace(
        &self,
        cluster_name: &str,
        namespace: &str,
    ) -> Result<Vec<JournalShard>, CommonError> {
        let prefix_key = key_shard_namespace_prefix(cluster_name, namespace);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;

        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalShard>(&raw.data)?);
        }
        Ok(results)
    }

    pub fn list_by_cluster(&self, cluster_name: &str) -> Result<Vec<JournalShard>, CommonError> {
        let prefix_key = key_shard_cluster_prefix(cluster_name);
        let data = engine_prefix_list_by_cluster(self.rocksdb_engine_handler.clone(), prefix_key)?;

        let mut results = Vec::new();
        for raw in data {
            results.push(serde_json::from_str::<JournalShard>(&raw.data)?);
        }
        Ok(results)
    }
}
