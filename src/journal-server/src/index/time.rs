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
use rocksdb_engine::storage::journal::{engine_get_by_journal, engine_save_by_journal};
use rocksdb_engine::warp::StorageDataWrap;

use super::keys::{
    timestamp_segment_end, timestamp_segment_start, timestamp_segment_time,
    timestamp_segment_time_prefix,
};
use super::IndexData;
use crate::core::consts::DB_COLUMN_FAMILY_INDEX;
use crate::core::error::JournalServerError;
use crate::segment::SegmentIdentity;

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
        segment_iden: &SegmentIdentity,
        start_timestamp: u64,
    ) -> Result<(), JournalServerError> {
        let key = timestamp_segment_start(segment_iden);
        Ok(engine_save_by_journal(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            &key,
            start_timestamp,
        )?)
    }

    pub fn get_start_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<i64, JournalServerError> {
        let key = timestamp_segment_start(segment_iden);
        if let Some(res) = engine_get_by_journal::<i64>(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            &key,
        )? {
            return Ok(res.data);
        }

        Ok(-1)
    }

    pub fn save_end_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
        end_timestamp: u64,
    ) -> Result<(), JournalServerError> {
        let key = timestamp_segment_end(segment_iden);
        Ok(engine_save_by_journal(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            &key,
            end_timestamp,
        )?)
    }

    pub fn get_end_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
    ) -> Result<i64, JournalServerError> {
        let key = timestamp_segment_end(segment_iden);
        if let Some(res) = engine_get_by_journal::<i64>(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            &key,
        )? {
            return Ok(res.data);
        }

        Ok(-1)
    }

    pub fn save_timestamp_offset(
        &self,
        segment_iden: &SegmentIdentity,
        timestamp: u64,
        index_data: IndexData,
    ) -> Result<(), JournalServerError> {
        let key = timestamp_segment_time(segment_iden, timestamp);
        Ok(engine_save_by_journal(
            self.rocksdb_engine_handler.clone(),
            DB_COLUMN_FAMILY_INDEX,
            &key,
            index_data,
        )?)
    }

    //todo Optimize the fetching to avoid scanning the RocksDb index every time
    pub async fn get_last_nearest_position_by_timestamp(
        &self,
        segment_iden: &SegmentIdentity,
        start_timestamp: u64,
    ) -> Result<Option<IndexData>, JournalServerError> {
        let prefix_key = timestamp_segment_time_prefix(segment_iden);

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

                    if index_data.timestamp < start_timestamp {
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

    use super::TimestampIndexManager;
    use crate::core::test::test_build_rocksdb_sgement;
    use crate::index::IndexData;

    #[test]
    fn start_end_index_test() {
        let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();

        let time_index = TimestampIndexManager::new(rocksdb_engine_handler);

        let start_timestamp = now_second();
        let res = time_index.save_start_timestamp(&segment_iden, start_timestamp);
        assert!(res.is_ok());

        let res = time_index.get_start_timestamp(&segment_iden);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), start_timestamp as i64);

        let end_timestamp = now_second();
        let res = time_index.save_end_timestamp(&segment_iden, end_timestamp);
        assert!(res.is_ok());

        let res = time_index.get_end_timestamp(&segment_iden);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), end_timestamp as i64);
    }

    #[tokio::test]
    async fn timestamp_index_test() {
        let (rocksdb_engine_handler, segment_iden) = test_build_rocksdb_sgement();

        let time_index = TimestampIndexManager::new(rocksdb_engine_handler);

        let timestamp = now_second();

        for i in 0..10 {
            let cur_timestamp = timestamp + i * 10;
            let index_data = IndexData {
                offset: i,
                timestamp: cur_timestamp,
                position: i * 5,
            };
            let res = time_index.save_timestamp_offset(&segment_iden, cur_timestamp, index_data);
            assert!(res.is_ok());
        }

        // get timestamp + 0
        let res = time_index
            .get_last_nearest_position_by_timestamp(&segment_iden, timestamp)
            .await;

        assert!(res.is_ok());
        let res_op = res.unwrap();
        assert!(res_op.is_some());
        let data = res_op.unwrap();
        assert_eq!(data.offset, 0);

        // get timestamp + 10
        let res = time_index
            .get_last_nearest_position_by_timestamp(&segment_iden, timestamp + 10)
            .await;
        assert!(res.is_ok());
        let res_op = res.unwrap();
        assert!(res_op.is_some());
        let data = res_op.unwrap();
        assert_eq!(data.offset, 1);

        // get timestamp + 50
        let res = time_index
            .get_last_nearest_position_by_timestamp(&segment_iden, timestamp + 50)
            .await;
        assert!(res.is_ok());
        let res_op = res.unwrap();
        assert!(res_op.is_some());
        let data = res_op.unwrap();
        assert_eq!(data.offset, 5);
    }
}
