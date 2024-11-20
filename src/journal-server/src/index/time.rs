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

use super::keys::{timestamp_segment_end, timestamp_segment_start, timestamp_segment_time};
use crate::core::consts::DB_COLUMN_FAMILY_INDEX;
use crate::core::error::JournalServerError;

pub struct TimestampIndexManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl TimestampIndexManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        TimestampIndexManager {
            rocksdb_engine_handler,
        }
    }

    pub fn save_start_timestamp(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        start_timestamp: u64,
    ) -> Result<(), JournalServerError> {
        let key = timestamp_segment_start(namespace, shard_name, segment);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            start_timestamp,
        )?)
    }

    pub fn get_start_timestamp(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
    ) -> Result<u64, JournalServerError> {
        let key = timestamp_segment_start(namespace, shard_name, segment);
        if let Some(res) = rocksdb_engine_get(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
        )? {
            return Ok(serde_json::from_slice::<u64>(&res.data)?);
        }

        Ok(0)
    }

    pub fn save_end_timestamp(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        end_timestamp: u64,
    ) -> Result<(), JournalServerError> {
        let key = timestamp_segment_end(namespace, shard_name, segment);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            end_timestamp,
        )?)
    }

    pub fn get_end_timestamp(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
    ) -> Result<u64, JournalServerError> {
        let key = timestamp_segment_end(namespace, shard_name, segment);
        if let Some(res) = rocksdb_engine_get(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
        )? {
            return Ok(serde_json::from_slice::<u64>(&res.data)?);
        }

        Ok(0)
    }

    pub fn save_timestamp_offset(
        &self,
        namespace: &str,
        shard_name: &str,
        segment: u32,
        timestamp: u64,
        position: u64,
    ) -> Result<(), JournalServerError> {
        let key = timestamp_segment_time(namespace, shard_name, segment, timestamp);
        Ok(rocksdb_engine_save(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            key,
            position,
        )?)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn timestamp_index_test() {}
}
