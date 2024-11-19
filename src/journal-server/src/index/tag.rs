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

use rocksdb_engine::engine::{rocksdb_engine_get, rocksdb_engine_save};
use rocksdb_engine::RocksDBEngine;

use super::keys::{key_segment, tag_segment};
use crate::core::consts::DB_COLUMN_FAMILY_INDEX;
use crate::core::error::JournalServerError;

pub struct TagIndexManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl TagIndexManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        TagIndexManager {
            rocksdb_engine_handler,
        }
    }

    pub fn save_tag_position(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        tag: String,
        position: u64,
    ) -> Result<(), JournalServerError> {
        let key = tag_segment(namespace, shard_name, segment, tag);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            position,
        )?)
    }

    pub fn get_tag_position(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        tag: String,
    ) -> Result<u64, JournalServerError> {
        let key = tag_segment(namespace, shard_name, segment, tag);
        if let Some(res) = rocksdb_engine_get(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
        )? {
            return Ok(serde_json::from_slice::<u64>(&res.data)?);
        }

        Ok(0)
    }

    pub fn save_key_position(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        key: String,
        position: u64,
    ) -> Result<(), JournalServerError> {
        let key = key_segment(namespace, shard_name, segment, key);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            position,
        )?)
    }

    pub fn get_key_position(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        key: String,
    ) -> Result<u64, JournalServerError> {
        let key = key_segment(namespace, shard_name, segment, key);
        if let Some(res) = rocksdb_engine_get(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
        )? {
            return Ok(serde_json::from_slice::<u64>(&res.data)?);
        }

        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn timestamp_index_test() {}
}
