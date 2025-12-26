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

use crate::core::error::StorageEngineError;
use crate::segment::keys::{
    offset_segment_end, offset_segment_start, timestamp_segment_end, timestamp_segment_start,
};
use crate::segment::SegmentIdentity;
use common_base::utils::serialize;
use rocksdb::WriteBatch;
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::engine::{engine_get_by_engine, engine_save_by_engine};
use rocksdb_engine::storage::family::DB_COLUMN_FAMILY_STORAGE_ENGINE;
use std::sync::Arc;

pub struct SegmentIndexManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl SegmentIndexManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        SegmentIndexManager {
            rocksdb_engine_handler,
        }
    }

    pub fn save_start_offset(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: i64,
    ) -> Result<(), StorageEngineError> {
        let key = offset_segment_start(segment_iden);
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
        let key = offset_segment_start(segment_iden);
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
        let key = offset_segment_end(segment_iden);
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
        let key = offset_segment_end(segment_iden);
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
        let key = timestamp_segment_start(segment_iden);
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
        let key = timestamp_segment_start(segment_iden);
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
        let key = timestamp_segment_end(segment_iden);
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
        let key = timestamp_segment_end(segment_iden);
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

        let key = offset_segment_start(segment_iden);
        batch.put_cf(&cf, key, serialize::serialize(&start_offset)?);

        let key = offset_segment_end(segment_iden);
        batch.put_cf(&cf, key, serialize::serialize(&end_offset)?);

        let key = timestamp_segment_start(segment_iden);
        batch.put_cf(&cf, key, serialize::serialize(&start_timestamp)?);

        let key = timestamp_segment_end(segment_iden);
        batch.put_cf(&cf, key, serialize::serialize(&end_timestamp)?);

        Ok(self.rocksdb_engine_handler.write_batch(batch)?)
    }
}

#[cfg(test)]
mod tests {
    use super::SegmentIndexManager;
    use crate::core::test::test_build_segment;
    use rocksdb_engine::test::test_rocksdb_instance;

    #[test]
    fn start_end_timestamp_index_test() {
        let rocksdb_engine_handler = test_rocksdb_instance();
        let segment_iden = test_build_segment();

        let offset_index = SegmentIndexManager::new(rocksdb_engine_handler);

        let start_offset = 100;
        let res = offset_index.save_start_offset(&segment_iden, start_offset);
        assert!(res.is_ok());

        let res = offset_index.get_start_offset(&segment_iden);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), start_offset);

        let end_offset = 1000;
        let res = offset_index.save_end_offset(&segment_iden, end_offset);
        assert!(res.is_ok());

        let res = offset_index.get_end_offset(&segment_iden);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), end_offset);
    }

    #[test]
    fn start_end_offset_index_test() {
        let rocksdb_engine_handler = test_rocksdb_instance();
        let segment_iden = test_build_segment();

        let offset_index = SegmentIndexManager::new(rocksdb_engine_handler);

        let start_offset = 100;
        let res = offset_index.save_start_offset(&segment_iden, start_offset);
        assert!(res.is_ok());

        let res = offset_index.get_start_offset(&segment_iden);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), start_offset);

        let end_offset = 1000;
        let res = offset_index.save_end_offset(&segment_iden, end_offset);
        assert!(res.is_ok());

        let res = offset_index.get_end_offset(&segment_iden);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), end_offset);
    }
}
