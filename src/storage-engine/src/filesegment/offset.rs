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
use common_base::tools::now_second;
use metadata_struct::adapter::adapter_offset::AdapterOffsetStrategy;
use rocksdb::WriteBatch;
use rocksdb_engine::keys::engine::{
    offset_segment_end, offset_segment_high_watermark, offset_segment_start, timestamp_segment_end,
    timestamp_segment_start,
};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::engine::{engine_get_by_engine, engine_save_by_engine};
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_STORAGE_ENGINE;
use std::sync::Arc;

#[derive(Clone)]
pub struct SegmentOffset {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
    cache_manager: Arc<StorageCacheManager>,
}

impl SegmentOffset {
    pub fn new(
        rocksdb_engine_handler: Arc<RocksDBEngine>,
        cache_manager: Arc<StorageCacheManager>,
    ) -> Self {
        SegmentOffset {
            rocksdb_engine_handler,
            cache_manager,
        }
    }

    // === persistence: raw offset / timestamp ===

    pub fn save_start_offset(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: i64,
    ) -> Result<(), StorageEngineError> {
        let key = offset_segment_start(&segment_iden.shard_name, segment_iden.segment);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
            start_offset,
        )?)
    }

    pub fn get_start_offset(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<i64, StorageEngineError> {
        let key = offset_segment_start(&segment_iden.shard_name, segment_iden.segment);
        if let Some(res) = engine_get_by_engine::<i64>(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
        )? {
            return Ok(res.data);
        }
        Ok(-1)
    }

    pub fn save_high_watermark_offset(
        &self,
        segment_iden: &SegmentIdentity,
        end_offset: i64,
    ) -> Result<(), StorageEngineError> {
        let key = offset_segment_high_watermark(&segment_iden.shard_name, segment_iden.segment);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
            end_offset,
        )?)
    }

    pub fn get_high_watermark_offset(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<i64, StorageEngineError> {
        let key = offset_segment_high_watermark(&segment_iden.shard_name, segment_iden.segment);
        if let Some(res) = engine_get_by_engine::<i64>(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
        )? {
            return Ok(res.data);
        }
        Ok(-1)
    }

    pub fn save_end_offset(
        &self,
        segment_iden: &SegmentIdentity,
        end_offset: i64,
    ) -> Result<(), StorageEngineError> {
        let key = offset_segment_end(&segment_iden.shard_name, segment_iden.segment);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
            end_offset,
        )?)
    }

    pub fn get_end_offset(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<i64, StorageEngineError> {
        let key = offset_segment_end(&segment_iden.shard_name, segment_iden.segment);
        if let Some(res) = engine_get_by_engine::<i64>(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
        )? {
            return Ok(res.data);
        }
        Ok(-1)
    }

    pub fn save_start_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
        start_timestamp: i64,
    ) -> Result<(), StorageEngineError> {
        let key = timestamp_segment_start(&segment_iden.shard_name, segment_iden.segment);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
            start_timestamp,
        )?)
    }

    pub fn get_start_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<i64, StorageEngineError> {
        let key = timestamp_segment_start(&segment_iden.shard_name, segment_iden.segment);
        if let Some(res) = engine_get_by_engine::<i64>(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
        )? {
            return Ok(res.data);
        }
        Ok(-1)
    }

    pub fn save_end_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
        end_timestamp: i64,
    ) -> Result<(), StorageEngineError> {
        let key = timestamp_segment_end(&segment_iden.shard_name, segment_iden.segment);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
            end_timestamp,
        )?)
    }

    pub fn get_end_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<i64, StorageEngineError> {
        let key = timestamp_segment_end(&segment_iden.shard_name, segment_iden.segment);
        if let Some(res) = engine_get_by_engine::<i64>(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_STORAGE_ENGINE,
            &key,
        )? {
            return Ok(res.data);
        }
        Ok(-1)
    }

    pub fn batch_save_segment_metadata(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: i64,
        end_offset: i64,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> Result<(), StorageEngineError> {
        use rocksdb_engine::storage::base::batch_encode_data;

        let cf = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_STORAGE_ENGINE)
            .ok_or_else(|| {
                StorageEngineError::CommonErrorStr(format!(
                    "Column family '{}' not found",
                    DB_COLUMN_FAMILY_STORAGE_ENGINE
                ))
            })?;

        let mut batch = WriteBatch::default();
        let key = offset_segment_start(&segment_iden.shard_name, segment_iden.segment);
        batch.put_cf(&cf, key, batch_encode_data(start_offset)?);
        let key = offset_segment_end(&segment_iden.shard_name, segment_iden.segment);
        batch.put_cf(&cf, key, batch_encode_data(end_offset)?);
        let key = timestamp_segment_start(&segment_iden.shard_name, segment_iden.segment);
        batch.put_cf(&cf, key, batch_encode_data(start_timestamp)?);
        let key = timestamp_segment_end(&segment_iden.shard_name, segment_iden.segment);
        batch.put_cf(&cf, key, batch_encode_data(end_timestamp)?);

        Ok(self.rocksdb_engine_handler.write_batch(batch)?)
    }

    // === higher-level: cache-aware offset queries ===

    /// Return the correct "next write offset" for a segment that this node has
    /// not yet written to.  When `offset_segment_end` is 0 (written by
    /// parse_segment_meta with the initial end_offset=0), the segment is empty
    /// on this node and writes must start at `meta.start_offset`, not 0.
    pub fn get_segment_next_write_offset(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<u64, StorageEngineError> {
        let end = self.get_end_offset(segment_iden)?;
        if end > 0 {
            return Ok(end as u64);
        }
        // end_offset is 0 or missing: use start_offset from metadata
        if let Some(meta) = self.cache_manager.get_segment_meta(segment_iden) {
            if meta.start_offset > 0 {
                return Ok(meta.start_offset as u64);
            }
        }
        Ok(0)
    }

    pub fn get_latest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        let segment = self
            .cache_manager
            .get_active_segment(shard_name)
            .ok_or_else(|| StorageEngineError::ShardNotExist(shard_name.to_string()))?;

        let segment_iden = SegmentIdentity::new(shard_name, segment.segment_seq);
        let offset = self.get_end_offset(&segment_iden)?;
        if offset < 0 {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "Invalid end offset {} for shard '{}' segment {}: offset cannot be negative",
                offset, shard_name, segment.segment_seq
            )));
        }
        Ok(offset as u64)
    }

    pub fn save_latest_offset(
        &self,
        segment_iden: &SegmentIdentity,
        offset: u64,
    ) -> Result<(), StorageEngineError> {
        // Do NOT update the in-memory EngineSegmentMetadata.end_offset here.
        // end_offset in the DashMap cache must only be set by meta-service notifications
        // (create_next_segment seals the old segment and notifies the engine). If we
        // set end_offset = next_write_offset here, is_end_reached fires prematurely on
        // the very next batch that contains that offset, causing runaway sealing.
        self.save_end_offset(segment_iden, offset as i64)?;
        self.save_end_timestamp(segment_iden, now_second() as i64)?;
        Ok(())
    }

    pub fn get_earliest_offset(&self, shard_name: &str) -> Result<u64, StorageEngineError> {
        let shard = self
            .cache_manager
            .shards
            .get(shard_name)
            .ok_or_else(|| StorageEngineError::ShardNotExist(shard_name.to_string()))?
            .clone();

        let segment_iden = SegmentIdentity::new(shard_name, shard.start_segment_seq);
        let meta = self
            .cache_manager
            .get_segment_meta(&segment_iden)
            .ok_or_else(|| StorageEngineError::SegmentMetaNotExists(segment_iden.name()))?
            .clone();

        if meta.start_offset < 0 {
            return Err(StorageEngineError::CommonErrorStr(format!(
                "Invalid start offset {} for shard '{}' segment {}: offset cannot be negative",
                meta.start_offset, shard_name, shard.start_segment_seq
            )));
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

#[cfg(test)]
mod tests {
    use super::SegmentOffset;
    use crate::core::test_tool::test_init_segment;
    use common_config::storage::StorageType;

    #[tokio::test]
    async fn start_end_offset_test() {
        let (segment_iden, cache_manager, _, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;
        let so = SegmentOffset::new(rocksdb, cache_manager);

        so.save_start_offset(&segment_iden, 100).unwrap();
        assert_eq!(so.get_start_offset(&segment_iden).unwrap(), 100);

        so.save_end_offset(&segment_iden, 1000).unwrap();
        assert_eq!(so.get_end_offset(&segment_iden).unwrap(), 1000);
    }

    #[tokio::test]
    async fn batch_save_segment_metadata_test() {
        let (segment_iden, cache_manager, _, rocksdb) =
            test_init_segment(StorageType::EngineSegment).await;
        let so = SegmentOffset::new(rocksdb, cache_manager);

        so.batch_save_segment_metadata(&segment_iden, 100, 1000, 1609459200, 1609545600)
            .unwrap();

        assert_eq!(so.get_start_offset(&segment_iden).unwrap(), 100);
        assert_eq!(so.get_end_offset(&segment_iden).unwrap(), 1000);
        assert_eq!(so.get_start_timestamp(&segment_iden).unwrap(), 1609459200);
        assert_eq!(so.get_end_timestamp(&segment_iden).unwrap(), 1609545600);
    }
}
