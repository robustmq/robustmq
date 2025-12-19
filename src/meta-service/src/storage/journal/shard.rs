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

use crate::storage::keys::{key_all_shard, key_shard};
use common_base::error::common::CommonError;
use metadata_struct::storage::shard::EngineShard;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};
use std::sync::Arc;

pub struct ShardStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl ShardStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        ShardStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, shard_info: &EngineShard) -> Result<(), CommonError> {
        let shard_key = key_shard(&shard_info.shard_name);
        engine_save_by_meta_metadata(self.rocksdb_engine_handler.clone(), &shard_key, shard_info)
    }

    pub fn get(&self, shard_name: &str) -> Result<Option<EngineShard>, CommonError> {
        let shard_key: String = key_shard(shard_name);
        if let Some(data) = engine_get_by_meta_metadata::<EngineShard>(
            self.rocksdb_engine_handler.clone(),
            &shard_key,
        )? {
            return Ok(Some(data.data));
        }
        Ok(None)
    }

    pub fn delete(&self, shard_name: &str) -> Result<(), CommonError> {
        let shard_key = key_shard(shard_name);
        engine_delete_by_meta_metadata(self.rocksdb_engine_handler.clone(), &shard_key)
    }

    pub fn all_shard(&self) -> Result<Vec<EngineShard>, CommonError> {
        let prefix_key = key_all_shard();
        let data = engine_prefix_list_by_meta_metadata::<EngineShard>(
            self.rocksdb_engine_handler.clone(),
            prefix_key,
        )?;

        let mut results = Vec::new();
        for raw in data {
            results.push(raw.data);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {}
