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

use super::engine::RocksDBStorageEngine;
use crate::core::error::StorageEngineError;
use rocksdb_engine::keys::storage::{
    key_index_prefix, shard_record_key_prefix, shard_record_shard_prefix, shard_segment_leo_key,
    shard_segment_leo_shard_prefix, tag_index_prefix, timestamp_index_prefix,
};

impl RocksDBStorageEngine {
    /// Delete all record data and indices for `shard_name`.
    pub fn delete_by_shard(&self, shard_name: &str) -> Result<(), StorageEngineError> {
        let cf = self.get_cf()?;
        self.rocksdb_engine_handler
            .delete_prefix(cf.clone(), &shard_record_shard_prefix(shard_name))
            .map_err(|e| StorageEngineError::CommonErrorStr(e.to_string()))?;
        self.rocksdb_engine_handler
            .delete_prefix(cf.clone(), &shard_segment_leo_shard_prefix(shard_name))
            .map_err(|e| StorageEngineError::CommonErrorStr(e.to_string()))?;
        self.rocksdb_engine_handler
            .delete_prefix(cf.clone(), &key_index_prefix(shard_name))
            .map_err(|e| StorageEngineError::CommonErrorStr(e.to_string()))?;
        self.rocksdb_engine_handler
            .delete_prefix(cf.clone(), &tag_index_prefix(shard_name))
            .map_err(|e| StorageEngineError::CommonErrorStr(e.to_string()))?;
        self.rocksdb_engine_handler
            .delete_prefix(cf, &timestamp_index_prefix(shard_name))
            .map_err(|e| StorageEngineError::CommonErrorStr(e.to_string()))?;
        Ok(())
    }

    /// Delete record data for one segment.
    ///
    /// Key/tag/timestamp indices are shard-level (no segment_seq) and cannot be
    /// cleanly range-deleted per segment without a full scan; they are left as
    /// dead references and tolerated.
    pub fn delete_by_segment(
        &self,
        shard_name: &str,
        segment_seq: u32,
    ) -> Result<(), StorageEngineError> {
        let cf = self.get_cf()?;
        self.rocksdb_engine_handler
            .delete_prefix(
                cf.clone(),
                &shard_record_key_prefix(shard_name, segment_seq),
            )
            .map_err(|e| StorageEngineError::CommonErrorStr(e.to_string()))?;
        self.rocksdb_engine_handler
            .delete(cf, &shard_segment_leo_key(shard_name, segment_seq))
            .map_err(|e| StorageEngineError::CommonErrorStr(e.to_string()))?;
        Ok(())
    }
}
