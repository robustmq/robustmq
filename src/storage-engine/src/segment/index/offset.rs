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

use common_base::{error::common::CommonError, utils::serialize};
use rocksdb_engine::rocksdb::RocksDBEngine;
use rocksdb_engine::storage::engine::{engine_get_by_engine, engine_save_by_engine};
use rocksdb_engine::warp::StorageDataWrap;

use crate::segment::keys::{
    offset_segment_end, offset_segment_position, offset_segment_position_prefix,
    offset_segment_start,
};
use super::IndexData;
use crate::core::consts::DB_COLUMN_FAMILY_INDEX;
use crate::core::error::StorageEngineError;
use crate::segment::SegmentIdentity;

pub struct OffsetIndexManager {
    rocksdb_engine_handler: Arc<RocksDBEngine>,
}

impl OffsetIndexManager {
    pub fn new(rocksdb_engine_handler: Arc<RocksDBEngine>) -> Self {
        OffsetIndexManager {
            rocksdb_engine_handler,
        }
    }

    pub fn save_start_offset(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: u64,
    ) -> Result<(), StorageEngineError> {
        let key = offset_segment_start(segment_iden);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_INDEX,
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
            DB_COLUMN_FAMILY_INDEX,
            &key,
        )? {
            return Ok(res.data);
        }

        Ok(-1)
    }

    pub fn save_end_offset(
        &self,
        segment_iden: &SegmentIdentity,
        end_offset: u64,
    ) -> Result<(), StorageEngineError> {
        let key = offset_segment_end(segment_iden);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_INDEX,
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
            DB_COLUMN_FAMILY_INDEX,
            &key,
        )? {
            return Ok(res.data);
        }

        Ok(-1)
    }

    pub fn save_position_offset(
        &self,
        segment_iden: &SegmentIdentity,
        offset: u64,
        index_data: IndexData,
    ) -> Result<(), StorageEngineError> {
        let key = offset_segment_position(segment_iden, offset);
        Ok(engine_save_by_engine(
            &self.rocksdb_engine_handler,
            DB_COLUMN_FAMILY_INDEX,
            &key,
            index_data,
        )?)
    }

    //todo Optimize the fetching to avoid scanning the RocksDb index every time
    pub async fn get_last_nearest_position_by_offset(
        &self,
        segment_iden: &SegmentIdentity,
        start_offset: u64,
    ) -> Result<Option<IndexData>, StorageEngineError> {
        let prefix_key = offset_segment_position_prefix(segment_iden);

        let cf = if let Some(cf) = self
            .rocksdb_engine_handler
            .cf_handle(DB_COLUMN_FAMILY_INDEX)
        {
            cf
        } else {
            return Err(
                CommonError::RocksDBFamilyNotAvailable(DB_COLUMN_FAMILY_INDEX.to_string()).into(),
            );
        };

        let mut iter = self.rocksdb_engine_handler.db.raw_iterator_cf(&cf);
        iter.seek(prefix_key.clone());

        while iter.valid() {
            if let Some(key) = iter.key() {
                if let Some(val) = iter.value() {
                    let key = String::from_utf8(key.to_vec())?;
                    if !key.starts_with(&prefix_key) {
                        break;
                    }

                    let data = serialize::deserialize::<StorageDataWrap<IndexData>>(val)?;
                    let index_data = data.data;

                    if index_data.offset < start_offset {
                        iter.next();
                        continue;
                    }

                    return Ok(Some(index_data));
                }
            }
            iter.next();
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {

    use common_base::tools::now_second;

    use super::OffsetIndexManager;
    use crate::core::test::test_build_rocksdb_sgement;
    use crate::segment::index::IndexData;

    #[test]
    fn start_end_index_test() {
        let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();

        let offset_index = OffsetIndexManager::new(rocksdb_engine_handler);

        let start_offset = 100;
        let res = offset_index.save_start_offset(&segment_iden, start_offset);
        assert!(res.is_ok());

        let res = offset_index.get_start_offset(&segment_iden);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), start_offset as i64);

        let end_offset = 1000;
        let res = offset_index.save_end_offset(&segment_iden, end_offset);
        assert!(res.is_ok());

        let res = offset_index.get_end_offset(&segment_iden);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), end_offset as i64);
    }

    #[tokio::test]
    async fn timestamp_index_test() {
        let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();

        let offset_index = OffsetIndexManager::new(rocksdb_engine_handler);

        let timestamp = now_second();

        for i in 0..10 {
            let cur_timestamp = timestamp + i * 10;
            let index_data = IndexData {
                offset: i,
                timestamp: cur_timestamp,
                position: i * 5,
            };
            let res = offset_index.save_position_offset(&segment_iden, i, index_data);
            assert!(res.is_ok());
        }

        // offset 0
        let res = offset_index
            .get_last_nearest_position_by_offset(&segment_iden, 0)
            .await;

        assert!(res.is_ok());
        let res_op = res.unwrap();
        assert!(res_op.is_some());
        assert_eq!(res_op.unwrap().offset, 0);

        // offset 6
        let res = offset_index
            .get_last_nearest_position_by_offset(&segment_iden, 6)
            .await;

        assert!(res.is_ok());
        let res_op = res.unwrap();
        assert!(res_op.is_some());
        assert_eq!(res_op.unwrap().offset, 6);

        // offset 9
        let res = offset_index
            .get_last_nearest_position_by_offset(&segment_iden, 9)
            .await;

        assert!(res.is_ok());
        let res_op = res.unwrap();
        assert!(res_op.is_some());
        assert_eq!(res_op.unwrap().offset, 9);
    }
}
