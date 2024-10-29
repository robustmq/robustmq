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

use rocksdb_engine::engine::{rocksdb_engine_delete, rocksdb_engine_save};
use rocksdb_engine::RocksDBEngine;

use super::engine::DB_COLUMN_FAMILY_INDEX;
use super::keys::{end_position_index_key, start_position_index_key};
use crate::core::error::JournalServerError;

pub struct SegmentIndexManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl SegmentIndexManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        SegmentIndexManager {
            rocksdb_engine_handler,
        }
    }

    pub fn save_start_position(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        offset: u64,
        position: u128,
    ) -> Result<(), JournalServerError> {
        let key = start_position_index_key(namespace, shard_name, segment);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            position,
        )?)
    }

    pub fn save_end_position(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        offset: u64,
        position: u128,
    ) -> Result<(), JournalServerError> {
        let key = end_position_index_key(namespace, shard_name, segment);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            position,
        )?)
    }

    pub fn delete_segment(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        offset: u64,
    ) -> Result<(), JournalServerError> {
        let key = start_position_index_key(namespace, shard_name, segment);
        rocksdb_engine_delete(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
        )?;

        let key = end_position_index_key(namespace, shard_name, segment);
        Ok(rocksdb_engine_delete(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
        )?)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn segment_index_test() {}
}
