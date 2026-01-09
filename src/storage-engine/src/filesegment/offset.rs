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
        segment_offset::SegmentOffset,
        SegmentIdentity,
    },
};
use common_base::tools::now_second;
use metadata_struct::storage::adapter_offset::AdapterOffsetStrategy;
use rocksdb_engine::rocksdb::RocksDBEngine;
use std::sync::Arc;

#[derive(Clone)]
pub struct FileSegmentOffset {
    pub rocksdb_engine_handler: Arc<RocksDBEngine>,
    pub cache_manager: Arc<StorageCacheManager>,
    pub segment_offset: SegmentOffset,
}

impl FileSegmentOffset {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<StorageCacheManager>,
    ) -> Self {
        FileSegmentOffset {
            rocksdb_engine_handler: rocksdb_engine_handler.clone(),
            cache_manager,
            segment_offset: SegmentOffset::new(rocksdb_engine_handler.clone()),
        }
    }

    pub fn get_latest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        let segment = if let Some(shard) = self.cache_manager.get_active_segment(shard_name) {
            shard.clone()
        } else {
            return Err(StorageEngineError::ShardNotExist(shard_name.to_string()));
        };

        // The end offset of the active segment is restored from persistence.
        let segment_iden = SegmentIdentity::new(shard_name, segment.segment_seq);
        let offset = self.segment_offset.get_end_offset(&segment_iden)?;
        if offset < 0 {
            return Err(StorageEngineError::CommonErrorStr("".to_string()));
        }
        Ok(offset as u64)
    }

    pub fn save_latest_offset(
        &self,
        segment_iden: &SegmentIdentity,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        self.cache_manager.update_end_meta(segment_iden, offset);
        self.segment_offset
            .save_end_offset(segment_iden, offset as i64)?;
        self.segment_offset
            .save_end_timestamp(segment_iden, now_second() as i64)?;
        Ok(())
    }

    pub fn get_earliest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        let shard = if let Some(shard) = self.cache_manager.shards.get(shard_name) {
            shard.clone()
        } else {
            return Err(StorageEngineError::ShardNotExist(shard_name.to_string()));
        };

        let segment_iden = SegmentIdentity::new(shard_name, shard.start_segment_seq);
        let meta = if let Some(meta) = self.cache_manager.get_segment_meta(&segment_iden) {
            meta.clone()
        } else {
            return Err(StorageEngineError::SegmentMetaNotExists(
                segment_iden.name(),
            ));
        };

        if meta.start_offset < 0 {
            return Err(StorageEngineError::CommonErrorStr("".to_string()));
        }
        Ok(meta.start_offset as u64)
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
