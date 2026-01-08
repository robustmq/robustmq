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

use crate::{
    core::{cache::StorageCacheManager, error::StorageEngineError},
    filesegment::{
        index::read::{get_in_segment_by_timestamp, get_index_data_by_timestamp},
        SegmentIdentity,
    },
};
use metadata_struct::storage::adapter_offset::AdapterOffsetStrategy;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

#[derive(Clone)]
pub struct FileSegmentOffset {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub cache_manager: Arc<StorageCacheManager>,
}

impl FileSegmentOffset {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<StorageCacheManager>,
    ) -> Self {
        FileSegmentOffset {
            rocksdb_engine_handler,
            cache_manager,
        }
    }

    pub fn save_latest_offset(
        &self,
        shard_name: &str,
        offset: u64,
    ) -> Result<u64, StorageEngineError> {
        Ok(0)
    }

    pub fn get_latest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        
        Ok(0)
    }

    pub fn save_earliest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        Ok(0)
    }

    pub fn get_earliest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        Ok(0)
    }

    pub fn get_offset_by_timestamp(
        &self,
        shard_name: &str,
        timestamp: u64,
        strategy: AdapterOffsetStrategy,
    ) -> Result<u64, StorageEngineError> {
        if let Some(segment) =
            get_in_segment_by_timestamp(&self.cache_manager, shard_name, timestamp as i64)?
        {
            let segment_iden = SegmentIdentity::new(shard_name, segment);
            if let Some(index_data) =
                get_index_data_by_timestamp(&self.rocksdb_engine_handler, &segment_iden, timestamp)?
            {
                Ok(index_data.offset)
            } else {
                Err(StorageEngineError::CommonErrorStr(format!(
                    "No index data found for timestamp {} in segment {}",
                    timestamp, segment
                )))
            }
        } else {
            match strategy {
                AdapterOffsetStrategy::Earliest => self.get_earliest_offset(shard_name),
                AdapterOffsetStrategy::Latest => self.get_latest_offset(shard_name),
            }
        }
    }
}
