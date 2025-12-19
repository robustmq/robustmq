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

use crate::storage::keys::{key_all_segment, key_segment, key_segment_shard_prefix};
use common_base::error::common::CommonError;
use metadata_struct::storage::segment::{JournalSegment, SegmentStatus};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::meta_metadata::{
    engine_delete_by_meta_metadata, engine_get_by_meta_metadata,
    engine_prefix_list_by_meta_metadata, engine_save_by_meta_metadata,
};
use std::sync::Arc;

#[allow(dead_code)]
pub fn is_seal_up_segment(status: &SegmentStatus) -> bool {
    *status == SegmentStatus::PreSealUp || *status == SegmentStatus::SealUp
}

pub struct SegmentStorage {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl SegmentStorage {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        SegmentStorage {
            rocksdb_engine_handler,
        }
    }

    pub fn save(&self, segment: JournalSegment) -> Result<(), CommonError> {
        let shard_key = key_segment(&segment.shard_name, segment.segment_seq);
        engine_save_by_meta_metadata(self.rocksdb_engine_handler.clone(), &shard_key, segment)
    }

    pub fn get(
        &self,
        shard_name: &str,
        segment_seq: u32,
    ) -> Result<Option<JournalSegment>, CommonError> {
        let shard_key: String = key_segment(shard_name, segment_seq);
        if let Some(data) = engine_get_by_meta_metadata::<JournalSegment>(
            self.rocksdb_engine_handler.clone(),
            &shard_key,
        )? {
            return Ok(Some(data.data));
        }
        Ok(None)
    }

    pub fn all_segment(&self) -> Result<Vec<JournalSegment>, CommonError> {
        let prefix_key = key_all_segment();
        let data = engine_prefix_list_by_meta_metadata::<JournalSegment>(
            self.rocksdb_engine_handler.clone(),
            prefix_key,
        )?;
        let mut results = Vec::new();
        for raw in data {
            results.push(raw.data);
        }
        Ok(results)
    }

    pub fn list_by_shard(&self, shard_name: &str) -> Result<Vec<JournalSegment>, CommonError> {
        let prefix_key = key_segment_shard_prefix(shard_name);
        let data = engine_prefix_list_by_meta_metadata::<JournalSegment>(
            self.rocksdb_engine_handler.clone(),
            &prefix_key,
        )?;
        let mut results = Vec::new();
        for raw in data {
            results.push(raw.data);
        }
        Ok(results)
    }

    pub fn delete(&self, shard_name: &str, segment_seq: u32) -> Result<(), CommonError> {
        let shard_key = key_segment(shard_name, segment_seq);
        engine_delete_by_meta_metadata(self.rocksdb_engine_handler.clone(), &shard_key)
    }
}

#[cfg(test)]
mod test {}
